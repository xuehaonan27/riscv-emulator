use std::ops::BitAnd;

use super::XLEN;

#[repr(transparent)]
pub struct Sstatus(u64);

const SSTATUS_SIE_MASK: u64 = 0b1 << 1;
const SSTATUS_MIE_MASK: u64 = 0b1 << 3;
const SSTATUS_SPIE_MASK: u64 = 0b1 << 5;
const SSTATUS_MPIE_MASK: u64 = 0b1 << 7;
const SSTATUS_SPP_MASK: u64 = 0b1 << 8;
const SSTATUS_MPP_MASK: u64 = 0b11 << 11;
const SSTATUS_FS_MASK: u64 = 0b11 << 13;
const SSTATUS_XS_MASK: u64 = 0b11 << 15;
const SSTATUS_MPRV_MASK: u64 = 0b1 << 17;
const SSTATUS_SUM_MASK: u64 = 0b1 << 18;
const SSTATUS_MXR_MASK: u64 = 0b1 << 19;
const SSTATUS_TVM_MASK: u64 = 0b1 << 20;
const SSTATUS_TW_MASK: u64 = 0b1 << 21;
const SSTATUS_TSR_MASK: u64 = 0b1 << 22;
const SSTATUS_SD_MASK: u64 = 0b1 << (XLEN - 1);

impl Sstatus {
    pub fn sie(&self) -> u64 {
        self.0.bitand(SSTATUS_SIE_MASK)
    }

    pub fn mie(&self) -> u64 {
        self.0.bitand(SSTATUS_MIE_MASK)
    }

    /// Original value of sie.
    pub fn spie(&self) -> u64 {
        self.0.bitand(SSTATUS_SPIE_MASK)
    }

    /// Original value of mie.
    pub fn mpie(&self) -> u64 {
        self.0.bitand(SSTATUS_MPIE_MASK)
    }

    pub fn spp(&self) -> u64 {
        self.0.bitand(SSTATUS_SPP_MASK)
    }

    pub fn mpp(&self) -> u64 {
        self.0.bitand(SSTATUS_MPP_MASK)
    }

    pub fn fs(&self) -> u64 {
        self.0.bitand(SSTATUS_FS_MASK)
    }

    pub fn xs(&self) -> u64 {
        self.0.bitand(SSTATUS_XS_MASK)
    }

    pub fn mprv(&self) -> u64 {
        self.0.bitand(SSTATUS_MPRV_MASK)
    }

    pub fn sum(&self) -> u64 {
        self.0.bitand(SSTATUS_SUM_MASK)
    }

    pub fn mxr(&self) -> u64 {
        self.0.bitand(SSTATUS_MXR_MASK)
    }

    pub fn tvm(&self) -> u64 {
        self.0.bitand(SSTATUS_TVM_MASK)
    }

    pub fn tw(&self) -> u64 {
        self.0.bitand(SSTATUS_TW_MASK)
    }

    pub fn tsr(&self) -> u64 {
        self.0.bitand(SSTATUS_TSR_MASK)
    }

    pub fn read(&self) -> u64 {
        self.0
    }

    pub fn write(&mut self, val: u64) {
        self.0 = val;
    }

    // pub fn sie_set(&mut self, val: bool) {
    //     let bit = if val { SSTATUS_SIE_MASK } else { 0 };
    //     self.0 = self.0.bitand(!SSTATUS_SIE_MASK).bitor(bit);
    // }

    // pub fn mie_set(&mut self, val: bool) {
    //     let bit = if val { SSTATUS_MIE_MASK } else { 0 };
    //     self.0 = self.0.bitand(!SSTATUS_MIE_MASK).bitor(bit);
    // }
}
