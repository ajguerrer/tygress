use core::marker::PhantomData;

use libc::ifreq;
use rustix::ioctl::{CompileTimeOpcode, Ioctl, IoctlOutput};

pub struct IfreqGetter<'a, Opcode> {
    ifreq: &'a mut ifreq,
    _opcode: PhantomData<Opcode>,
}

impl<'a, Opcode: CompileTimeOpcode> IfreqGetter<'a, Opcode> {
    pub fn new(ifreq: &'a mut ifreq) -> Self {
        Self {
            ifreq,
            _opcode: PhantomData,
        }
    }
}

unsafe impl<'a, Opcode: CompileTimeOpcode> Ioctl for IfreqGetter<'a, Opcode> {
    type Output = ();

    const OPCODE: rustix::ioctl::Opcode = Opcode::OPCODE;
    const IS_MUTATING: bool = true;

    fn as_ptr(&mut self) -> *mut libc::c_void {
        (self.ifreq as *mut ifreq).cast()
    }

    unsafe fn output_from_ptr(
        _out: IoctlOutput,
        _extract_output: *mut libc::c_void,
    ) -> rustix::io::Result<Self::Output> {
        Ok(())
    }
}
