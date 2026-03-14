use crate::page::{PageTable, init_page};
extern crate alloc;
use alloc::alloc::{Layout, alloc};

const PROCS_MAX: usize = 8;

pub struct ProcessManager {
    pub procs: [Process; PROCS_MAX],
    pub current: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Process {
    pub pid: usize,
    pub state: ProcessState,
    pub context: Context,
    pub stack: [u8; 8192],
    pub table: *mut PageTable,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProcessState {
    Unused,
    Idle,
    Runnable,
}

impl Process {
    pub const fn new(pid: usize) -> Self {
        Self {
            pid,
            state: ProcessState::Unused,
            context: Context::new(),
            stack: [0u8; 8192],
            table: core::ptr::null_mut(),
        }
    }
}

impl ProcessManager {
    pub const fn new() -> Self {
        Self {
            procs: [Process::new(0); PROCS_MAX],
            current: 0,
        }
    }

    pub fn create_process(&mut self, pc: u32) {
        if let Some((i, proc)) = self
            .procs
            .iter_mut()
            .enumerate()
            .find(|(_, p)| matches!(p.state, ProcessState::Unused))
        {
            let layout = Layout::from_size_align(4096, 4096).unwrap();
            let table_ptr = unsafe { alloc(layout) as *mut PageTable };

            init_page(table_ptr);

            proc.table = table_ptr;

            proc.pid = i + 1;
            proc.state = ProcessState::Runnable;
            proc.context = Context::new();
            proc.context.ra = pc;

            let stack_top = proc.stack.as_ptr() as usize + core::mem::size_of_val(&proc.stack);
            proc.context.sp = stack_top as u32;
        } else {
            panic!("No free process");
        }
    }

    pub fn schedule(&mut self) {
        let prev_index = self.current;
        let mut next_index = 0;

        for i in 1..PROCS_MAX {
            let index = (prev_index + i) % PROCS_MAX;
            if self.procs[index].state == ProcessState::Runnable {
                next_index = index;
                break;
            }
        }

        if next_index == prev_index {
            return;
        }

        self.current = next_index;

        let prev = &mut self.procs[prev_index].context as *mut Context;
        let next = &mut self.procs[next_index].context as *mut Context;

        let next_table = self.procs[next_index].table as u32;
        let satp = (1 << 31) | (next_table >> 12);

        unsafe {
            core::arch::asm!(
                "csrw satp, {satp}",
                "sfence.vma",
                satp = in(reg) satp,
            );
            switch(prev, next);
        }
    }
}

#[repr(C)]
#[derive(Copy, PartialEq, Eq, Clone, Debug)]
pub struct Context {
    pub ra: u32,
    pub s0: u32,
    pub s1: u32,
    pub s2: u32,
    pub s3: u32,
    pub s4: u32,
    pub s5: u32,
    pub s6: u32,
    pub s7: u32,
    pub s8: u32,
    pub s9: u32,
    pub s10: u32,
    pub s11: u32,
    pub sp: u32,
}

impl Context {
    pub const fn new() -> Self {
        Self {
            ra: 0,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
            sp: 0,
        }
    }
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub extern "C" fn switch(prev_context: *mut Context, next_context: *mut Context) {
    core::arch::naked_asm!(
        "sw ra, 0 * 4(a0)",
        "sw s0, 1 * 4(a0)",
        "sw s1, 2 * 4(a0)",
        "sw s2, 3 * 4(a0)",
        "sw s3, 4 * 4(a0)",
        "sw s4, 5 * 4(a0)",
        "sw s5, 6 * 4(a0)",
        "sw s6, 7 * 4(a0)",
        "sw s7, 8 * 4(a0)",
        "sw s8, 9 * 4(a0)",
        "sw s9, 10 * 4(a0)",
        "sw s10, 11 * 4(a0)",
        "sw s11, 12 * 4(a0)",
        "sw sp, 13 * 4(a0)",
        "lw ra, 0 * 4(a1)",
        "lw s0, 1 * 4(a1)",
        "lw s1, 2 * 4(a1)",
        "lw s2, 3 * 4(a1)",
        "lw s3, 4 * 4(a1)",
        "lw s4, 5 * 4(a1)",
        "lw s5, 6 * 4(a1)",
        "lw s6, 7 * 4(a1)",
        "lw s7, 8 * 4(a1)",
        "lw s8, 9 * 4(a1)",
        "lw s9, 10 * 4(a1)",
        "lw s10, 11 * 4(a1)",
        "lw s11, 12 * 4(a1)",
        "lw sp, 13 * 4(a1)",
        "ret",
    );
}
