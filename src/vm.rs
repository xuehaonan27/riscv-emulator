use std::ptr::{read_volatile, write_volatile};

use crate::elf::LoadElfInfo;

const PROTECT_SIZE: usize = 1 * 1024 * 1024; // 1 MiB, for separation of stack
const STACK_SIZE: usize = 8 * 1024 * 1024; // 8 MiB, for the stack

/// For now, we view virtual memory as a continuous bytes array.
#[derive(Debug)]
pub struct VirtualMemory {
    ld_start: usize, // vaddr where the code starts
    mm: Vec<u8>,
}

impl VirtualMemory {
    pub fn new(size: usize) -> VirtualMemory {
        let mut mm = Vec::with_capacity(size);
        mm.resize(size, 0);
        VirtualMemory {
            ld_start: 0, // default to 0, but usually not what the case is.
            mm,
        }
    }

    #[allow(unused)]
    pub fn clear(&mut self) {
        self.mm.clear();
    }

    pub fn from_elf_info(info: &LoadElfInfo) -> VirtualMemory {
        let prog_size = (info.max_vaddr() - info.min_vaddr()) as usize;

        let tot_size = prog_size + PROTECT_SIZE + STACK_SIZE;

        let mut vm = VirtualMemory::new(tot_size);
        vm.ld_start = info.min_vaddr();
        // debug!("vm.ld_start = {:#x}", vm.ld_start);

        for (vm_range, file_range) in std::iter::zip(info.vm_ranges(), info.file_ranges()) {
            // copy all bytes into the virtual memory
            let mut load_range = vm_range.clone();
            let load_length = file_range.end - file_range.start;
            load_range.start -= vm.ld_start;
            // load_range.end -= vm.ld_start;
            load_range.end = load_range.start + load_length;
            // debug!("load {:#x?} from {:#x?}", load_range, file_range);
            vm.mm[load_range].copy_from_slice(&info.raw_data()[file_range.clone()]);
        }

        vm
    }

    /// Read a value from a position
    #[inline(always)]
    fn host_read<T: Sized>(&self, pos: usize) -> T {
        // a raw pointer to the vector's buffer
        let mem_0 = self.mm.as_ptr();
        unsafe { read_volatile(mem_0.add(pos) as *const T) }
    }

    /// Read a value from a virtual memory address.
    #[inline(always)]
    pub fn mread<T: Sized>(&self, vaddr: usize) -> T {
        self.host_read(vaddr - self.ld_start)
    }

    /// Write a value into a position
    #[inline(always)]
    fn host_write<T: Sized>(&mut self, pos: usize, value: T) {
        // an unsafe mutable pointer to the vector's buffer
        let mem_0 = self.mm.as_mut_ptr();
        unsafe { write_volatile(mem_0.add(pos) as *mut T, value) };
    }

    /// Write a value into a virtual memory address.
    #[inline(always)]
    pub fn mwrite<T: Sized>(&mut self, vaddr: usize, value: T) {
        self.host_write(vaddr - self.ld_start, value);
    }
}
