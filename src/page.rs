use bitflags::bitflags;
use core::ptr;

extern crate alloc;
use alloc::alloc::{Layout, alloc, dealloc};

const PAGE_SIZE: u32 = 4 * 1024;

unsafe extern "C" {
    static _stext: u8;
    static _eheap: u8;
}

bitflags! {
    #[derive(Clone, Copy)]
    pub struct PteFlags: u32 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[repr(align(4096))]
pub struct PageTable {
    table: [u32; 1024],
}

pub fn map_page(table: *mut PageTable, vaddr: *const u32, paddr: *const u32, flags: PteFlags) {
    if !vaddr.is_aligned_to(PAGE_SIZE as usize) {
        panic!("unaligned vaddr {:?}", vaddr);
    }

    if !paddr.is_aligned_to(PAGE_SIZE as usize) {
        panic!("unaligned paddr {:?}", paddr);
    }

    let vpn1 = (vaddr as usize >> 22) & 0x3ff;
    let vpn0 = (vaddr as usize >> 12) & 0x3ff;

    let table1 = unsafe { &mut *table };
    if (table1.table[vpn1] & PteFlags::V.bits() as u32) == 0 {
        let layout = Layout::from_size_align(4096, 4096).expect("Failed to allocate");
        let ptr = unsafe { alloc(layout) };
        table1.table[vpn1] = ((ptr as u32 / PAGE_SIZE) << 10) | PteFlags::V.bits();
    }

    let table0_paddr = (table1.table[vpn1] >> 10) << 12;
    let table0 = unsafe { &mut *(table0_paddr as *mut PageTable) };
    table0.table[vpn0] = ((paddr as u32 / PAGE_SIZE) << 10) | flags.bits() | PteFlags::V.bits();
}

pub fn init_page(table: *mut PageTable) {
    let start = unsafe { ptr::addr_of!(_stext) as usize };
    let end = unsafe { ptr::addr_of!(_eheap) as usize };

    for addr in (start..end).step_by(PAGE_SIZE as usize) {
        map_page(
            table,
            addr as *const u32,
            addr as *const u32,
            PteFlags::R | PteFlags::W | PteFlags::X,
        );
    }
}
