use core::ptr::{read_volatile, write_volatile};

use crate::elf::LoadElfInfo;

/// General purpose register file with machine word = 64 bits.
#[derive(Debug, PartialEq, Eq)]
#[repr(C)]
pub struct RegisterFile {
    zero: u64, // x0  Hard-wired zero
    ra: u64,   // x1  Return address
    sp: u64,   // x2  Stack pointer
    gp: u64,   // x3  Global pointer
    tp: u64,   // x4  Thread pointer
    t0: u64,   // x5  Temporary
    t1: u64,   // x6  Temporary
    t2: u64,   // x7  Temporary
    s0: u64,   // x8  Saved register / Frame pointer
    s1: u64,   // x9  Saved register
    a0: u64,   // x10 Function arguments / Return values
    a1: u64,   // x11 Function arguments / Return values
    a2: u64,   // x12 Function arguments
    a3: u64,   // x13 Function arguments
    a4: u64,   // x14 Function arguments
    a5: u64,   // x15 Function arguments
    a6: u64,   // x16 Function arguments
    a7: u64,   // x17 Function arguments
    s2: u64,   // x18 Saved register
    s3: u64,   // x19 Saved register
    s4: u64,   // x20 Saved register
    s5: u64,   // x21 Saved register
    s6: u64,   // x22 Saved register
    s7: u64,   // x23 Saved register
    s8: u64,   // x24 Saved register
    s9: u64,   // x25 Saved register
    s10: u64,  // x26 Saved register
    s11: u64,  // x27 Saved register
    t3: u64,   // x28 Temporary
    t4: u64,   // x29 Temporary
    t5: u64,   // x30 Temporary
    t6: u64,   // x31 Temporary
}

pub const REGNAME: [&'static str; 32] = [
    "zero", // 0
    "ra",   // 1
    "sp",   // 2
    "gp",   // 3
    "tp",   // 4
    "t0",   // 5
    "t1",   // 6
    "t2",   // 7
    "s0",   // 8
    "s1",   // 9
    "a0",   // 10
    "a1",   // 11
    "a2",   // 12
    "a3",   // 13
    "a4",   // 14
    "a5",   // 15
    "a6",   // 16
    "a7",   // 17
    "s2",   // 18
    "s3",   // 19
    "s4",   // 20
    "s5",   // 21
    "s6",   // 22
    "s7",   // 23
    "s8",   // 24
    "s9",   // 25
    "s10",  // 26
    "s11",  // 27
    "t3",   // 28
    "t4",   // 29
    "t5",   // 30
    "t6",   // 31
];

pub struct ProgramCounter {
    inner: u64,
}

impl ProgramCounter {
    pub fn new() -> ProgramCounter {
        ProgramCounter { inner: 0 }
    }
    /// For now, only simulate 64-bit machine
    pub fn read(&self) -> u64 {
        self.inner
    }

    /// For now, only simulate 64-bit machine
    pub fn write(&mut self, value: u64) {
        self.inner = value;
    }
}

impl RegisterFile {
    /// Get an empty register file
    pub fn empty() -> RegisterFile {
        unsafe { std::mem::zeroed() }
    }

    /// Read from a register
    #[inline]
    pub fn read(&self, reg_index: u8) -> u64 {
        let ptr = self as *const RegisterFile as *const u64;
        // Pointer add safe because of RISC-V ISA 5 bits register index
        let value = unsafe { read_volatile(ptr.add(reg_index.into())) };
        value
    }

    /// Write into a register
    #[inline]
    pub fn write(&mut self, reg_index: u8, value: u64) {
        let ptr = self as *mut RegisterFile as *mut u64;
        // Pointer add safe because of RISC-V ISA 5 bits register index
        unsafe { write_volatile(ptr.add(reg_index.into()), value) };
    }
}

impl RegisterFile {
    /// Initialize register file with ELF info
    pub fn init_elfinfo_64(&mut self, info: &LoadElfInfo) {
        assert!(info.is_64_bit());

        self.zero = 0;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    fn demo_reg_file() -> RegisterFile {
        let mut reg_file = RegisterFile::empty();
        reg_file.zero = 0;
        reg_file.ra = 1;
        reg_file.sp = 2;
        reg_file.gp = 3;
        reg_file.tp = 4;
        reg_file.t0 = 5;
        reg_file.t1 = 6;
        reg_file.t2 = 7;
        reg_file.s0 = 8;
        reg_file.s1 = 9;
        reg_file.a0 = 10;
        reg_file.a1 = 11;
        reg_file.a2 = 12;
        reg_file.a3 = 13;
        reg_file.a4 = 14;
        reg_file.a5 = 15;
        reg_file.a6 = 16;
        reg_file.a7 = 17;
        reg_file.s2 = 18;
        reg_file.s3 = 19;
        reg_file.s4 = 20;
        reg_file.s5 = 21;
        reg_file.s6 = 22;
        reg_file.s7 = 23;
        reg_file.s8 = 24;
        reg_file.s9 = 25;
        reg_file.s10 = 26;
        reg_file.s11 = 27;
        reg_file.t3 = 28;
        reg_file.t4 = 29;
        reg_file.t5 = 30;
        reg_file.t6 = 31;
        reg_file
    }

    #[test]
    fn read_test() {
        let reg_file = demo_reg_file();
        for i in 0..32 {
            assert_eq!(reg_file.read(i), i as u64);
        }
    }

    #[test]
    fn write_test() {
        let reg_file = demo_reg_file();
        let mut empty_reg = RegisterFile::empty();
        for i in 0..32 {
            empty_reg.write(i, i as u64);
        }
        assert_eq!(reg_file, empty_reg);
    }
}
