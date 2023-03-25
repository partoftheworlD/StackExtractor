pub mod decoder;
pub mod extractor;

use crate::extractor::{Extractor, Stack};
use std::ptr::addr_of_mut;
use std::{arch::asm, mem};
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
        let mut stack = Stack::new();
        stack.attach(r#"notepad.exe"#);
        let mut th32 = Box::new(mem::zeroed::<THREADENTRY32>());
        let ptr_th32 = addr_of_mut!(*th32);
        let mut hthread = 0isize;

        let hprocess = OpenProcess(PROCESS_ALL_ACCESS, 0, stack.pid);
        println!("HProcess {hprocess:?}");
        // TODO: Get hthread
        let hsnap = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD | TH32CS_SNAPPROCESS, 0);
        while Thread32First(hsnap, ptr_th32) == 1 {
            Thread32Next(hsnap, ptr_th32);
            println!("th32 {:#X}", (*ptr_th32).th32ThreadID);
            if (*ptr_th32).th32OwnerProcessID == stack.pid {
                hthread = OpenThread(THREAD_QUERY_INFORMATION, 1, (*ptr_th32).th32ThreadID);
                break;
            }
        }

        if hthread != 0 {
            println!("HThread {hthread:?}");
        } else {
            println!("Can't get HThread");
        }
        stack.stacktrace(hprocess, hthread);
        CloseHandle(hprocess);
        CloseHandle(hthread);
    }
}
