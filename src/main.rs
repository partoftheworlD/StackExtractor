use std::{mem, ptr};

use windows_sys::Win32::{
    Foundation::EXCEPTION_SINGLE_STEP,
    System::{
        Diagnostics::Debug::{SetUnhandledExceptionFilter, CONTEXT, EXCEPTION_POINTERS},
        Kernel::{ExceptionContinueExecution, ExceptionContinueSearch},
        RemoteDesktop::WTSEnumerateProcessesA,
    },
};
const CONTEXT_DEBUG_REGISTERS: u32 = 0x10000 | 0x10;

struct Stack;
trait Extractor {
    fn new() -> Self;
    unsafe fn attach(&self, process_name: &str);
    fn set_hw_breakpoint(&self, addr: u64, exception_filter: fn(*const EXCEPTION_POINTERS));
    unsafe extern "system" fn exception_filter(exception_info: *const EXCEPTION_POINTERS) -> i32;
    unsafe extern "system" fn set_veh_breakpoint(&self, addr: u64);
}

impl Extractor for Stack {
    fn new() -> Self {
        Self
    }

    unsafe fn attach(&self, process_name: &str) {
        const WTS_CURRENT_SERVER_HANDLE: isize = 0;
        let mut process_info = ptr::null_mut();
        let mut process_count = 0u32;
        println!("Start WTSEnumerateProcessesA");
        let wts_result = WTSEnumerateProcessesA(
            WTS_CURRENT_SERVER_HANDLE,
            0u32,
            1,
            &mut process_info,
            &mut process_count,
        );
        if wts_result == 0 {
            panic!("WTSEnumerateProcessesA failed");
        } else {
            for idx in 0..=process_count {
                let process_name = (*(process_info).offset(idx as isize)).pProcessName;
                if process_name.is_null() {
                    println!("Found process: {process_name:?}");
                }
                // println!("{:?}", std::str::from_utf8_mut((process_info).offset(idx as isize)).pProcessName);
            }
            println!("{process_count:?}");
        }
    }

    fn set_hw_breakpoint(&self, addr: u64, exception_filter: fn(*const EXCEPTION_POINTERS)) {
        unsafe {
            SetUnhandledExceptionFilter(Some(std::mem::transmute(exception_filter)));
        }
        let mut thread_context: CONTEXT = unsafe { mem::zeroed() };
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
    let stack = Stack::new();
    unsafe {
        stack.attach("notepad.exe");
    }
}
