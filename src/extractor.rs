use std::{mem, ptr};
use crate::decoder::Decoder;
use windows_sys::Win32::{
    Foundation::EXCEPTION_SINGLE_STEP,
    System::{
        Diagnostics::Debug::{SetUnhandledExceptionFilter, CONTEXT, EXCEPTION_POINTERS},
        Kernel::{ExceptionContinueExecution, ExceptionContinueSearch},
        RemoteDesktop::WTSEnumerateProcessesA,
    },
};

const CONTEXT_DEBUG_REGISTERS: u32 = 0x10000 | 0x10;

pub struct Stack {
    pub pid: u32,
}

pub trait Extractor {
    fn new() -> Self;
    fn attach(&mut self, process_name: &str);
    fn set_hw_breakpoint(&self, addr: u64, exception_filter: fn(*const EXCEPTION_POINTERS));
    unsafe extern "system" fn exception_filter(exception_info: *const EXCEPTION_POINTERS) -> i32;
    fn set_veh_breakpoint(&self, addr: u64);
}

impl Extractor for Stack {
    fn new() -> self::Stack {
        self::Stack { pid: 0 }
    }
    fn attach(&mut self, process_name: &str) {
        const WTS_CURRENT_SERVER_HANDLE: isize = 0;
        let mut process_info = ptr::null_mut();
        let mut process_count = 0u32;
        let wts_result = unsafe {
            WTSEnumerateProcessesA(
                WTS_CURRENT_SERVER_HANDLE,
                0u32,
                1,
                &mut process_info,
                &mut process_count,
            )
        };
        if wts_result != 0 {
            for idx in 1..process_count {
                let info = unsafe { *process_info.offset((idx).try_into().unwrap()) };
                let pname = Stack::from_lpstr(info.pProcessName);
                let pid = info.ProcessId;
                if pname.eq_ignore_ascii_case(process_name) {
                    self.pid = pid;
                    break;
                }
            }
            assert!((self.pid != 0), "Process {process_name} not found");
            println!("{process_count:?}");
        } else {
            panic!("WTSEnumerateProcessesA failed");
        }
    }

    fn set_hw_breakpoint(&self, addr: u64, exception_filter: fn(*const EXCEPTION_POINTERS)) {
        unsafe {
            SetUnhandledExceptionFilter(Some(mem::transmute(exception_filter)));
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
    fn set_veh_breakpoint(&self, addr: u64) {
        todo!();
    }
}