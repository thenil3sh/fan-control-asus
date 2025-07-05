use futures_util::StreamExt;
use smol::block_on;
use zbus::Connection;
use zbus::fdo::PropertiesProxy;
use zbus::names::InterfaceName;

fn main() {
    let connection = block_on(async { Connection::system().await }).unwrap();
    let service_name = "org.freedesktop.UPower";
    let root_path = "/org/freedesktop/UPower/devices/line_power_AC0";

    let proxy = block_on(async {
        PropertiesProxy::builder(&connection)
            .destination(service_name)
            .unwrap()
            .path(root_path)
            .unwrap()
            .build()
            .await
    })
    .unwrap();

    let interface = InterfaceName::from_static_str("org.freedesktop.UPower.Device").unwrap();

    let mut prop_stream = block_on(async { proxy.receive_properties_changed().await }).unwrap();

    block_on(async {
        while let Some(signal) = prop_stream.next().await {
            let args = signal.args().unwrap();
            if &args.interface_name == &interface {
                if let Some(percentage) = args.changed_properties().get("Online") {
                    println!("Device is charging : {percentage}");
                }
            }
        }
    });

    todo!("Now track battery level as well, concurrently");
    todo!("Gain root access to edit /sys/devices/platform/asus-nb-wmi/throttle_thermal_policy");
    todo!("or maybe use dbus for that, ;)")
}
