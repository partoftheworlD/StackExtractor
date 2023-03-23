use std::fmt::Error;

use windows_sys::Win32::{
    Foundation::{EXCEPTION_SINGLE_STEP, HANDLE},
    System::{
        Diagnostics::Debug::{SetUnhandledExceptionFilter, CONTEXT, EXCEPTION_POINTERS},
        Kernel::{ExceptionContinueExecution, ExceptionContinueSearch},
    },
};
const CONTEXT_DEBUG_REGISTERS: u32 = 0x10000 | 0x10;

struct Stack;
trait Extractor {
    fn new() -> Self;
    fn attach(&self, process_name: &str) -> Result<HANDLE, Error>;
    fn set_hw_breakpoint(&self, addr: u64, exception_filter: &fn(*const EXCEPTION_POINTERS));
    unsafe extern "system" fn exception_filter(exception_info: *const EXCEPTION_POINTERS) -> i32;
    unsafe extern "system" fn set_veh_breakpoint(&self, addr: u64);
}

impl Extractor for Stack {
    fn new() -> Self {
        Self
    }

    fn attach(&self, process_name: &str) -> Result<HANDLE, Error> {
        todo!()
    }

    fn set_hw_breakpoint(&self, addr: u64, exception_filter: &fn(*const EXCEPTION_POINTERS)) {
        unsafe {
            SetUnhandledExceptionFilter(Some(std::mem::transmute(exception_filter)));
        }
        let mut thread_context: CONTEXT = unsafe { std::mem::zeroed() };
        thread_context.ContextFlags = CONTEXT_DEBUG_REGISTERS;
        thread_context.Dr0 = addr;
        thread_context.Dr7 = 1 << 0;
    }

    unsafe extern "system" fn exception_filter(exception_info: *const EXCEPTION_POINTERS) -> i32 {
        if (*(*exception_info).ExceptionRecord).ExceptionCode == EXCEPTION_SINGLE_STEP {
            let debug_context = (*exception_info).ContextRecord;
            todo!();
            return ExceptionContinueExecution;
        }
        ExceptionContinueSearch
    }
    unsafe extern "system" fn set_veh_breakpoint(&self, addr: u64) {
        todo!();
    }
}

fn main() {
    let mut stack = Stack::new();
}
