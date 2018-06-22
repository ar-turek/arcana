extern crate dbus;

use std::sync::Arc;

pub struct Daemon {
    connection: dbus::Connection,
}

impl Daemon {
    pub fn new() -> Daemon {
        let daemon = match Daemon::connect() {
            Ok(connection) => Daemon { connection: connection },
            Err(_) => panic!("Failed to connect to DBus!"),
        };
        daemon.add_service_handler().expect("Failed to set handlers for DBus methods!");

        daemon
    }

    pub fn run(&self) {
        loop {
            self.connection.incoming(1000).next();
        }
    }

    fn connect() -> Result<dbus::Connection, dbus::Error> {
        let connection = dbus::Connection::get_private(dbus::BusType::Session)?;
        connection.register_name("org.freedesktop.secrets", dbus::NameFlag::ReplaceExisting as u32)?;
        Ok(connection)
    }

    fn add_service_handler(&self) -> Result<(), dbus::Error> {
        let fn_factory = Daemon::get_fn_factory();

        let signal = Arc::new(fn_factory.signal("Secrets", ()).sarg::<&str, _>("sender"));
        let signal_clone = signal.clone();

        let tree = fn_factory.tree(()).add(
            fn_factory.object_path("/", ())
                .introspectable()
                .add(
                    fn_factory.interface("org.freedesktop.secrets", ())
                    .add_m(
                        fn_factory.method(
                            "Secrets",
                            (),
                            move |m| {
                                let name: &str = m.msg.read1()?;
                                let s = format!("Hello {}!", name);
                                let mret = m.msg.method_return().append1(s);

                                let sig = signal_clone.msg(m.path.get_name(), m.iface.get_name())
                                    .append1(&*name);

                                Ok(vec!(mret, sig))
                            }
                        )
                        .inarg::<&str, _>("name")
                        .outarg::<&str, _>("message")
                    )
                    .add_s(signal.clone())
                )
        );

        tree.set_registered(&self.connection, true)?;
        self.connection.add_handler(tree);

        Ok(())
    }

    fn get_fn_factory() -> dbus::tree::Factory<dbus::tree::MTFn> {
        dbus::tree::Factory::new_fn::<()>()
    }
}
