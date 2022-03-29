#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![deny(unsafe_code)]

//! A library that implements several common Internet protocol suites in user-space.
//!
//! Tygress is a learning project and not meant for use in production. While the primary goal is
//! correctness, operating systems and [The Rust Standard Library][std] offer far more robust
//! [networking primitives][net]. Use those instead. If you are interested in a embedded library,
//! take a look at [smoltcp]. It was used heavily as a reference.
//!
//! A secondary goal of Tygress is `#![no_std]`. Everything is BYOB (Bring Your Own Buffers). The
//! sole exception to this rule, are a couple [NetDev][`netdev::NetDev`] implementations since they
//! rely on `#[cfg(unix)]`. These types are opt-in by enabling the `netdev` feature.
//!
//! [std]: https://doc.rust-lang.org/std
//! [net]: https://doc.rust-lang.org/std/net/
//! [smoltcp]: https://docs.rs/smoltcp/latest/smoltcp/

pub mod driver;
pub mod error;
pub mod header;
pub mod netdev;
