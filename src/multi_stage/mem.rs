use log::{debug, trace};

use crate::{
    core::{insts::sext, vm::VirtualMemory},
    multi_stage::debug::m_pinst,
};

use super::phases::{InternalExecMem, InternalMemWb};

pub fn mem(
    itl_e_m: &InternalExecMem,
    vm: &mut VirtualMemory,
    pipeline_info: bool,
) -> InternalMemWb {
    if pipeline_info {
        trace!("MEM: {}", m_pinst(itl_e_m));
    }

    let mem_read = itl_e_m.mem_flags.mem_read;
    let mem_write = itl_e_m.mem_flags.mem_write;

    let vaddr = itl_e_m.mem_addr as usize;
    let mem_bitwidth = &itl_e_m.mem_bitwidth;
    let mem_sext_to = &itl_e_m.mem_sext_to;
    let alu_out = itl_e_m.alu_out;

    // Mux for memory-to-memory hazard
    let mut regval = if itl_e_m.m2m_forward {
        itl_e_m.m2m_forward_val
    } else {
        alu_out
    };

    assert!(!(mem_read & mem_write));
    if mem_read {
        if pipeline_info {
            debug!("MEM.read {:#x}", vaddr);
        }
        assert!(!itl_e_m.m2m_forward);
        let result = match mem_bitwidth {
            8 => vm.mread::<u8>(vaddr) as u64,
            16 => vm.mread::<u16>(vaddr) as u64,
            32 => vm.mread::<u32>(vaddr) as u64,
            64 => vm.mread::<u64>(vaddr) as u64,
            _ => unreachable!("MEM.read"),
        };
        let result = match mem_sext_to {
            8 => sext(result, 8) as u64,
            16 => sext(result, 16) as u64,
            32 => sext(result, 32) as u64,
            0 => result,
            _ => unreachable!("MEM.read"),
        };
        regval = result;
    }
    if mem_write {
        if pipeline_info {
            debug!("MEM.write {:#x} -> M[{:#x}]", regval, vaddr);
        }
        match mem_bitwidth {
            8 => vm.mwrite::<u8>(vaddr, regval as u8),
            16 => vm.mwrite::<u16>(vaddr, regval as u16),
            32 => vm.mwrite::<u32>(vaddr, regval as u32),
            64 => vm.mwrite::<u64>(vaddr, regval),
            _ => unreachable!("MEM.write"),
        }
    }

    InternalMemWb {
        wb_flags: itl_e_m.wb_flags,
        branch_flags: itl_e_m.branch_flags,
        mem_read,
        pc: itl_e_m.pc,
        rs1: itl_e_m.rs1,
        rs2: itl_e_m.rs2,
        rs3: itl_e_m.rs3,
        rd: itl_e_m.rd,
        imm: itl_e_m.imm,
        regval,
        alu_op: itl_e_m.alu_op,
    }
}
