use crate::{
    core::{insts::Inst64, reg::REGNAME},
    error::{Error, Result},
    pinst,
};

use super::phases::{InternalDecodeExec, InternalExecMem, InternalFetchDecode, InternalMemWb};

pub fn f_pinst(itl: &InternalFetchDecode) -> String {
    pinst(
        itl.pc,
        itl.exec_flags.alu_op,
        itl.rd,
        itl.rs1,
        itl.rs2,
        itl.imm,
    )
}

pub fn d_pinst(itl: &InternalFetchDecode) -> String {
    pinst(
        itl.pc,
        itl.exec_flags.alu_op,
        itl.rd,
        itl.rs1,
        itl.rs2,
        itl.imm,
    )
}

pub fn e_pinst(itl: &InternalDecodeExec) -> String {
    pinst(
        itl.pc,
        itl.exec_flags.alu_op,
        itl.rd,
        itl.rs1,
        itl.rs2,
        itl.imm,
    )
}

pub fn m_pinst(itl: &InternalExecMem) -> String {
    pinst(itl.pc, itl.alu_op, itl.rd, itl.rs1, itl.rs2, itl.imm)
}

pub fn w_pinst(itl: &InternalMemWb) -> String {
    pinst(itl.pc, itl.alu_op, itl.rd, itl.rs1, itl.rs2, itl.imm)
}

fn pinst(pc: u64, alu_op: Inst64, rd: u8, rs1: u8, rs2: u8, imm: u64) -> String {
    use crate::core::insts::Inst64::*;
    let msg = match alu_op {
        noop => pinst!(pc, noop),
        add => pinst!(pc, add, rd, rs1, rs2),
        addi => pinst!(pc, addi, rd, rs1, imm=>imm),
        addiw => pinst!(pc, addiw, rd, rs1, imm=>imm),
        addw => pinst!(pc, addw, rd, rs1, rs2),
        and => pinst!(pc, and, rd, rs1, rs2),
        andi => pinst!(pc, andi, rd, rs1, imm=>imm),
        auipc => pinst!(pc, auipc, rd, imm=>imm),
        beq => pinst!(pc, beq, rs1, rs2, imm=>offset),
        bge => pinst!(pc, bge, rs1, rs2, imm=>offset),
        bgeu => pinst!(pc, bgeu, rs1, rs2, imm=>offset),
        blt => pinst!(pc, blt, rs1, rs2, imm=>offset),
        bltu => pinst!(pc, bltu, rs1, rs2, imm=>offset),
        bne => pinst!(pc, bne, rs1, rs2, imm=>offset),
        div => pinst!(pc, div, rd, rs1, rs2),
        divu => pinst!(pc, divu, rd, rs1, rs2),
        divuw => pinst!(pc, divuw, rd, rs1, rs2),
        divw => pinst!(pc, divw, rd, rs1, rs2),
        ebreak => pinst!(pc, ebreak),
        ecall => pinst!(pc, ecall),
        jal => pinst!(pc, jal, rd, imm=>offset),
        jalr => pinst!(pc, jalr, rd, imm(rs1)),
        lb => pinst!(pc, lb, rd, imm(rs1)),
        lbu => pinst!(pc, lbu, rd, imm(rs1)),
        ld => pinst!(pc, ld, rd, imm(rs1)),
        lh => pinst!(pc, lh, rd, imm(rs1)),
        lhu => pinst!(pc, lhu, rd, imm(rs1)),
        lui => pinst!(pc, lui, rd, imm=>imm),
        lw => pinst!(pc, lw, rd, imm(rs1)),
        lwu => pinst!(pc, lwu, rd, imm(rs1)),
        mret => pinst!(pc, mret),
        mul => pinst!(pc, mul, rd, rs1, rs2),
        mulh => pinst!(pc, mulh, rd, rs1, rs2),
        mulhsu => pinst!(pc, mulhsu, rd, rs1, rs2),
        mulhu => pinst!(pc, mulhu, rd, rs1, rs2),
        mulw => pinst!(pc, mulw, rd, rs1, rs2),
        or => pinst!(pc, or, rd, rs1, rs2),
        ori => pinst!(pc, ori, rd, rs1, imm=>imm),
        rem => pinst!(pc, rem, rd, rs1, rs2),
        remu => pinst!(pc, remu, rd, rs1, rs2),
        remuw => pinst!(pc, remuw, rd, rs1, rs2),
        remw => pinst!(pc, remw, rd, rs1, rs2),
        sb => pinst!(pc, sb, rs2, imm(rs1)),
        sd => pinst!(pc, sd, rs2, imm(rs1)),
        sh => pinst!(pc, sh, rs2, imm(rs1)),
        sll => pinst!(pc, sll, rd, rs1, rs2),
        slli => pinst!(pc, slli, rd, rs1, imm=>imm),
        slliw => pinst!(pc, slliw, rd, rs1, imm=>imm),
        sllw => pinst!(pc, sllw, rd, rs1, rs2),
        slt => pinst!(pc, slt, rd, rs1, rs2),
        slti => pinst!(pc, slti, rd, rs1, imm=>imm),
        sltiu => pinst!(pc, sltiu, rd, rs1, imm=>imm),
        sltu => pinst!(pc, sltu, rd, rs1, rs2),
        sra => pinst!(pc, sra, rd, rs1, rs2),
        srai => pinst!(pc, srai, rd, rs1, imm=>imm),
        sraiw => pinst!(pc, sraiw, rd, rs1, imm=>imm),
        sraw => pinst!(pc, sraw, rd, rs1, rs2),
        sret => pinst!(pc, sret),
        srl => pinst!(pc, srl, rd, rs1, rs2),
        srli => pinst!(pc, srli, rd, rs1, imm=>imm),
        srliw => pinst!(pc, srliw, rd, rs1, imm=>imm),
        srlw => pinst!(pc, srlw, rd, rs1, rs2),
        sub => pinst!(pc, sub, rd, rs1, rs2),
        subw => pinst!(pc, subw, rd, rs1, rs2),
        sw => pinst!(pc, sw, rs2, imm(rs1)),
        xor => pinst!(pc, xor, rd, rs1, rs2),
        xori => pinst!(pc, xori, rd, rs1, imm=>imm),
        _ => format!("Unknown inst {:?}", alu_op),
    };
    msg
}
