use std::ops::BitAnd;

#[repr(transparent)]
pub struct Mie(u64);

const MIP_SSIE_MASK: u64 = 1 << 1;
const MIP_MSIE_MASK: u64 = 1 << 3;
const MIP_STIE_MASK: u64 = 1 << 5;
const MIP_MTIE_MASK: u64 = 1 << 7;
const MIP_SEIE_MASK: u64 = 1 << 9;
const MIP_MEIE_MASK: u64 = 1 << 11;

impl Mie {
    pub fn ssie(&self) -> u64 {
        self.0.bitand(MIP_SSIE_MASK)
    }

    pub fn msie(&self) -> u64 {
        self.0.bitand(MIP_MSIE_MASK)
    }

    pub fn stie(&self) -> u64 {
        self.0.bitand(MIP_STIE_MASK)
    }

    pub fn mtie(&self) -> u64 {
        self.0.bitand(MIP_MTIE_MASK)
    }

    pub fn seie(&self) -> u64 {
        self.0.bitand(MIP_SEIE_MASK)
    }

    pub fn meie(&self) -> u64 {
        self.0.bitand(MIP_MEIE_MASK)
    }

    pub fn read(&self) -> u64 {
        self.0
    }

    pub fn write(&mut self, val: u64) {
        self.0 = val;
    }
}
