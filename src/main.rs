use futures_util::StreamExt;
use smol::block_on;
use std::fs::{File, OpenOptions};
use std::io::Write;
use zbus::Connection;
use zbus::fdo::PropertiesProxy;
use zbus::names::InterfaceName;

#[cfg(debug_assertions)]
static mut COUNTER: usize = 0;

fn main() {
    // let args = std::env::args();

    // let modes = Modes::get(args);
    block_on(async {
        let connection = Connection::system().await.unwrap();
        let service_name = "org.freedesktop.UPower";
        let device_path = "/org/freedesktop/UPower/devices/line_power_AC0";
        let proxy = PropertiesProxy::builder(&connection)
            .destination(service_name)
            .unwrap()
            .path(device_path)
            .unwrap()
            .build()
            .await
            .unwrap();

        let mut thermal_throttle_policy = OpenOptions::new()
            .write(true)
            .open("/sys/devices/platform/asus-nb-wmi/throttle_thermal_policy")
            .unwrap();

        let interface = InterfaceName::from_static_str("org.freedesktop.UPower.Device").unwrap();

        let charging_status: bool = proxy
            .get(interface.clone(), "Online")
            .await
            .unwrap()
            .try_into()
            .unwrap();

        handle_ttp(charging_status, &mut thermal_throttle_policy);

        let mut prop_stream = proxy.receive_properties_changed().await.unwrap();

        while let Some(signal) = prop_stream.next().await {
            let args = signal.args().unwrap();
            if args.interface_name == interface {
                if let Some(charging) = args.changed_properties().get("Online") {
                    handle_ttp(charging.try_into().unwrap(), &mut thermal_throttle_policy);
                }
            }
        }
    });
}

#[cfg(debug_assertions)]
#[allow(static_mut_refs)]
fn handle_ttp(charging: bool, ttp: &mut File) {
    unsafe {
        if charging {
            ttp.write_all("1\n".as_bytes()).unwrap();
            println!("{COUNTER}. Power Source : AC, Ttp set to Performance");
        } else {
            ttp.write_all("0\n".as_bytes()).unwrap();
            println!("{COUNTER}. Power Source : Battery, Ttp set to Default");
        }
        COUNTER += 1;
    }
}

#[cfg(not(debug_assertions))]
fn handle_ttp(charging: bool, ttp: &mut File) {
    if charging {
        ttp.write_all("1\n".as_bytes()).unwrap();
    } else {
        ttp.write_all("0\n".as_bytes()).unwrap();
    }
}
