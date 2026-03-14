#![no_main]
#![no_std]

mod shell;

use core::arch::naked_asm;

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

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
