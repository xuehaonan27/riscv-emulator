use log::{debug, info, trace};

use crate::{
    core::{insts::Inst64, reg::RegisterFile, utils::reg_name_by_id},
    multi_stage::{cpu::halt, debug::w_pinst},
};

use super::phases::InternalMemWb;

pub fn writeback(
    itl_m_w: &InternalMemWb,
    reg_file: &mut RegisterFile,
    pipeline_info: bool,
) -> bool {
    if pipeline_info {
        trace!("WB : {}", w_pinst(itl_m_w));
    }

    let mem_to_reg = itl_m_w.wb_flags.mem_to_reg;
    if mem_to_reg {
        let write_val = if itl_m_w.rd != 0 { itl_m_w.regval } else { 0 };
        if pipeline_info {
            debug!(
                "WB : {:#x} -> REG[{}]({})",
                write_val,
                itl_m_w.rd,
                reg_name_by_id(itl_m_w.rd)
            )
        };
        reg_file.write(itl_m_w.rd, write_val);
    }

    if itl_m_w.alu_op == Inst64::ebreak {
        let x10 = reg_file.read(10);
        let msg = format!("ebreak at {:#x}, code {}", itl_m_w.pc, x10);
        info!("{msg}");
        halt(itl_m_w.pc, x10); // HALT at current code.
        false
    } else {
        true
    }
}
