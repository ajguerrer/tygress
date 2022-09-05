//! The Tygress async I/O driver.
//!
//! Asynchronous Rust programs require a runtime in order to be useful. Tygress uses a lightweight
//! single-threaded I/O event loop for its runtime. Wrapping a [`NetDev`], [`Driver`] is responsible
//! for calling  [`send`][NetDev::send] and [`recv`][NetDev::recv], multiplexing data from a
//! [`NetDev`] to Tygress's higher level networking primitives.

#[macro_use]
mod pin;
mod waker;

use core::fmt::Debug;
use core::future::Future;
use core::task::{Context, Poll};
use core::{mem, panic};

use crate::driver::waker::NoopWaker;
use crate::header::internet::{IpVersion, Ipv4};
use crate::header::link::{EtherType, EthernetII};
use crate::header::Result;
use crate::netdev::{Event, NetDev, Topology};

#[derive(Debug)]
pub struct Driver<T: NetDev, const MTU: usize>
where
    T::Error: Debug,
{
    netdev: T,
    buffer: [u8; MTU],
}

impl<T: NetDev, const MTU: usize> Driver<T, MTU>
where
    T::Error: Debug,
{
    /// Creates a new asynchronous I/O driver around a [`NetDev`].
    pub fn new(netdev: T) -> Driver<T, MTU> {
        let mtu = netdev.mtu();
        let mtu = match netdev.topology() {
            Topology::Ip => mtu,
            Topology::EthernetII => mtu + mem::size_of::<EthernetII>(),
        };
        if mtu > MTU {
            panic!("netdev requires {} mtu", mtu);
        }
        Driver {
            netdev,
            buffer: [0; MTU],
        }
    }

    /// Runs `future` on the current thread, blocking until it completes, yielding its resolved
    /// result. Main entrypoint for running `async` I/O with Tygress' higher level networking
    /// primitives.
    ///
    /// # Note on blocking
    ///
    /// Just like any `async` runtime, care should be taken to avoid writing blocking code inside
    /// the `future`. Actual I/O only occurs at the `.await` points specified within the `future`.
    pub fn turn<F: Future>(mut self, future: F) -> F::Output {
        pin!(future);
        let waker = NoopWaker::new();
        let mut cx = Context::from_waker(&waker);
        loop {
            if let Poll::Ready(v) = future.as_mut().poll(&mut cx) {
                return v;
            }

            let event = self
                .netdev
                .poll(Event::READABLE, None)
                .expect("netdev poll");
            if event.is_writable() {
                self.dispatch()
            }

            if event.is_readable() {
                let read = self.netdev.recv(&mut self.buffer).expect("netdev recv");
                self.process(&self.buffer[..read]).expect("netdev process");
            }
        }
    }

    fn dispatch(&self) {}

    fn process(&self, bytes: &[u8]) -> Result<()> {
        match self.netdev.topology() {
            Topology::EthernetII => {
                let (header, bytes) = EthernetII::from_bytes(bytes)?;
                self.process_ethernet(header, bytes)
            }
            Topology::Ip => match IpVersion::from_bytes(bytes)? {
                IpVersion::Ipv4 => {
                    let (header, options, bytes) = Ipv4::from_bytes(bytes)?;
                    self.process_ipv4(header, options, bytes)
                }
                IpVersion::Ipv6 => todo!(),
            },
        }
    }

    fn process_ethernet(&self, header: &EthernetII, bytes: &[u8]) -> Result<()> {
        match header.ethertype() {
            EtherType::Ipv4 => {
                let (header, options, bytes) = Ipv4::from_bytes(bytes)?;
                self.process_ipv4(header, options, bytes)
            }
            EtherType::Arp => todo!(),
            EtherType::Ipv6 => todo!(),
        }
    }

    fn process_ipv4(&self, _header: &Ipv4, _options: &[u8], _bytes: &[u8]) -> Result<()> {
        todo!()
    }
}
