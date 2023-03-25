use std::{
    arch::asm,
    mem::{self, MaybeUninit},
};

pub mod decoder;
pub mod extractor;
use windows_sys::Win32::{
    Foundation::CloseHandle,
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPPROCESS,
            TH32CS_SNAPTHREAD, THREADENTRY32,
        },
        Threading::{OpenProcess, OpenThread, PROCESS_ALL_ACCESS, THREAD_QUERY_INFORMATION},
    },
};

use crate::extractor::{Extractor, Stack};
// https://en.wikipedia.org/wiki/Win32_Thread_Information_Block
unsafe fn tebx64() {
    let mut teb: u64 = mem::zeroed();
    asm!(
        "mov rax, GS:[0x30]",
        "mov {teb}, rax",
        teb = out(reg) teb,
    );
    println!("teb {teb:#X}");
}

unsafe fn pebx64() {
    let mut peb: u64 = mem::zeroed();
    asm!(
        "mov rax, GS:[0x60]",
        "mov {peb}, rax",
        peb = out(reg) peb,
    );
    println!("Peb {peb:#X}");
}

unsafe fn ldr64() {
    let mut ldr: u64 = mem::zeroed();
    asm!(
        "mov rax, GS:[0x60]",    // get peb
        "mov rax, [rax + 0x18]", // get ldr
        "mov {ldr}, rax",
        ldr = out(reg) ldr,
    );
    println!("Ldr {ldr:#X}");
}

fn main() {
    unsafe {
        tebx64();
        pebx64();
        ldr64();

        let stack = Stack::new();
        // stack.attach(r#"notepad.exe"#);
        let mut th32 = MaybeUninit::<THREADENTRY32>::uninit();
        let p_th32 = th32.as_mut_ptr();
        let hprocess = OpenProcess(PROCESS_ALL_ACCESS, 0, stack.pid);
        let mut hthread = 0isize;
        println!("HProcess {:?}", hprocess);
        // TODO: Get hthread
        let hsnap = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD | TH32CS_SNAPPROCESS, 0);
        while Thread32First(hsnap, p_th32) == 1 {
            Thread32Next(hsnap, p_th32);
            println!("th32 {:#X}", (*p_th32).th32ThreadID);
            if (*p_th32).th32OwnerProcessID == stack.pid {
                hthread = OpenThread(THREAD_QUERY_INFORMATION, 1, (*p_th32).th32ThreadID);
                break;
            }
        }
        th32.assume_init();
        if hthread != 0 {
            println!("HThread {:?}", hthread);
        } else {
            println!("Can't get HThread");
        }
        stack.stacktrace(hprocess, hthread);
        CloseHandle(hprocess);
        CloseHandle(hthread);
    }
}
