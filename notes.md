# Tygress

## Runtime

The tygress runtime relies on two core concepts: 

- network interface
- async I/O driver

### Network interface

Network interfaces are built slightly differently, based on Topology. Below are a list of dynamic
resources:

- **NeighborCache** - Map of IpAddr to EtherAddr, with expiry. EthernetII topology only.
- **SocketSet** - List of open network sockets. Need to determine if this will be an array of enum
  or multiple arrays, one per socket type. Sockets will generally need a pair of dynamic  TX/RX ring
  buffers.
- **Ipv4MulticastGroups** - Set of multicast Ipv4Addr's.
- **IpAddrs** - Set of IpAddr's bound to interface.
- **IpFragBuffers** - List of buffers used for reassembly of IP fragments. This is the first
  thing to clean up, should we start to run out of memory.
- **Routes** - Map of Cidr to IpAddr, with expiry.
- **ScheduledIO** - List of read and write wakers. Generally scales with the number of sockets.

The async I/O driver will need a way to reference itself globally. Socket objects will register
themselves with the I/O driver for waking via ScheduledIO.
