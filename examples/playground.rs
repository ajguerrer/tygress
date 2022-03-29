use tygress::driver::Driver;
use tygress::netdev::{Layer, PacketSocket};

fn main() {
    let socket = PacketSocket::bind("eth0", Layer::Ethernet).expect("failed to bind eth0");
    let body = async {
        println!("hello world");
    };
    Driver::new(socket).turn(body);
}
