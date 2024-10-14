use std::ops::{BitAnd, BitOrAssign};

/// M mode trap vector base address.
/// XLEN bit read-write register.
/// `base`: base address of trap vector. Aligend to 4 bytes.
/// `mode`: vector mode.
/// mode=0: set pc to `base` under any exception.
/// mode=1: set pc to `base + (4 * cause)` under asynchronous exception.
#[repr(transparent)]
pub struct Mtvec(u64);

const MTVEC_MODE_MASK: u64 = 0b11;

impl Mtvec {
    pub fn mode(&self) -> u64 {
        self.0.bitand(MTVEC_MODE_MASK)
    }

    pub fn base(&self) -> u64 {
        self.0.bitand(!MTVEC_MODE_MASK)
    }

    pub fn mode_set(&mut self, mode: u64) {
        assert!(mode == 0 || mode == 1);
        self.0.bitor_assign(mode);
    }

    pub fn base_set(&mut self, base: u64) {
        assert_eq!(base.bitand(MTVEC_MODE_MASK), 0);
        self.0.bitor_assign(base);
    }

    pub fn read(&self) -> u64 {
        self.0
    }

    pub fn write(&mut self, val: u64) {
        self.0 = val;
    }
}
