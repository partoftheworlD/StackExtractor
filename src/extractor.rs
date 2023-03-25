use crate::decoder::Decoder;
use std::{
    ffi::c_void,
    mem::{self, MaybeUninit},
    ops::DerefMut,
    ptr::{self, addr_of, addr_of_mut},
};
use windows_sys::Win32::{
    Foundation::EXCEPTION_SINGLE_STEP,
    System::{
        Diagnostics::Debug::{
            AddrModeFlat, RtlCaptureContext, RtlCaptureStackBackTrace, SetUnhandledExceptionFilter,
            StackWalk64, CONTEXT, EXCEPTION_POINTERS, STACKFRAME64,
        },
        Kernel::{ExceptionContinueExecution, ExceptionContinueSearch},
        RemoteDesktop::WTSEnumerateProcessesA,
        SystemInformation::IMAGE_FILE_MACHINE_AMD64,
        Threading::{GetCurrentProcess, GetCurrentThread},
    },
};

const CONTEXT_DEBUG_REGISTERS: u32 = 0x10000 | 0x10;

pub struct Stack {
    pub pid: u32,
    pub tid: u32,
}
pub trait Extractor {
    fn new() -> Self;
    fn attach(&mut self, process_name: &str);
    fn set_hw_breakpoint(&self, addr: u64, exception_filter: fn(*const EXCEPTION_POINTERS));
    unsafe extern "system" fn exception_filter(exception_info: *const EXCEPTION_POINTERS) -> i32;
    fn set_veh_breakpoint(&self, addr: u64);
    unsafe fn stacktrace(&self, hprocess: isize, hthread: isize);
}

impl Extractor for Stack {
    fn new() -> self::Stack {
        self::Stack { pid: 0, tid: 0 }
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
        if wts_result != 0 && !process_info.is_null() {
            for idx in 1..=process_count {
                let info = unsafe { *process_info.offset((idx).try_into().unwrap()) };
                let pname = Stack::from_lpstr(info.pProcessName);
                if pname.eq_ignore_ascii_case(process_name) {
                    self.pid = info.ProcessId;
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
        let mut thread_context_bind = MaybeUninit::<CONTEXT>::uninit();
        let mut thread_context = thread_context_bind.as_mut_ptr();
        unsafe {
            (*thread_context).ContextFlags = CONTEXT_DEBUG_REGISTERS;
            (*thread_context).Dr0 = addr;
            (*thread_context).Dr7 = 1 << 0;

            thread_context_bind.assume_init();
        }
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

    unsafe fn stacktrace(&self, hprocess: isize, hthread: isize) {
        let mut backtrace = vec![0u64; u16::MAX as usize];
        let pbacktrace = backtrace.as_mut_ptr().cast::<*mut std::ffi::c_void>();
        let num_frames = RtlCaptureStackBackTrace(0, 63, pbacktrace, std::ptr::null_mut::<u32>());
        if num_frames > 0 {
            println!("stack trace");
            backtrace
                .into_iter()
                .take_while(|stack_ptr| *stack_ptr != 0)
                .for_each(|stack_ptr| {
                    println!(" {:#X}", stack_ptr);
                });
        }

        let mut stackframe_bind = MaybeUninit::<STACKFRAME64>::uninit();
        let stackframe = stackframe_bind.assume_init_mut();

        let mut context_bind = MaybeUninit::<CONTEXT>::uninit();
        let context = context_bind.as_mut_ptr();

        RtlCaptureContext(context);
        context_bind.assume_init();

        let rip = (*context).Rip;
        let rsp = (*context).Rsp;
        let rbp = (*context).Rbp;
        println!("Rip: {:#X}", rip);
        println!("Rsp: {:#X}", rsp);
        println!("Rbp: {:#X}", rbp);
        stackframe.AddrPC.Offset = rip;
        stackframe.AddrStack.Offset = rsp;
        stackframe.AddrFrame.Offset = rbp;
        stackframe.AddrPC.Mode = AddrModeFlat;
        stackframe.AddrStack.Mode = AddrModeFlat;
        stackframe.AddrFrame.Mode = AddrModeFlat;

        // TODO Отладить StackWalk64
        
        StackWalk64(
            u32::from(IMAGE_FILE_MACHINE_AMD64),
            hprocess,
            hthread,
            stackframe,
            context.cast::<c_void>(),
            None,
            None,
            None,
            None,
        );

        println!("Number of frames {:?}", num_frames);
        println!("context Rip: {:#X}", (*context).Rip);
        println!("Addr return frame: {:#X}", stackframe.AddrReturn.Offset);
    }
}
