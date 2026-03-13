#![no_main]
#![no_std]

mod console;

use core::arch::naked_asm;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
#[unsafe(naked)]
pub extern "C" fn boot() -> ! {
    naked_asm!(
        "la sp, __stack_top\n
        j {kernel_main}\n",
        kernel_main = sym kernel_main,
    );
}

#[unsafe(no_mangle)]
pub fn kernel_main() -> ! {
    println!("Hello, World!");

    loop {}
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
