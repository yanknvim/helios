#![no_main]
#![no_std]
#![feature(pointer_is_aligned_to)]

mod allocator;
mod console;
mod page;
mod process;
mod trap;

use core::alloc::Layout;
use core::arch::naked_asm;

use crate::page::{PageTable, init_page};
use crate::process::{ProcessManager, ProcessState};

static mut PROCESS_MANAGER: ProcessManager = ProcessManager::new();

unsafe extern "C" {
    static _sheap: u8;
    static _eheap: u8;
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

    let layout = Layout::from_size_align(4096, 4096).unwrap();
    let root_table = unsafe { alloc::alloc::alloc_zeroed(layout) as *mut PageTable };
    init_page(root_table);

    println!("Page Ready");

    let pm = unsafe { &mut *core::ptr::addr_of_mut!(PROCESS_MANAGER) };
    pm.procs[0].table = root_table;
    pm.procs[0].state = ProcessState::Runnable;
    pm.current = 0;

    pm.create_process(proc1 as u32);
    pm.create_process(proc2 as u32);

    println!("Process Ready");

    loop {
        pm.schedule();
    }
}

fn proc1() -> ! {
    loop {
        println!("A");
        unsafe {
            let pm = &mut *core::ptr::addr_of_mut!(PROCESS_MANAGER);
            pm.schedule();
        }
    }
}

fn proc2() -> ! {
    loop {
        println!("B");
        unsafe {
            let pm = &mut *core::ptr::addr_of_mut!(PROCESS_MANAGER);
            pm.schedule();
        }
    }
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    println!("====== KERNEL PANIC! ======");
    println!("{}", info);

    sbi_rt::system_reset(sbi_rt::Shutdown, sbi_rt::NoReason);
    loop {}
}
