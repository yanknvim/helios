#![no_main]
#![no_std]

mod console;
mod trap;

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
    let addr_trap_entry = trap::trap_entry as usize;
    unsafe {
        core::arch::asm!("csrw stvec, {trap_entry}", trap_entry = in(reg) addr_trap_entry);
    }

    println!("Hello, World!");

    loop {}
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    println!("====== KERNEL PANIC! ======");
    println!("{}", info);

    sbi_rt::system_reset(sbi_rt::Shutdown, sbi_rt::NoReason);
    loop {}
}
