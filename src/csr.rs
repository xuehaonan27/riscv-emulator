pub use mcause::Mcause;
pub use mepc::Mepc;
use mhartid::Mhartid;
pub use mie::Mie;
pub use mip::Mip;
pub use mscratch::Mscratch;
pub use mstatus::Mstatus;
pub use mtval::Mtval;
pub use mtvec::Mtvec;
pub use scause::Scause;
pub use sepc::Sepc;
pub use sie::Sie;
pub use sip::Sip;
pub use sscratch::Sscratch;
pub use sstatus::Sstatus;
pub use stval::Stval;
pub use stvec::Stvec;

pub mod mcause;
pub mod mepc;
pub mod mhartid;
pub mod mie;
pub mod mip;
pub mod misa;
pub mod mscratch;
pub mod mstatus;
pub mod mtval;
pub mod mtvec;

pub mod scause;
pub mod sepc;
pub mod sie;
pub mod sip;
pub mod sscratch;
pub mod sstatus;
pub mod stval;
pub mod stvec;

/// Machine word length under 64-bit
pub const XLEN: usize = 64;

/// Machine Mode Control Status Register File.
#[repr(C)]
pub struct MModeRegFile {
    pub mstatus: Mstatus,
    pub mip: Mip,
    pub mie: Mie,
    pub mcause: Mcause,
    pub mtvec: Mtvec,
    pub mtval: Mtval,
    pub mepc: Mepc,
    pub mscratch: Mscratch,
    pub mhartid: Mhartid, // One `mhartid` for a core
}

/// Supervisor Mode Control Status Register File.
pub struct SModeRegFile {
    pub sstatus: Sstatus,
    pub sip: Sip,
    pub sie: Sie,
    pub scause: Scause,
    pub stvec: Stvec,
    pub stval: Stval,
    pub sepc: Sepc,
    pub sscratch: Sscratch,
}

impl MModeRegFile {
    /// Get an empty register file
    pub fn empty() -> MModeRegFile {
        unsafe { std::mem::zeroed() }
    }
}

impl SModeRegFile {
    /// Get an empty register file
    pub fn empty() -> SModeRegFile {
        unsafe { std::mem::zeroed() }
    }
}

pub enum PrivilegeLevel {
    Machine,
    Supervisor,
    User,
}
