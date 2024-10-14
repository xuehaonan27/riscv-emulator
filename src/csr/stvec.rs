use std::ops::{BitAnd, BitOrAssign};

#[repr(transparent)]
pub struct Stvec(u64);

const STVEC_MODE_MASK: u64 = 0b11;
pub const STVEC_MODE_USER: u64 = 0b00;
pub const STVEC_MODE_SUPERVISOR: u64 = 0b01;
pub const STVEC_MODE_MACHINE: u64 = 0b11;

impl Stvec {
    pub fn mode(&self) -> u64 {
        self.0.bitand(STVEC_MODE_MASK)
    }

    pub fn base(&self) -> u64 {
        self.0.bitand(!STVEC_MODE_MASK)
    }

    pub fn mode_set(&mut self, mode: u64) {
        assert!(mode == 0 || mode == 1);
        self.0.bitor_assign(mode);
    }

    pub fn base_set(&mut self, base: u64) {
        assert_eq!(base.bitand(STVEC_MODE_MASK), 0);
        self.0.bitor_assign(base);
    }
}
