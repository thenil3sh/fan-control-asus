// use fan_control_asus::Modes;
use futures_util::StreamExt;
use smol::block_on;
use std::fs::{File, OpenOptions};
use std::io::Write;
use zbus::Connection;
use zbus::fdo::PropertiesProxy;
use zbus::names::InterfaceName;

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
    // let connection = block_on(async { Connection::system().await }).unwrap();
    // let service_name = "org.freedesktop.UPower";
    // let device_path = "/org/freedesktop/UPower/devices/line_power_AC0";

    // let proxy = block_on(async {
    //     PropertiesProxy::builder(&connection)
    //         .destination(service_name)
    //         .unwrap()
    //         .path(device_path)
    //         .unwrap()
    //         .build()
    //         .await
    // })
    // .unwrap();
    // let mut thermal_throttle_policy = OpenOptions::new()
    //     .write(true)
    //     .open("/sys/devices/platform/asus-nb-wmi/throttle_thermal_policy")
    //     .unwrap();

    // let interface = InterfaceName::from_static_str("org.freedesktop.UPower.Device").unwrap();

    // block_on(async {
    //     let charging_status: bool = proxy
    //         .get(interface.clone(), "Online")
    //         .await
    //         .unwrap()
    //         .try_into()
    //         .unwrap();
    //     handle_ttp(charging_status, &mut thermal_throttle_policy);
    // });

    // let mut prop_stream = block_on(async { proxy.receive_properties_changed().await }).unwrap();

    // block_on(async {
    //     while let Some(signal) = prop_stream.next().await {
    //         let args = signal.args().unwrap();
    //         if args.interface_name == interface {
    //             if let Some(charging) = args.changed_properties().get("Online") {
    //                 handle_ttp(charging.try_into().unwrap(), &mut thermal_throttle_policy);
    //             }
    //         }
    //     }
    // });

    todo!("Now track battery level as well, concurrently");
    // todo!("access & edit /sys/devices/platform/asus-nb-wmi/throttle_thermal_policy");
}

fn handle_ttp(charging: bool, ttp: &mut File) {
    if charging {
        ttp.write_all("1\n".as_bytes()).unwrap();
        println!("Power source : AC, ttp set to Performance")
    } else {
        ttp.write_all("0\n".as_bytes()).unwrap();
        println!("Power source : Battery, ttp set to Normal")
    }
}
