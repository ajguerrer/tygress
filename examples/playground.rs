use tygress::netdev::{Event, NetDev, Topology, TunTapInterface};

fn main() {
    let name = "tun0";
    let socket = TunTapInterface::bind(name, Topology::Ip)
        .unwrap_or_else(|_| panic!("failed to bind {}", name));
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
