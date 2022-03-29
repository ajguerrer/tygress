#![allow(unsafe_code)]

use core::ops::Deref;
use core::ptr;
use core::task::{RawWaker, RawWakerVTable, Waker};

#[derive(Debug)]
pub(crate) struct NoopWaker {
    inner: Waker,
}

impl NoopWaker {
    pub(crate) fn new() -> Self {
        NoopWaker {
            inner: unsafe { Waker::from_raw(RawWaker::new(ptr::null(), &NOOP_WAKER_VTABLE)) },
        }
    }
}

impl Deref for NoopWaker {
    type Target = Waker;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

const NOOP_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(noop_clone, noop, noop, noop);

unsafe fn noop_clone(_: *const ()) -> RawWaker {
    RawWaker::new(ptr::null(), &NOOP_WAKER_VTABLE)
}

unsafe fn noop(_: *const ()) {}
