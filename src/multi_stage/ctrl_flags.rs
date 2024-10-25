use crate::core::insts::Inst64;

#[derive(Debug, Clone, Copy)]
pub enum SextType {
    None,
    I,
    S,
    B,
    U,
    J
}

#[derive(Debug, Clone, Copy)]
pub struct DecodeFlags {
    pub sext: SextType,
}

impl DecodeFlags {
    pub fn clear(&mut self) {
        self.sext = SextType::None;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BranchFlags {
    pub branch: bool,
    pub pc_src: bool,
    pub predicted_src: bool,
    pub predicted_target: u64,
}

impl BranchFlags {
    pub fn clear(&mut self) {
        self.branch = false;
        self.pc_src = false;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ExecFlags {
    pub alu_op: Inst64,
    pub alu_src: bool,
}

impl ExecFlags {
    pub fn clear(&mut self) {
        self.alu_op = Inst64::noop;
        self.alu_src = false;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemFlags {
    pub mem_read: bool,
    pub mem_write: bool,
}

impl MemFlags {
    pub fn clear(&mut self) {
        self.mem_read = false;
        self.mem_write = false;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WbFlags {
    pub mem_to_reg: bool,
}

impl WbFlags {
    pub fn clear(&mut self) {
        self.mem_to_reg = false;
    }
}
