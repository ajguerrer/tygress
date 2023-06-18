# Tygress (Work in Progress)

A library implementing popular networking transport protocols in user space with Rust.

## Goals

- **Correct, and comprehensive** - The primary goal is create a network stack that works. Rusts
  world-class type system is used at every opportunity to properly capture the constraints of the
  network stack detailed in the [IETF RFCs][rfc]. From the link layer to the transport layer, the
  following transport protocols shall be supported:
    - UDP
    - TCP
    - QUIC
- **Familiar, but `async`** - Rust already has [networking primitives for TCP/UDP][net]. That
  familiar API is reproduced here, but with non-blocking primitives similar to the API in
  [Tokio][tokio]. These primitives are intended for use with the Tygress I/O driver multiplexing
  network traffic between a `NetDev` to a collection of sockets.
- **Safe, but zero-copy** - Some use of the the `unsafe` keyword is necessary to provide a type-safe
  zero-copy API. Usage of `unsafe` is cordoned off to a small, well documented section of the
  library, while the rest is written in safe Rust.
- **No dependencies, and `no_std`** - A network stack all in one place using only the core 
  components of the Rust standard library. Implement the `NetDev` trait for your own network device 
  capable of receiving/transmitting Ethernet frames and BYOB (Bring Your Own Buffers) for the 
  sockets. Those running Unix/BSD may opt-in to some provided `NetDev` implementations.

## Influences

- [Jon Gjengset][jon] - Where this project all started.
- [`smoltcp`][smoltcp] - Used heavily as a reference throughout the project.
- [`tokio`][tokio] - Ideas for `async` networking primitives.
- [`embassy`][embassy] - Ideas for a `no_std` executor.
- [`zerocopy`][zerocopy] - Ideas for zero-copy type-safe headers.

[rfc]: https://www.ietf.org/standards/rfcs
[net]: https://doc.rust-lang.org/std/net
[tokio]: https://docs.rs/tokio/latest/tokio/net
[jon]: https://www.youtube.com/watch?v=bzja9fQWzdA
[smoltcp]: https://docs.rs/smoltcp/latest/smoltcp
[embassy]: https://embassy.dev/embassy/dev/runtime.html
[zerocopy]: https://docs.rs/zerocopy/latest/zerocopy
