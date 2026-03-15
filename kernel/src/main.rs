#![no_main]
#![no_std]
#![feature(pointer_is_aligned_to)]

mod allocator;
mod console;
mod page;
mod process;
mod trap;

use core::arch::naked_asm;
use core::ptr;

use crate::process::{ProcessManager, ProcessState};

static mut PROCESS_MANAGER: ProcessManager = ProcessManager::new();
const USER_BIN: &[u8] = include_bytes!("../../target/user.bin");

unsafe extern "C" {
    static _sheap: u8;
    static _eheap: u8;
    static __bss: u8;
    static __bss_end: u8;
}

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
    unsafe {
        core::ptr::write_bytes(__bss as *mut u8, 0, __bss_end as usize - __bss as usize);
    }

    let addr_trap_entry = trap::trap_entry as *const usize;
    unsafe {
        core::arch::asm!("csrw stvec, {trap_entry}", trap_entry = in(reg) addr_trap_entry);
    }

    unsafe {
        let start = core::ptr::addr_of!(_sheap) as usize;
        let end = core::ptr::addr_of!(_eheap) as usize;
        let size = end - start;

        crate::allocator::ALLOCATOR
            .lock()
            .init(start as *mut u8, size);
    }
    println!("Allocation Ready");

    extern crate alloc;

    let pm = unsafe { &mut *core::ptr::addr_of_mut!(PROCESS_MANAGER) };

    // init
    pm.create_process(ptr::null_mut(), 0);
    pm.procs[0].state = ProcessState::Runnable;
    pm.procs[0].pid = 0;
    pm.current = 0;

    pm.create_process(
        USER_BIN.as_ptr() as *const u32,
        core::mem::size_of_val(USER_BIN),
    );

    println!("Process Ready");

    pm.schedule();
    panic!("IDLE");
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    println!("====== KERNEL PANIC! ======");
    println!("{}", info);

    sbi_rt::system_reset(sbi_rt::Shutdown, sbi_rt::NoReason);
    loop {}
}
