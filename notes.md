# Tygress

## Protocol

- [RFC 1122] - Requirements for Internet Hosts -- Communication Layers

### IPv4

- [RFC 791] - Internet Protocol
- [RFC 1191] - Path MTU discovery
- [RFC 826] - Ethernet Address Resolution Protocol
- [RFC 5227] - IPv4 Address Conflict Detection
- [RFC 792] - Internet Control Message Protocol
- [RFC 3376] - Internet Group Management Protocol, Version 3

### IPv6

- [RFC 8200] - Internet Protocol, Version 6 (IPv6) Specification
- [RFC 4291] - IP Version 6 Addressing Architecture
- [RFC 8201] - Path MTU Discovery for IP version 6
- [RFC 4861] - Neighbor Discovery for IP version 6 (IPv6)
- [RFC 4862] - IPv6 Stateless Address Autoconfiguration
- [RFC 4443] - Internet Control Message Protocol (ICMPv6) for the Internet Protocol Version 6 (IPv6)
  Specification.
- [RFC 3810] - Multicast Listener Discovery Version 2 (MLDv2) for IPv6

### UDP

- [RFC 768] - User Datagram Protocol
- [RFC 8085] - UDP Usage Guidelines

### TCP

- [RFC 9293] - Transmission Control Protocol (TCP)
- [RFC 7414] - A Roadmap for Transmission Control Protocol (TCP) Specification Documents

## Runtime

The tygress runtime relies on two core concepts: 

- network interface
- async I/O driver

### Network interface

Network interfaces are built slightly differently, based on HardwareType. Below are a list of
interface state:

- **NeighborCache** - Map of IpAddr to EtherAddr, with expiry. EthernetII hardware only. Size will
  be static, but configurable, and require (expiry-based) garbage collection.
- **Ipv4MulticastGroups** - Set of multicast Ipv4Addr's.
- **IpAddrs** - Set of IpAddr's bound to the interface.
- **IpFragBuffers** - List of buffers used for reassembly of IP fragments. Size will be static, but
  configurable.
- **Routes** - Map of Cidr block to IpAddr, with expiry.
- **SocketSet** - List of open network sockets. Need to determine if this will be an array of enum
  or multiple arrays, one per socket type. Sockets will generally need a pair of TX/RX ring buffers.
  - **ScheduledIO** - List of read and write wakers. Accessible by the async I/O driver. Socket's
    will register themselves with the I/O driver for waking via ScheduledIO.

ARP and NDP serve a similar functionality and will both add entries to a neighbor cache structure.
To save space, it may be desirable to make different caching structures. In particular, IPv6 makes
entries roughly twice as large:

- Timestamp: 8 bytes
- IPv4 address: 4 bytes
- IPv6 address: 16 bytes
- Ethernet address: 6 bytes
- IEEE 802.15.4 address: 8 bytes

However, these tables contain immediate network neighbors, so they shouldn't get too large (?). 


[RFC 768]: https://tools.ietf.org/html/rfc768
[RFC 791]: https://tools.ietf.org/html/rfc791
[RFC 792]: https://tools.ietf.org/html/rfc792
[RFC 826]: https://tools.ietf.org/html/rfc826
[RFC 1122]: https://tools.ietf.org/html/rfc1122
[RFC 1191]: https://tools.ietf.org/html/rfc1191
[RFC 3376]: https://tools.ietf.org/html/rfc3376
[RFC 3810]: https://tools.ietf.org/html/rfc3810
[RFC 4291]: https://tools.ietf.org/html/rfc4291
[RFC 4443]: https://tools.ietf.org/html/rfc4443
[RFC 4861]: https://tools.ietf.org/html/rfc4861
[RFC 4862]: https://tools.ietf.org/html/rfc4862
[RFC 5227]: https://tools.ietf.org/html/rfc5227
[RFC 7414]: https://tools.ietf.org/html/rfc7414
[RFC 8085]: https://tools.ietf.org/html/rfc8085
[RFC 8200]: https://tools.ietf.org/html/rfc8200
[RFC 8201]: https://tools.ietf.org/html/rfc8201
[RFC 9293]: https://tools.ietf.org/html/rfc9293
