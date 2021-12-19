# Tygress


A library implementing popular networking protocols in user space with Rust.

## Goals

- **Correctness** - The primary goal is to internalize essential [IETF RFCs][rfc], and there is
  no better way to do that than to implement them in source code. Every effort is made to implement 
  them correctly.
- **Support** - Deliver a working network stack from the link layer to the transport layer with the 
  following common transport protocols:
    - UDP
    - TCP
    - QUIC
- **Familiarity** - Rust already has [networking primitives for TCP/UDP][net]. To improve 
  familiarity, a similar API is reproduced here.
- **`async`** - Networking primitives are non-blocking, or `async`, similar to the ones in 
  [Tokio][tokio]. These primitives are intended for use with a provided executor which runs on top 
  of a single `NetDev`.
- **`no_std`** - Implement the `NetDev` trait for your own network device capable of 
  receiving/transmitting Ethernet frames and BYOB (Bring Your Own Buffers) for the sockets. Enable 
  the `netdev` feature to use `NetDev` implementations available to Unix/BSD.

[rfc]: https://www.ietf.org/standards/rfcs
[net]: https://doc.rust-lang.org/std/net
[tokio]: https://docs.rs/tokio/latest/tokio/net