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

use crate::driver::waker::NoopWaker;
use crate::header::checksum::verify_checksum;
use crate::header::error::HeaderTruncated;
use crate::header::internet::{IpVersion, Ipv4};
use crate::header::link::{EtherType, EthernetII, Ieee802154};
use crate::netdev::{Event, HardwareType, NetDev};

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

    fn process(&self, bytes: &[u8]) -> Result<(), HeaderTruncated> {
        match self.netdev.hw_type() {
            HardwareType::EthernetII => {
                let (header, bytes) = EthernetII::from_bytes(bytes)?;
                self.process_ethernet(header, bytes)
            }
            HardwareType::Ieee802154 => {
                let (header, bytes) = Ieee802154::from_bytes(bytes).unwrap();
                self.process_ieee802154(&header, bytes)
            }
            HardwareType::Opaque => match bytes.first().cloned().map(IpVersion::from) {
                Some(IpVersion::Ipv4) => {
                    let (header, bytes) = Ipv4::from_bytes(bytes)?;
                    if verify_checksum(&bytes[..header.header_len()]).is_err() {
                        return Ok(());
                    }
                    self.process_ipv4(&header, bytes)
                }
                Some(IpVersion::Ipv6) => todo!(),
                _ => Ok(()),
            },
        }
    }

    fn process_ethernet(&self, header: &EthernetII, bytes: &[u8]) -> Result<(), HeaderTruncated> {
        match header.ethertype() {
            EtherType::Ipv4 => {
                let (header, bytes) = Ipv4::from_bytes(bytes)?;
                self.process_ipv4(&header, bytes)
            }
            EtherType::Arp => todo!(),
            EtherType::Ipv6 => todo!(),
            _ => Ok(()),
        }
    }

    fn process_ieee802154(
        &self,
        _header: &Ieee802154,
        _bytes: &[u8],
    ) -> Result<(), HeaderTruncated> {
        todo!()
    }

    fn process_ipv4(&self, _header: &Ipv4, _bytes: &[u8]) -> Result<(), HeaderTruncated> {
        todo!()
    }
}
