use std::ops::{BitAnd, BitOr, BitOrAssign};

/// The mcause register is an MXLEN-bit read-write register formatted
/// as shown in Figure 1.21. When a trap is taken into M-mode,  mcause
/// is written with a code indicating the event that caused the trap. 
/// Otherwise, mcause is never written by the implementation, though 
/// it may be explicitly written by software.
/// 
/// The Interrupt bit in the mcause register is set if the trap was 
/// caused by an interrupt. The Exception Code field contains a code 
/// identifying the last exception or interrupt. Table [mcauses] lists 
/// the possible machine-level exception codes. The Exception Code is 
/// a WLRL field, so is only guaranteed to hold supported exception codes.
#[repr(transparent)]
pub struct Mcause(u64);

const MCAUSE_INTERRUPT_MASK: u64 = 0x80000000_00000000;
const MCAUSE_EXCEPTION_MASK: u64 = 0x7FFFFFFF_FFFFFFFF;

/* Interrupts */
pub const MCAUSE_SMODE_SOFTWARE_INT: u64 = 0x80000000_00000001;
pub const MCAUSE_MMODE_SOFTWARE_INT: u64 = 0x80000000_00000003;
pub const MCAUSE_SMODE_TIME_INT: u64 = 0x80000000_00000005;
pub const MCAUSE_MMODE_TIME_INT: u64 = 0x80000000_00000007;
pub const MCAUSE_SMODE_EXTERNAL_INT: u64 = 0x80000000_00000009;
pub const MCAUSE_MMODE_EXTERNAL_INT: u64 = 0x80000000_0000000b;

/* Exceptions */
pub const MCAUSE_INSTRUCTION_ADDRESS_MISALIGNED: u64 = 0x00000000_00000000;
pub const MCAUSE_INSTRUCTION_ACCESS_FAULT: u64 = 0x00000000_00000001;
pub const MCAUSE_ILLEGAL_INSTRUCTION: u64 = 0x00000000_00000002;
pub const MCAUSE_BREAKPOINT: u64 = 0x00000000_00000003;
pub const MCAUSE_LOAD_ADDRESS_MISALIGNED: u64 = 0x00000000_00000004;
pub const MCAUSE_LOAD_ACCESS_FAULT: u64 = 0x00000000_00000005;
pub const MCAUSE_STORE_ADDRESS_MISALIGNED: u64 = 0x00000000_00000006;
pub const MCAUSE_STORE_ACCESS_FAULT: u64 = 0x00000000_00000007;
pub const MCAUSE_UMODE_ECALL: u64 = 0x00000000_00000008;
pub const MCAUSE_SMODE_ECALL: u64 = 0x00000000_00000009;
pub const MCAUSE_MMODE_ECALL: u64 = 0x00000000_0000000b;
pub const MCAUSE_INSTRUCTION_PAGE_FAULT: u64 = 0x00000000_0000000c;
pub const MCAUSE_LOAD_PAGE_FAULT: u64 = 0x00000000_0000000d;
pub const MCAUSE_STORE_PAGE_FAULT: u64 = 0x00000000_0000000f;

impl Mcause {
    pub fn interrupt(&self) -> u64 {
        self.0.bitand(MCAUSE_INTERRUPT_MASK)
    }

    pub fn set_interrupt(&mut self) {
        self.0.bitor_assign(MCAUSE_INTERRUPT_MASK);
    }

    pub fn exception(&self) -> u64 {
        self.0.bitand(MCAUSE_EXCEPTION_MASK)
    }

    pub fn set_exception(&mut self, num: u64) {
        self.0 = self.0.bitand(MCAUSE_INTERRUPT_MASK).bitor(num);
    }

    pub fn read(&self) -> u64 {
        self.0
    }

    pub fn write(&mut self, val: u64) {
        self.0 = val;
    }
}
