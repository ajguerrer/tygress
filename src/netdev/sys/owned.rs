use std::fmt;
use std::os::unix::prelude::{AsRawFd, FromRawFd, RawFd};

use nix::libc;

// Takes the desired parts of the impl from https://doc.rust-lang.org/std/os/unix/io/struct.OwnedFd.html
// This can be removed once https://github.com/rust-lang/rust/issues/87074 is stable.
//
// An owned file descriptor.
//
// This closes the file descriptor on drop.
//
// This uses `repr(transparent)` and has the representation of a host file
// descriptor, so it can be used in FFI in places where a file descriptor is
// passed as a consumed argument or returned as an owned value, and it never
// has the value `-1`.
#[repr(transparent)]
pub(crate) struct OwnedFd {
    fd: RawFd,
}

impl AsRawFd for OwnedFd {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl FromRawFd for OwnedFd {
    /// Constructs a new instance of `Self` from the given raw file descriptor.
    ///
    /// # Safety
    ///
    /// The resource pointed to by `fd` must be open and suitable for assuming
    /// ownership. The resource must not require any cleanup other than `close`.
    #[inline]
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        assert_ne!(fd, u32::MAX as RawFd);
        // SAFETY: we just asserted that the value is in the valid range and isn't `-1` (the only value bigger than `0xFF_FF_FF_FE` unsigned)
        Self { fd }
    }
}

impl fmt::Debug for OwnedFd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OwnedFd").field("fd", &self.fd).finish()
    }
}

impl Drop for OwnedFd {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            // Note that errors are ignored when closing a file descriptor. The
            // reason for this is that if an error occurs we don't actually know if
            // the file descriptor was closed or not, and if we retried (for
            // something like EINTR), we might close another valid file descriptor
            // opened after we closed ours.
            let _ = libc::close(self.fd);
        }
    }
}
