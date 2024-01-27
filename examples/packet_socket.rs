use std::io;

use tygress::netdev::{Event, HardwareType, NetDev, PacketSocket};

fn main() -> io::Result<()> {
    let name = "enp8s0";
    let socket = PacketSocket::bind(name, HardwareType::Opaque)?;
    println!("mtu: {}", socket.mtu());
    let mut buf = vec![0; socket.mtu()];
    loop {
        let event = socket.poll(Event::READABLE, None).unwrap();
        if event.is_readable() {
            let read = socket.recv(&mut buf).unwrap();
            println!("{:?}", &buf[..read]);
        }
    }
}
