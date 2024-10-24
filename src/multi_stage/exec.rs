use std::ops::{BitAnd, BitOr, BitXor};

use log::{trace, warn};

use crate::{
    callstack::CallStack, core::insts::{
        get_high_64_bit, sext, trunc_to_16_bit, trunc_to_32_bit, trunc_to_5_bit,
        trunc_to_5_bit_and_check, trunc_to_6_bit, trunc_to_8_bit, BYTE_BITWIDTH, HALF_BITWIDTH,
        WORD_BITWIDTH,
    }, error::{Error, Exception, Result}, multi_stage::{ctrl_flags::BranchFlags, debug::e_pinst}
};

use super::phases::{InternalDecodeExec, InternalExecMem};

pub fn exec(
    itl_d_e: &InternalDecodeExec,
    pipeline_info: bool,
    callstack: &mut CallStack,
) -> Result<(InternalExecMem, bool, bool, u64, u64)> {
    use crate::core::insts::Inst64::*;
    if pipeline_info {
        trace!("EX : {}", e_pinst(itl_d_e));
    }

    let ex_mem_forward = itl_d_e.ex_mem_forward;
    let mem_wb_forward = itl_d_e.mem_wb_forward;

    // data forward
    let src1 = match itl_d_e.forward_a {
        0 => itl_d_e.src1,
        0b10 => {
            if pipeline_info {
                warn!("ALU SRC A received data from EX/MEM: {ex_mem_forward}");
            }
            ex_mem_forward
        }
        0b01 => {
            if pipeline_info {
                warn!("ALU SRC A received data from MEM/WB: {ex_mem_forward}");
            }
            mem_wb_forward
        }
        _ => unreachable!("Data forwarding A"),
    };

    let src2 = match itl_d_e.forward_b {
        0 => itl_d_e.src2,
        0b10 => {
            if pipeline_info {
                warn!("ALU SRC B received data from EX/MEM: {ex_mem_forward}");
            }
            ex_mem_forward
        }
        0b01 => {
            if pipeline_info {
                warn!("ALU SRC B received data from MEM/WB: {ex_mem_forward}");
            }
            mem_wb_forward
        }
        _ => unreachable!("Data forwarding B"),
    };

    let imm = itl_d_e.imm;
    let pc = itl_d_e.pc;

    let ex_branch = itl_d_e.branch_flags.branch;
    let mut pc_src = itl_d_e.branch_flags.pc_src;
    let new_pc_0 = pc.wrapping_add(4);
    let mut new_pc_1 = pc.wrapping_add(4);

    let mut mem_addr = 0;
    let mem_bitwidth = match itl_d_e.exec_flags.alu_op {
        lb | lbu | sb => 8,
        lh | lhu | sh => 16,
        lw | lwu | sw => 32,
        ld | sd => 64,
        _ => 0,
    };
    let mem_sext_to = match itl_d_e.exec_flags.alu_op {
        lb => BYTE_BITWIDTH,
        lh => HALF_BITWIDTH,
        lw => WORD_BITWIDTH,
        _ => 0,
    };

    let alu_out = match itl_d_e.exec_flags.alu_op {
        noop => 0,
        auipc => pc.wrapping_add((imm as u64) << 12),
        lui => {
            let mask: u64 = !0b1111_1111_1111;
            let result = ((imm << 12) as u64) & mask;
            result
        }
        lb | lh | lw | ld | lbu | lhu | lwu => {
            mem_addr = src1.wrapping_add(imm);
            0
        }
        sb => {
            let vaddr = src1.wrapping_add(imm);
            mem_addr = vaddr;
            let result = trunc_to_8_bit(src2);
            result
        }
        sh => {
            let vaddr = src1.wrapping_add(imm);
            mem_addr = vaddr;
            let result = trunc_to_16_bit(src2);
            result
        }
        sw => {
            let vaddr = src1.wrapping_add(imm);
            mem_addr = vaddr;
            let result = trunc_to_32_bit(src2);
            result
        }
        sd => {
            let vaddr = src1.wrapping_add(imm);
            mem_addr = vaddr;
            let result = src2;
            result
        }
        jal => {
            pc_src = true;
            new_pc_1 = pc.wrapping_add(imm);
            let result = new_pc_0;

            // call
            callstack.call(pc, new_pc_1);

            result
        }
        jalr => {
            pc_src = true;
            new_pc_1 = src1.wrapping_add(imm) & (!1);
            let result = new_pc_0;

            // ret
            // 00008067          	jalr	zero,0(ra)
            if itl_d_e.rd == 0 && itl_d_e.imm == 0 && itl_d_e.rs1 == 1 {
                callstack.ret(pc);
            }

            result
        }
        beq => {
            if src1 == src2 {
                new_pc_1 = pc.wrapping_add(imm);
                pc_src = true;
            } else {
                pc_src = false;
            }
            0
        }
        bge => {
            if (src1 as i64) >= (src2 as i64) {
                new_pc_1 = pc.wrapping_add(imm);
                pc_src = true;
            }
            0
        }
        bgeu => {
            if (src1 as u64) >= (src2 as u64) {
                new_pc_1 = pc.wrapping_add(imm);
                pc_src = true;
            }
            0
        }
        blt => {
            if (src1 as i64) < (src2 as i64) {
                new_pc_1 = pc.wrapping_add(imm);
                pc_src = true;
            }
            0
        }
        bltu => {
            if (src1 as u64) < (src2 as u64) {
                new_pc_1 = pc.wrapping_add(imm);
                pc_src = true;
            }
            0
        }
        bne => {
            if src1 != src2 {
                new_pc_1 = pc.wrapping_add(imm);
                pc_src = true;
            }
            0
        }
        ecall => {
            todo!("ecall");
        }
        ebreak => 0,
        add => src1.wrapping_add(src2),
        addi => src1.wrapping_add(imm),
        addiw => {
            let result = src1.wrapping_add(imm);
            let result = sext(trunc_to_32_bit(result), WORD_BITWIDTH);
            result as u64
        }
        addw => {
            let result = src1.wrapping_add(src2);
            let result = sext(trunc_to_32_bit(result), WORD_BITWIDTH);
            result as u64
        }
        sub => src1.wrapping_sub(src2),
        subw => {
            let result = trunc_to_32_bit(src1.wrapping_sub(src2));
            let result = sext(result, WORD_BITWIDTH);
            result as u64
        }
        slt => {
            if (src1 as i64) < (src2 as i64) {
                1
            } else {
                0
            }
        }
        slti => {
            if (src1 as i64) < imm as i64 {
                1
            } else {
                0
            }
        }
        sltu => {
            if (src1 as u64) < (src2 as u64) {
                1
            } else {
                0
            }
        }
        sltiu => {
            if (src1 as u64) < (imm as u64) {
                1
            } else {
                0
            }
        }
        xor => src1.bitxor(src2),
        xori => src1.bitxor(imm),
        or => src1.bitor(src2),
        ori => src1.bitor(imm),
        and => src1.bitand(src2),
        andi => src1.bitand(imm),
        sll => {
            let t_src2 = trunc_to_6_bit(src2); // RV64
            let result = src1.wrapping_shl(t_src2 as u32);
            result
        }
        slli => {
            let shamt = trunc_to_6_bit(imm);
            let result = src1.wrapping_shl(shamt as u32);
            result
        }
        slliw => {
            let (shamt, legal) = trunc_to_5_bit_and_check(imm);
            if !legal {
                return Err(Error::Exception(Exception::IllegalInstruction));
            }
            let result = src1.wrapping_shl(shamt as u32);
            let result = sext(trunc_to_32_bit(result), WORD_BITWIDTH);
            result as u64
        }
        sllw => {
            let t_src1 = trunc_to_32_bit(src1);
            let t_src2 = trunc_to_5_bit(src2);
            let result = t_src1.wrapping_shl(t_src2 as u32);
            let result = sext(trunc_to_32_bit(result), WORD_BITWIDTH);
            result as u64
        }
        srl => {
            let t_src2 = trunc_to_6_bit(src2); // RV64
                                               // i64 shr automatically fill high bits with 0-bit
            let result = (src1 as u64).wrapping_shr(t_src2 as u32);
            result as u64
        }
        srli => {
            let shamt = trunc_to_6_bit(imm);
            let result = (src1 as u64).wrapping_shr(shamt as u32);
            result
        }
        srliw => {
            let t_src1: u64 = trunc_to_32_bit(src1);
            let (shamt, legal) = trunc_to_5_bit_and_check(imm);
            if !legal {
                return Err(Error::Exception(Exception::IllegalInstruction));
            }
            let result = sext(
                trunc_to_32_bit(t_src1.wrapping_shr(shamt as u32)),
                WORD_BITWIDTH,
            );
            result as u64
        }
        srlw => {
            let t_src1: u64 = trunc_to_32_bit(src1);
            let t_src2 = trunc_to_5_bit(src2);
            let result = t_src1.wrapping_shr(t_src2 as u32);
            let result = sext(result as u64, WORD_BITWIDTH);
            result as u64
        }
        sra => {
            let t_src2 = trunc_to_6_bit(src2); // RV64
                                               // i64 shr automatically fill high bits with sign-bit
            let result = (src1 as i64).wrapping_shr(t_src2 as u32);
            result as u64
        }
        srai => {
            let shamt = trunc_to_6_bit(imm);
            let result = (src1 as i64).wrapping_shr(shamt as u32);
            result as u64
        }
        sraiw => {
            let t_src1: i64 = sext(trunc_to_32_bit(src1), WORD_BITWIDTH);
            let (shamt, legal) = trunc_to_5_bit_and_check(imm);
            if !legal {
                return Err(Error::Exception(Exception::IllegalInstruction));
            }
            let result = t_src1.wrapping_shr(shamt as u32);
            result as u64
        }
        sraw => {
            let t_src1: i64 = sext(trunc_to_32_bit(src1), WORD_BITWIDTH);
            let t_src2 = trunc_to_5_bit(src2);
            let result = t_src1.wrapping_shr(t_src2 as u32);
            let result = sext(result as u64, WORD_BITWIDTH);
            result as u64
        }
        mul => src1.wrapping_mul(src2),
        mulh => {
            // RV64
            let result = (src1 as i128).wrapping_mul(src2 as i128);
            let result = get_high_64_bit(result as u128);
            result
        }
        mulhsu => {
            let t_src1 = src1 as i64;
            let t_src2 = src2 as u64;
            let result = (t_src1 as i128).wrapping_mul(t_src2 as i128);
            let result = get_high_64_bit(result as u128);
            result
        }
        mulhu => {
            let result = (src1 as u128).wrapping_mul(src2 as u128);
            let result = get_high_64_bit(result);
            result
        }
        mulw => {
            let result = src1.wrapping_mul(src2);
            let result = sext(trunc_to_32_bit(result), WORD_BITWIDTH);
            result as u64
        }
        div => {
            if src2 == 0 {
                return Err(Error::Exception(Exception::DividedByZero));
            }
            let result = (src1 as i64).wrapping_div(src2 as i64);
            result as u64
        }
        divu => {
            if src2 == 0 {
                return Err(Error::Exception(Exception::DividedByZero));
            }
            let result = src1.wrapping_div(src2);
            result
        }
        divuw => {
            if trunc_to_32_bit(src2) == 0 {
                return Err(Error::Exception(Exception::DividedByZero));
            }
            let result = trunc_to_32_bit(src1).wrapping_div(trunc_to_32_bit(src2));
            let result = sext(trunc_to_32_bit(result), WORD_BITWIDTH);
            result as u64
        }
        divw => {
            if trunc_to_32_bit(src2) == 0 {
                return Err(Error::Exception(Exception::DividedByZero));
            }
            let result = (trunc_to_32_bit(src1) as i32).wrapping_div(trunc_to_32_bit(src2) as i32);
            let result = sext(trunc_to_32_bit(result as u64), WORD_BITWIDTH);
            result as u64
        }
        rem => {
            if src2 == 0 {
                return Err(Error::Exception(Exception::DividedByZero));
            }
            let result = (src1 as i64).wrapping_rem(src2 as i64);
            result as u64
        }
        remu => {
            if src2 == 0 {
                return Err(Error::Exception(Exception::DividedByZero));
            }
            let result = src1.wrapping_rem(src2);
            result
        }
        remuw => {
            if src2 == 0 {
                return Err(Error::Exception(Exception::DividedByZero));
            }
            let t_src1 = trunc_to_32_bit(src1);
            let t_src2 = trunc_to_32_bit(src2);
            let result = t_src1.wrapping_rem(t_src2);
            let result = sext(result, WORD_BITWIDTH);
            result as u64
        }
        remw => {
            if src2 == 0 {
                return Err(Error::Exception(Exception::DividedByZero));
            }
            let t_src1 = trunc_to_32_bit(src1);
            let t_src2 = trunc_to_32_bit(src2);
            let result = (t_src1 as i64).wrapping_rem(t_src2 as i64);
            result as u64
        }
        csrrc | csrrci | csrrs | csrrsi | csrrw | csrrwi | mret | sret | fence | fence_i | wfi => {
            unimplemented!("Control registers")
        }
    };

    let itl_e_m = InternalExecMem {
        mem_flags: itl_d_e.mem_flags,
        wb_flags: itl_d_e.wb_flags,
        branch_flags: BranchFlags {
            pc_src,
            ..itl_d_e.branch_flags
        },
        pc: itl_d_e.pc,
        rs1: itl_d_e.rs1,
        rs2: itl_d_e.rs2,
        rs3: itl_d_e.rs3,
        rd: itl_d_e.rd,
        imm: itl_d_e.imm,
        alu_out,
        mem_addr,
        mem_bitwidth,
        mem_sext_to,
        m2m_forward: false, // set by hazard detect unit
        m2m_forward_val: 0, // set by hazard detect unit
        alu_op: itl_d_e.exec_flags.alu_op,
    };

    Ok((itl_e_m, ex_branch, pc_src, new_pc_0, new_pc_1))
}
