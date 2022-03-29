//! The Tygress async I/O driver.
//!
//! Asynchronous Rust programs require a runtime in order to be useful. Tygress uses a lightweight
//! single-threaded I/O event loop for its runtime. Wrapping a [`NetDev`], [`Driver`] is responsible
//! for calling [`recv`][NetDev::recv] and [`send`][NetDev::send], multiplexing data from the
//! [`NetDev`] to Tygress's higher level networking primitives..

#[macro_use]
mod pin;
mod waker;

use core::fmt::Debug;
use core::future::Future;
use core::task::{Context, Poll};

use crate::driver::waker::NoopWaker;
use crate::netdev::NetDev;

#[derive(Debug)]
pub struct Driver<T: NetDev> {
    netdev: T,
}

impl<T: NetDev> Driver<T> {
    /// Creates a new asynchronous I/O driver around a [`NetDev`].
    pub fn new(netdev: T) -> Self {
        Driver { netdev }
    }

    /// Runs `future` on the current thread, blocking until it completes, yielding its resolved
    /// result. Main entrypoint for running `async` I/O with Tygress's higher level networking
    /// primitives.
    ///
    /// # Note on blocking
    ///
    /// Just like any `async` runtime, care should be taken to avoid writing blocking code inside
    /// the `future`. Actual I/O only occurs at the specified `.await` points within the `future`.
    pub fn turn<F: Future>(self, future: F) -> F::Output
    where
        T::Error: Debug,
    {
        pin!(future);
        let waker = NoopWaker::new();
        let mut cx = Context::from_waker(&waker);
        loop {
            if let Poll::Ready(v) = future.as_mut().poll(&mut cx) {
                return v;
            }

            let event = self.netdev.poll(None).expect("failed to poll NetDev");
            if event.is_writable() {
                self.dispatch()
            }

            if event.is_readable() {
                self.process()
            }
        }
    }

    fn dispatch(&self) {}

    fn process(&self) {}
}
