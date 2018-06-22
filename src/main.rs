extern crate arcana;

use arcana::Daemon;

fn main() {
    let daemon = Daemon::new();
    daemon.run();
}
