//! Decode phase
use crate::{
    error::{Error, Result},
    insts::*,
};
use log::{debug, error};

pub fn decode(inst: u32) -> Result<ExecInternal> {
    use crate::insts::inst_64_opcode::*;
    // Format
    let opcode = opcode(inst);

    let ex_inst = match opcode {
        LOAD => decode_load(inst),
        LOAD_FP => decode_load_fp(inst),
        MISC_MEM => decode_misc_mem(inst),
        OP_IMM => decode_op_imm(inst),
        AUIPC => decode_op_auipc(inst),
        OP_IMM_32 => decode_op_imm_32(inst),
        STORE => decode_store(inst),
        STORE_FP => decode_store_fp(inst),
        AMO => decode_amo(inst),
        OP => decode_op(inst),
        LUI => decode_lui(inst),
        OP_32 => decode_op_32(inst),
        MADD => decode_madd(inst),
        MSUB => decode_msub(inst),
        NMSUB => decode_nmsub(inst),
        NMADD => decode_nmadd(inst),
        OP_FP => decode_op_fp(inst),
        BRANCH => decode_branch(inst),
        JALR => decode_jalr(inst),
        JAL => decode_jal(inst),
        SYSTEM => decode_system(inst),
        _ => todo!(),
    };

    if let Ok(ref ex_inst) = ex_inst {
        // debug!("DECODE: {:?}", ex_inst.inst);
    } else {
        error!("ERROR DECODING: {:#x}", inst);
    }

    ex_inst

    /*
    let format = match opcode {
        OP_IMM_32 | AMO | OP | OP_32 | OP_FP => Inst32Format::R,
        MADD | MSUB | NMSUB | NMADD => Inst32Format::R4,
        LOAD | LOAD_FP | MISC_MEM | OP_IMM | JALR | SYSTEM => Inst32Format::I,
        AUIPC | LUI => Inst32Format::U,
        JAL => Inst32Format::UJ,
        STORE | STORE_FP => Inst32Format::S,
        BRANCH => Inst32Format::SB,
        _ => return Err(Error::Decode(format!("Unknown opcode {opcode}"))),
    };
    */
}

/// 0000011 LOAD: I type
fn decode_load(inst: u32) -> Result<ExecInternal> {
    let mut exec_internal = ExecInternal::default();

    let funct3 = funct3(inst);
    exec_internal.inst = match funct3 {
        0b000 => Inst64::lb,
        0b001 => Inst64::lh,
        0b010 => Inst64::lw,
        0b011 => Inst64::ld,
        0b100 => Inst64::lbu,
        0b101 => Inst64::lhu,
        0b110 => Inst64::lwu,
        _ => {
            let msg = format!("Unknown LOAD instruction funct3={funct3}");
            error!("{msg}");
            return Err(Error::Decode(msg));
        }
    };

    exec_internal.rd = rd(inst);
    exec_internal.rs1 = rs1(inst);
    exec_internal.imm = imm_I(inst);

    Ok(exec_internal)
}

/// 0000111 LOAD_FP: I type
fn decode_load_fp(_inst: u32) -> Result<ExecInternal> {
    todo!()
}

/// 0001111 MISC_MEM: I type
fn decode_misc_mem(_inst: u32) -> Result<ExecInternal> {
    todo!()
}

/// 0010011 OP_IMM: I type
fn decode_op_imm(inst: u32) -> Result<ExecInternal> {
    let mut exec_internal = ExecInternal::default();

    let funct3 = funct3(inst);
    let funct7 = funct7(inst);
    let funct6 = funct6(inst);
    exec_internal.inst = match funct3 {
        0b000 => Inst64::addi,
        0b010 => Inst64::slti,
        0b011 => Inst64::sltiu,
        0b100 => Inst64::xori,
        0b110 => Inst64::ori,
        0b111 => Inst64::andi,

        /* // RV32
        0b001 => Inst64::slli,
        0b101 => match funct7 {
            0b0000000 => Inst64::srli,
            0b0100000 => Inst64::srai,
            _ => {
                let msg = format!("Unknown OP_IMM instruction NOT srli or srai funct7={funct7}");
                error!("{msg}");
                return Err(Error::Decode(msg));
            }
        },
        */
        // RV64
        0b001 => Inst64::slli,
        0b101 => match funct6 {
            0b000000 => Inst64::srli,
            0b010000 => Inst64::srai,
            _ => {
                let msg = format!("Unknown OP_IMM instruction NOT srli or srai funct7={funct7}");
                error!("{msg}");
                return Err(Error::Decode(msg));
            }
        },
        _ => {
            let msg = format!("Unknown OP_IMM instruction funct3={funct3}");
            error!("{msg}");
            return Err(Error::Decode(msg));
        }
    };

    exec_internal.rd = rd(inst);
    exec_internal.rs1 = rs1(inst);
    exec_internal.imm = match funct3 {
        0b000 | 0b010 | 0b011 | 0b100 | 0b110 | 0b111 => imm_I(inst),
        // 0b001 | 0b101 => rs2.into(), // RV32
        0b001 | 0b101 => shift64_I(inst), // RV64
        _ => unreachable!("Should return error before control flow reaches here"),
    };

    Ok(exec_internal)
}

/// 0010111 AUIPC: U type
fn decode_op_auipc(inst: u32) -> Result<ExecInternal> {
    let mut exec_internal = ExecInternal::default();

    exec_internal.inst = Inst64::auipc;
    exec_internal.rd = rd(inst);
    exec_internal.imm = imm_U(inst);

    Ok(exec_internal)
}

/// 0011011 OP_IMM_32: R type
fn decode_op_imm_32(inst: u32) -> Result<ExecInternal> {
    let mut exec_internal = ExecInternal::default();

    let funct3 = funct3(inst);
    let funct7 = funct7(inst);
    let rs2 = rs2(inst);
    exec_internal.inst = match funct3 {
        0b000 => Inst64::addiw,
        0b001 => Inst64::slliw,
        0b101 => match funct7 {
            0b0000000 => Inst64::srliw,
            0b0100000 => Inst64::sraiw,
            _ => {
                let msg = format!("Unknown OP_IMM_32 instruction funct7={funct7}");
                error!("{msg}");
                return Err(Error::Decode(msg));
            }
        },
        _ => {
            let msg = format!("Unknown OP_IMM_32 instruction funct3={funct3}");
            error!("{msg}");
            return Err(Error::Decode(msg));
        }
    };

    exec_internal.imm = match funct3 {
        0b000 => imm_I(inst),
        0b001 | 0b101 => rs2.into(),
        _ => unreachable!("Should return error before control flow reaches here"),
    };

    exec_internal.rd = rd(inst);
    exec_internal.rs1 = rs1(inst);

    Ok(exec_internal)
}

/// 0100011 STORE: S type
fn decode_store(inst: u32) -> Result<ExecInternal> {
    let mut exec_internal = ExecInternal::default();

    let funct3 = funct3(inst);

    exec_internal.inst = match funct3 {
        0b000 => Inst64::sb,
        0b001 => Inst64::sh,
        0b010 => Inst64::sw,
        0b011 => Inst64::sd,
        _ => {
            let msg = format!("Unknown STORE instruction funct3={funct3}");
            error!("{msg}");
            return Err(Error::Decode(msg));
        }
    };

    exec_internal.rs1 = rs1(inst);
    exec_internal.rs2 = rs2(inst);
    exec_internal.imm = imm_S(inst);

    Ok(exec_internal)
}

/// 0100111 STORE_FP: S type
fn decode_store_fp(_inst: u32) -> Result<ExecInternal> {
    todo!()
}

/// 0101111 AMO: R type
fn decode_amo(_inst: u32) -> Result<ExecInternal> {
    todo!()
}

/// 0110011 OP: R type
fn decode_op(inst: u32) -> Result<ExecInternal> {
    let mut exec_internal = ExecInternal::default();

    let funct3 = funct3(inst);
    let funct7 = funct7(inst);
    exec_internal.inst = match funct3 {
        0b000 => match funct7 {
            0b0000000 => Inst64::add,
            0b0100000 => Inst64::sub,
            0b0000001 => Inst64::mul,
            _ => {
                let msg = format!("Unknown OP instruction funct3={funct3} funct7={funct7}");
                error!("{msg}");
                return Err(Error::Decode(msg));
            }
        },
        0b001 => match funct7 {
            0b0000000 => Inst64::sll,
            0b0000001 => Inst64::mulh,
            _ => {
                let msg = format!("Unknown OP instruction funct3={funct3} funct7={funct7}");
                error!("{msg}");
                return Err(Error::Decode(msg));
            }
        },
        0b010 => match funct7 {
            0b0000000 => Inst64::slt,
            0b0000001 => Inst64::mulhsu,
            _ => {
                let msg = format!("Unknown OP instruction funct3={funct3} funct7={funct7}");
                error!("{msg}");
                return Err(Error::Decode(msg));
            }
        },
        0b011 => match funct7 {
            0b0000000 => Inst64::sltu,
            0b0000001 => Inst64::mulhu,
            _ => {
                let msg = format!("Unknown OP instruction funct3={funct3} funct7={funct7}");
                error!("{msg}");
                return Err(Error::Decode(msg));
            }
        },
        0b100 => match funct7 {
            0b0000000 => Inst64::xor,
            0b0000001 => Inst64::div,
            _ => {
                let msg = format!("Unknown OP instruction funct3={funct3} funct7={funct7}");
                error!("{msg}");
                return Err(Error::Decode(msg));
            }
        },
        0b101 => match funct7 {
            0b0000000 => Inst64::srl,
            0b0100000 => Inst64::sra,
            0b0000001 => Inst64::divu,
            _ => {
                let msg = format!("Unknown OP instruction funct3={funct3} funct7={funct7}");
                error!("{msg}");
                return Err(Error::Decode(msg));
            }
        },
        0b110 => match funct7 {
            0b0000000 => Inst64::or,
            0b0000001 => Inst64::rem,
            _ => {
                let msg = format!("Unknown OP instruction funct3={funct3} funct7={funct7}");
                error!("{msg}");
                return Err(Error::Decode(msg));
            }
        },
        0b111 => match funct7 {
            0b0000000 => Inst64::and,
            0b0000001 => Inst64::remu,
            _ => {
                let msg = format!("Unknown OP instruction funct3={funct3} funct7={funct7}");
                error!("{msg}");
                return Err(Error::Decode(msg));
            }
        },
        _ => {
            let msg = format!("Unknown OP instruction funct3={funct3}");
            error!("{msg}");
            return Err(Error::Decode(msg));
        }
    };

    exec_internal.rd = rd(inst);
    exec_internal.rs1 = rs1(inst);
    exec_internal.rs2 = rs2(inst);

    Ok(exec_internal)
}

/// 0110111 LUI: U type
fn decode_lui(inst: u32) -> Result<ExecInternal> {
    let mut exec_internal = ExecInternal::default();

    exec_internal.inst = Inst64::lui;
    exec_internal.rd = rd(inst);
    exec_internal.imm = imm_U(inst);

    Ok(exec_internal)
}

/// 0111011 OP_32: R type
fn decode_op_32(inst: u32) -> Result<ExecInternal> {
    let mut exec_internal = ExecInternal::default();

    let funct3 = funct3(inst);
    let funct7 = funct7(inst);
    exec_internal.inst = match funct3 {
        0b000 => match funct7 {
            0b0000000 => Inst64::addw,
            0b0100000 => Inst64::subw,
            0b0000001 => Inst64::mulw,
            _ => {
                let msg = format!("Unknown OP_32 instruction funct3={funct3} funct7={funct7}");
                error!("{msg}");
                return Err(Error::Decode(msg));
            }
        },
        0b001 => Inst64::sllw,
        0b100 => match funct7 {
            0b0000001 => Inst64::divw,
            _ => {
                let msg = format!("Unknown OP_32 instruction funct3={funct3} funct7={funct7}");
                error!("{msg}");
                return Err(Error::Decode(msg));
            }
        },
        0b101 => match funct7 {
            0b0000000 => Inst64::srlw,
            0b0100000 => Inst64::sraw,
            0b0000001 => Inst64::divuw,
            _ => {
                let msg = format!("Unknown OP_32 instruction funct3={funct3} funct7={funct7}");
                error!("{msg}");
                return Err(Error::Decode(msg));
            }
        },
        0b110 => match funct7 {
            0b0000001 => Inst64::remw,
            _ => {
                let msg = format!("Unknown OP_32 instruction funct3={funct3} funct7={funct7}");
                error!("{msg}");
                return Err(Error::Decode(msg));
            }
        },
        0b111 => match funct7 {
            0b0000001 => Inst64::remuw,
            _ => {
                let msg = format!("Unknown OP_32 instruction funct3={funct3} funct7={funct7}");
                error!("{msg}");
                return Err(Error::Decode(msg));
            }
        },
        _ => {
            let msg = format!("Unknown OP_32 instruction funct3={funct3}");
            error!("{msg}");
            return Err(Error::Decode(msg));
        }
    };

    exec_internal.rd = rd(inst);
    exec_internal.rs1 = rs1(inst);
    exec_internal.rs2 = rs2(inst);

    Ok(exec_internal)
}

/// 1000011 MADD: R4 type
fn decode_madd(_inst: u32) -> Result<ExecInternal> {
    todo!()
}

/// 1000111 MSUB: R4 type
fn decode_msub(_inst: u32) -> Result<ExecInternal> {
    todo!()
}

/// 1001011 NMSUB: R4 type
fn decode_nmsub(_inst: u32) -> Result<ExecInternal> {
    todo!()
}

/// 1001111 NMADD: R4 type
fn decode_nmadd(_inst: u32) -> Result<ExecInternal> {
    todo!()
}

/// 1010011 OP_FP: R type
fn decode_op_fp(_inst: u32) -> Result<ExecInternal> {
    todo!()
}

/// 1100011 BRANCH: SB type
fn decode_branch(inst: u32) -> Result<ExecInternal> {
    let mut exec_internal = ExecInternal::default();

    let funct3 = funct3(inst);

    exec_internal.inst = match funct3 {
        0b000 => Inst64::beq,
        0b001 => Inst64::bne,

        0b100 => Inst64::blt,
        0b101 => Inst64::bge,
        0b110 => Inst64::bltu,
        0b111 => Inst64::bgeu,

        _ => {
            let msg = format!("Unknown BRANCH instruction funct3={funct3}");
            error!("{msg}");
            return Err(Error::Decode(msg));
        }
    };

    exec_internal.rs1 = rs1(inst);
    exec_internal.rs2 = rs2(inst);
    exec_internal.imm = imm_SB(inst);

    Ok(exec_internal)
}

/// 1100111 JALR: I type
fn decode_jalr(inst: u32) -> Result<ExecInternal> {
    let mut exec_internal = ExecInternal::default();

    let funct3 = funct3(inst);
    exec_internal.inst = match funct3 {
        0b000 => Inst64::jalr,
        _ => {
            let msg = format!("Unknown JALR instruction funct3={funct3}");
            error!("{msg}");
            return Err(Error::Decode(msg));
        }
    };

    exec_internal.rd = rd(inst);
    exec_internal.rs1 = rs1(inst);
    exec_internal.imm = imm_I(inst);

    Ok(exec_internal)
}

/// 1101111 JAL: UJ type
fn decode_jal(inst: u32) -> Result<ExecInternal> {
    let mut exec_internal = ExecInternal::default();

    exec_internal.inst = Inst64::jal;
    exec_internal.rd = rd(inst);
    exec_internal.imm = imm_UJ(inst);

    Ok(exec_internal)
}

/// 1110011 SYSTEM: I type
fn decode_system(inst: u32) -> Result<ExecInternal> {
    let mut exec_internal = ExecInternal::default();

    let funct3 = funct3(inst);
    let csr = imm_I(inst);
    exec_internal.inst = match funct3 {
        0b000 => match csr {
            0 => Inst64::ecall,
            1 => Inst64::ebreak,
            _ => {
                let msg = format!("Unknown SYSTEM E- instruction csr={csr}");
                error!("{msg}");
                return Err(Error::Decode(msg));
            }
        },
        0b001 => Inst64::csrrw,
        0b010 => Inst64::csrrs,
        0b011 => Inst64::csrrc,
        0b101 => Inst64::csrrwi,
        0b110 => Inst64::csrrsi,
        0b111 => Inst64::csrrci,
        _ => {
            let msg = format!("Unknown SYSTEM instruction funct3={funct3}");
            error!("{msg}");
            return Err(Error::Decode(msg));
        }
    };

    exec_internal.rd = rd(inst);
    exec_internal.rs1 = rs1(inst); // zimm for csrrwi, csrrsi, csrrci

    Ok(exec_internal)
}
