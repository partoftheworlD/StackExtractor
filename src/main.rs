use std::fmt::Error;

use windows_sys::Win32::{Foundation::{HANDLE, EXCEPTION_SINGLE_STEP}, System::{Diagnostics::Debug::{EXCEPTION_POINTERS, CONTEXT}, Kernel::{ExceptionContinueExecution, ExceptionContinueSearch}}};

struct Stack;
trait Extractor {
    fn new() -> Self;
    fn attach(&self, process_name: &str) -> Result<HANDLE, Error>;
    fn set_breakpoint(&self, addr: u64);
    unsafe extern "system" fn exception_filter(exception_info: *mut EXCEPTION_POINTERS) -> i32;
}

impl Extractor for Stack {
    fn new() -> Self {
        Self
    }

    fn attach(&self, process_name: &str) -> Result<HANDLE, Error> {
        todo!()
    }

    fn set_breakpoint(&self, addr: u64) {
        todo!()
    }

    unsafe extern "system" fn exception_filter(exception_info: *mut EXCEPTION_POINTERS) -> i32 {
        if (*(*exception_info).ExceptionRecord).ExceptionCode == EXCEPTION_SINGLE_STEP {
            let debug_context= (*exception_info).ContextRecord;
            todo!();
            return ExceptionContinueExecution;
        }
        ExceptionContinueSearch
    }
}



fn main() {
    println!("Hello, world!");
}
