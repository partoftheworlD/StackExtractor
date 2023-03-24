use std::{arch::asm, mem};

pub mod extractor;
pub mod decoder;
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
    }
    let mut stack = Stack::new();
    stack.attach(r#"notepad.exe"#);
}
