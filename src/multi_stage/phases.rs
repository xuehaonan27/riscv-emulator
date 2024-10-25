use crate::core::insts::Inst64;

use super::ctrl_flags::*;

#[derive(Debug, Clone, Copy)]
pub struct InternalFetchDecode {
    pub raw_inst: u32,
    pub decode_flags: DecodeFlags,
    pub exec_flags: ExecFlags,
    pub mem_flags: MemFlags,
    pub wb_flags: WbFlags,
    pub branch_flags: BranchFlags,
    pub pc: u64, // current instruction PC
    pub rs1: u8,
    pub rs2: u8,
    #[allow(unused)]
    pub rs3: u8,
    pub rd: u8,
    pub imm: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct InternalDecodeExec {
    pub raw_inst: u32,
    pub exec_flags: ExecFlags,
    pub mem_flags: MemFlags,
    pub wb_flags: WbFlags,
    pub branch_flags: BranchFlags,
    pub pc: u64, // current instruction PC
    pub rs1: u8,
    pub rs2: u8,
    #[allow(unused)]
    pub rs3: u8,
    pub rd: u8,
    pub src1: u64,
    pub src2: u64,
    pub imm: u64,
    pub forward_a: u8,
    pub forward_b: u8,
    pub ex_mem_forward: u64,
    pub mem_wb_forward: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct InternalExecMem {
    pub raw_inst: u32,
    pub mem_flags: MemFlags,
    pub wb_flags: WbFlags,
    pub branch_flags: BranchFlags,
    pub pc: u64, // current instruction PC
    pub rs1: u8,
    pub rs2: u8,
    #[allow(unused)]
    pub rs3: u8,
    pub rd: u8,
    pub imm: u64, // for pinst
    pub alu_out: u64,
    pub mem_addr: u64,
    pub mem_bitwidth: u8,
    pub mem_sext_to: u8,
    pub m2m_forward: bool,    // whether receive data forward from MEM/WB
    pub m2m_forward_val: u64, //data forward from MEM/WB
    pub alu_op: Inst64,       // for branch hazard detection
}

impl InternalExecMem {
    pub fn is_call(&self) -> bool {
        self.alu_op == Inst64::jal && self.rd == 1
    }

    pub fn is_ret(&self) -> bool {
        self.raw_inst == 0x00008067
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InternalMemWb {
    pub raw_inst: u32,
    pub wb_flags: WbFlags,
    pub branch_flags: BranchFlags,
    pub mem_read: bool, // for memory-to-memory hazard detection
    pub pc: u64,        // current instruction PC
    pub rs1: u8,
    pub rs2: u8,
    #[allow(unused)]
    pub rs3: u8,
    pub rd: u8,
    pub imm: u64, // for pinst
    pub regval: u64,
    pub alu_op: Inst64, // for ebreak
}

impl Default for InternalFetchDecode {
    fn default() -> Self {
        Self {
            raw_inst: 0,
            decode_flags: DecodeFlags {
                sext: SextType::None,
            },
            exec_flags: ExecFlags {
                alu_op: Inst64::noop,
                alu_src: false,
            },
            mem_flags: MemFlags {
                mem_read: false,
                mem_write: false,
            },
            wb_flags: WbFlags { mem_to_reg: false },
            branch_flags: BranchFlags {
                branch: false,
                pc_src: false,
                predicted_src: false,
                predicted_target: 0,
            },
            pc: 0,
            rs1: 0,
            rs2: 0,
            rs3: 0,
            rd: 0,
            imm: 0,
        }
    }
}

impl Default for InternalDecodeExec {
    fn default() -> Self {
        Self {
            raw_inst: 0,
            exec_flags: ExecFlags {
                alu_op: Inst64::noop,
                alu_src: false,
            },
            mem_flags: MemFlags {
                mem_read: false,
                mem_write: false,
            },
            wb_flags: WbFlags { mem_to_reg: false },
            branch_flags: BranchFlags {
                branch: false,
                pc_src: false,
                predicted_src: false,
                predicted_target: 0,
            },
            pc: 0,
            rs1: 0,
            rs2: 0,
            rs3: 0,
            rd: 0,
            src1: 0,
            src2: 0,
            imm: 0,
            forward_a: 0,
            forward_b: 0,
            ex_mem_forward: 0,
            mem_wb_forward: 0,
        }
    }
}

impl Default for InternalExecMem {
    fn default() -> Self {
        Self {
            raw_inst: 0,
            mem_flags: MemFlags {
                mem_read: false,
                mem_write: false,
            },
            wb_flags: WbFlags { mem_to_reg: false },
            branch_flags: BranchFlags {
                branch: false,
                pc_src: false,
                predicted_src: false,
                predicted_target: 0,
            },
            pc: 0,
            rs1: 0,
            rs2: 0,
            rs3: 0,
            rd: 0,
            imm: 0,
            alu_out: 0,
            mem_addr: 0,
            mem_bitwidth: 0,
            mem_sext_to: 0,
            m2m_forward: false,
            m2m_forward_val: 0,
            alu_op: Inst64::noop,
        }
    }
}

impl Default for InternalMemWb {
    fn default() -> Self {
        Self {
            raw_inst: 0,
            wb_flags: WbFlags { mem_to_reg: false },
            branch_flags: BranchFlags {
                branch: false,
                pc_src: false,
                predicted_src: false,
                predicted_target: 0,
            },
            mem_read: false,
            pc: 0,
            rs1: 0,
            rs2: 0,
            rs3: 0,
            rd: 0,
            imm: 0,
            regval: 0,
            alu_op: Inst64::noop,
        }
    }
}
