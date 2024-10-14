use std::ops::{BitAnd, BitAndAssign, BitOrAssign};

use super::{PrivilegeLevel, XLEN};

/// The mstatus register keeps track of and controls the hart’s current operating state.
#[repr(transparent)]
pub struct Mstatus(u64);

const MSTATUS_SIE_MASK: u64 = 0b1 << 1;
const MSTATUS_MIE_MASK: u64 = 0b1 << 3;
const MSTATUS_SPIE_MASK: u64 = 0b1 << 5;
const MSTATUS_UBE_MASK: u64 = 0b1 << 6;
const MSTATUS_MPIE_MASK: u64 = 0b1 << 7;
const MSTATUS_SPP_MASK: u64 = 0b1 << 8;
const MSTATUS_VS_MASK: u64 = 0b11 << 9;
const MSTATUS_MPP_MASK: u64 = 0b11 << 11;
const MSTATUS_FS_MASK: u64 = 0b11 << 13;
const MSTATUS_XS_MASK: u64 = 0b11 << 15;
const MSTATUS_MPRV_MASK: u64 = 0b1 << 17;
const MSTATUS_SUM_MASK: u64 = 0b1 << 18;
const MSTATUS_MXR_MASK: u64 = 0b1 << 19;
const MSTATUS_TVM_MASK: u64 = 0b1 << 20;
const MSTATUS_TW_MASK: u64 = 0b1 << 21;
const MSTATUS_TSR_MASK: u64 = 0b1 << 22;
const MSTATUS_SD_MASK: u64 = 0b1 << (XLEN - 1);

const USER_MODE_PRIVILEGE: u64 = 0b00 << 11;
const SUPERVISOR_MODE_PRIVILEGE: u64 = 0b01 << 11;
const MACHINE_MODE_PRIVILEGE: u64 = 0b11 << 11;

impl Mstatus {
    pub fn sie(&self) -> u64 {
        self.0.bitand(MSTATUS_SIE_MASK)
    }

    pub fn mie(&self) -> u64 {
        self.0.bitand(MSTATUS_MIE_MASK)
    }

    /// Original value of sie.
    pub fn spie(&self) -> u64 {
        self.0.bitand(MSTATUS_SPIE_MASK)
    }

    pub fn ube_mask(&self) -> u64 {
        self.0.bitand(MSTATUS_UBE_MASK)
    }

    /// Original value of mie.
    pub fn mpie(&self) -> u64 {
        self.0.bitand(MSTATUS_MPIE_MASK)
    }

    pub fn spp(&self) -> u64 {
        self.0.bitand(MSTATUS_SPP_MASK)
    }

    fn _mpp(&self) -> u64 {
        self.0.bitand(MSTATUS_MPP_MASK)
    }

    pub fn mpp(&self) -> PrivilegeLevel {
        let val = self._mpp();
        match val {
            MACHINE_MODE_PRIVILEGE => PrivilegeLevel::Machine,
            SUPERVISOR_MODE_PRIVILEGE => PrivilegeLevel::Supervisor,
            USER_MODE_PRIVILEGE => PrivilegeLevel::User,
            _ => unreachable!(),
        }
    }

    pub fn fs(&self) -> u64 {
        self.0.bitand(MSTATUS_FS_MASK)
    }

    pub fn xs(&self) -> u64 {
        self.0.bitand(MSTATUS_XS_MASK)
    }

    pub fn mprv(&self) -> u64 {
        self.0.bitand(MSTATUS_MPRV_MASK)
    }

    pub fn sum(&self) -> u64 {
        self.0.bitand(MSTATUS_SUM_MASK)
    }

    pub fn mxr(&self) -> u64 {
        self.0.bitand(MSTATUS_MXR_MASK)
    }

    pub fn tvm(&self) -> u64 {
        self.0.bitand(MSTATUS_TVM_MASK)
    }

    pub fn tw(&self) -> u64 {
        self.0.bitand(MSTATUS_TW_MASK)
    }

    pub fn tsr(&self) -> u64 {
        self.0.bitand(MSTATUS_TSR_MASK)
    }

    pub fn read(&self) -> u64 {
        self.0
    }

    pub fn write(&mut self, val: u64) {
        self.0 = val;
    }

    /// Clear mstatus.MIE and restore it into MPIE
    pub fn mie2mpie_and_clear(&mut self) {
        let bit = if self.mie() != 0 {
            MSTATUS_MPIE_MASK
        } else {
            0
        };
        self.0.bitor_assign(bit);
        self.0.bitand_assign(!MSTATUS_MIE_MASK);
    }

    /// Copy mstatus.MPIE to MIE.
    /// Used by mret.
    pub fn mpie2mie(&mut self) {
        let bit = if self.mpie() != 0 {
            MSTATUS_MIE_MASK
        } else {
            0
        };
        self.0.bitor_assign(bit);
    }

    pub fn mpp_set(&mut self, lv: PrivilegeLevel) {
        let val = match lv {
            PrivilegeLevel::Machine => MACHINE_MODE_PRIVILEGE,
            PrivilegeLevel::Supervisor => SUPERVISOR_MODE_PRIVILEGE,
            PrivilegeLevel::User => USER_MODE_PRIVILEGE,
        };
        self.0.bitand_assign(!MSTATUS_MPP_MASK);
        self.0.bitor_assign(val);
    }
}
