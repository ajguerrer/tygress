use tygress::driver::Driver;
use tygress::netdev::{PacketSocket, Topology};

fn main() {
    let socket = PacketSocket::bind("eth0", Topology::EthernetII).expect("failed to bind eth0");
    let body = async {
        println!("hello world");
    };
    Driver::<PacketSocket, 1400>::new(socket).turn(body);
}
