use std::ops::BitAnd;

#[repr(transparent)]
pub struct Mip(u64);

const MIP_SSIP_MASK: u64 = 1 << 1;
const MIP_MSIP_MASK: u64 = 1 << 3;
const MIP_STIP_MASK: u64 = 1 << 5;
const MIP_MTIP_MASK: u64 = 1 << 7;
const MIP_SEIP_MASK: u64 = 1 << 9;
const MIP_MEIP_MASK: u64 = 1 << 11;

impl Mip {
    pub fn ssip(&self) -> u64 {
        self.0.bitand(MIP_SSIP_MASK)
    }

    pub fn msip(&self) -> u64 {
        self.0.bitand(MIP_MSIP_MASK)
    }

    pub fn stip(&self) -> u64 {
        self.0.bitand(MIP_STIP_MASK)
    }

    pub fn mtip(&self) -> u64 {
        self.0.bitand(MIP_MTIP_MASK)
    }

    pub fn seip(&self) -> u64 {
        self.0.bitand(MIP_SEIP_MASK)
    }

    pub fn meip(&self) -> u64 {
        self.0.bitand(MIP_MEIP_MASK)
    }

    pub fn read(&self) -> u64 {
        self.0
    }

    fn _write(&mut self, val: u64) {
        self.0 = val;
    }

    pub fn write(&mut self, val: u64) {
        // Only SSIP, STIP and SEIP could be written by CSR instructions.
        let val = val.bitand(MIP_SSIP_MASK | MIP_STIP_MASK | MIP_SEIP_MASK);
        self._write(val);
    }
}
