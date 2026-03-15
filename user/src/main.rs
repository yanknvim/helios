#![no_main]
#![no_std]

mod shell;

use core::arch::{asm, naked_asm};
use shared::SysCall;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.start")]
#[unsafe(naked)]
pub extern "C" fn start() -> ! {
    naked_asm!("la sp, __stack_top", "call main", "call exit",);
}

#[unsafe(no_mangle)]
pub extern "C" fn exit() -> () {
    loop {}
}

unsafe fn syscall(sysno: usize, arg0: u32, arg1: u32, arg2: u32) -> u32 {
    let mut result: u32;

    unsafe {
        asm!(
            "ecall",
            in("a0") arg0,
            in("a1") arg1,
            in("a2") arg2,
            in("a3") sysno,
            lateout("a0") result,
        );
    }

    result
}

fn putchar(c: char) {
    unsafe { syscall(SysCall::PUTCHAR as usize, c as u32, 0, 0) };
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
