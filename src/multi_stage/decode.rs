use log::trace;

use crate::{
    core::{insts::*, reg::RegisterFile},
    multi_stage::debug::d_pinst,
};

use super::{
    ctrl_flags::SextType,
    phases::{InternalDecodeExec, InternalFetchDecode},
};

pub fn decode(
    reg_file: &RegisterFile,
    itl_f_d: &InternalFetchDecode,
    pipeline_info: bool,
) -> InternalDecodeExec {
    if pipeline_info {
        trace!("ID : {}", d_pinst(itl_f_d));
    }

    let src1 = reg_file.read(itl_f_d.rs1);
    let src2 = reg_file.read(itl_f_d.rs2);
    let imm = itl_f_d.imm;
    let imm = match itl_f_d.decode_flags.sext {
        SextType::None => imm,
        SextType::I => sext(imm, I_TYPE_IMM_BITWIDTH) as u64,
        SextType::S => sext(imm, S_TYPE_IMM_BITWIDTH) as u64,
        SextType::B => sext(imm, B_TYPE_IMM_BITWIDTH) as u64,
        SextType::J => sext(imm, J_TYPE_IMM_BITWIDTH) as u64,
        SextType::U => sext(imm, U_TYPE_IMM_BITWIDTH) as u64,
    };

    let itl_d_e = InternalDecodeExec {
        raw_inst: itl_f_d.raw_inst,
        exec_flags: itl_f_d.exec_flags,
        mem_flags: itl_f_d.mem_flags,
        wb_flags: itl_f_d.wb_flags,
        branch_flags: itl_f_d.branch_flags,
        pc: itl_f_d.pc,
        rs1: itl_f_d.rs1,
        rs2: itl_f_d.rs2,
        rs3: itl_f_d.rs3,
        rd: itl_f_d.rd,
        src1,
        src2,
        imm,
        forward_a: 0, // default using self
        forward_b: 0, // default using self
        ex_mem_forward: 0, // set by data forwarding logic
        mem_wb_forward: 0, // set by data forwarding logic
    };

    itl_d_e
}
