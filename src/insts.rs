//! Instruction definition

use std::fmt::Display;

use bitflags::bitflags;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct RtypeInst(u32);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct R4typeInst(u32);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ItypeInst(u32);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct StypeInst(u32);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SBtypeInst(u32);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct UtypeInst(u32);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct UJtypeInst(u32);

impl RtypeInst {
    #[inline(always)]
    pub fn new(inst: u32) -> Self {
        Self(inst)
    }

    #[inline(always)]
    pub fn funct7(self) -> Self {
        self.intersection(Self::mask_funct7)
    }

    #[inline(always)]
    pub fn rs2(self) -> Self {
        self.intersection(Self::mask_rs2)
    }

    #[inline(always)]
    pub fn rs1(self) -> Self {
        self.intersection(Self::mask_rs1)
    }

    #[inline(always)]
    pub fn funct3(self) -> Self {
        self.intersection(Self::mask_funct3)
    }

    #[inline(always)]
    pub fn rd(self) -> Self {
        self.intersection(Self::mask_rd)
    }

    #[inline(always)]
    pub fn opcode(self) -> Self {
        self.intersection(Self::mask_opcode)
    }
}

impl R4typeInst {
    #[inline(always)]
    pub fn new(inst: u32) -> Self {
        Self(inst)
    }

    #[inline(always)]
    pub fn rs3(self) -> u32 {
        self.intersection(Self::mask_rs3).bits()
    }

    #[inline(always)]
    pub fn funct2(self) -> u32 {
        self.intersection(Self::mask_funct2).bits()
    }

    #[inline(always)]
    pub fn rs2(self) -> u32 {
        self.intersection(Self::mask_rs2).bits()
    }

    #[inline(always)]
    pub fn rs1(self) -> u32 {
        self.intersection(Self::mask_rs1).bits()
    }

    #[inline(always)]
    pub fn rm(self) -> u32 {
        self.intersection(Self::mask_rm).bits()
    }

    #[inline(always)]
    pub fn rd(self) -> u32 {
        self.intersection(Self::mask_rd).bits()
    }

    #[inline(always)]
    pub fn opcode(self) -> u32 {
        self.intersection(Self::mask_opcode).bits()
    }
}

impl ItypeInst {
    #[inline(always)]
    pub fn new(inst: u32) -> Self {
        Self(inst)
    }

    #[inline(always)]
    pub fn imm_11_0(self) -> u32 {
        self.intersection(Self::mask_imm_11_0).bits()
    }
}

impl UtypeInst {
    #[inline(always)]
    pub fn new(inst: u32) -> Self {
        Self(inst)
    }
}

impl UJtypeInst {
    #[inline(always)]
    pub fn new(inst: u32) -> Self {
        Self(inst)
    }
}

impl StypeInst {
    #[inline(always)]
    pub fn new(inst: u32) -> Self {
        Self(inst)
    }
}

impl SBtypeInst {
    #[inline(always)]
    pub fn new(inst: u32) -> Self {
        Self(inst)
    }
}

bitflags! {
    impl RtypeInst: u32 {
        const mask_funct7    = 0b11111110_00000000_00000000_00000000;
        const mask_rs2       = 0b00000001_11110000_00000000_00000000;
        const mask_rs1       = 0b00000000_00001111_10000000_00000000;
        const mask_funct3    = 0b00000000_00000000_01110000_00000000;
        const mask_rd        = 0b00000000_00000000_00001111_10000000;
        const mask_opcode    = 0b00000000_00000000_00000000_01111111;

        /* OP_IMM_32 */
        const op_imm_32_op   = 0b00000000_00000000_00000000_00011011;
        const addiw_funct3   = 0b00000000_00000000_00000000_00000000;
    }

    impl R4typeInst: u32 {
        const mask_rs3       = 0b11111000_00000000_00000000_00000000;
        const mask_funct2    = 0b00000110_00000000_00000000_00000000;
        const mask_rs2       = 0b00000001_11110000_00000000_00000000;
        const mask_rs1       = 0b00000000_00001111_10000000_00000000;
        const mask_rm        = 0b00000000_00000000_01110000_00000000;
        const mask_rd        = 0b00000000_00000000_00001111_10000000;
        const mask_opcode    = 0b00000000_00000000_00000000_01111111;
    }

    impl ItypeInst: u32 {
        const mask_imm_11_0  = 0b11111111_11110000_00000000_00000000;
        const mask_rs1       = 0b00000000_00001111_10000000_00000000;
        const mask_funct3    = 0b00000000_00000000_01110000_00000000;
        const mask_rd        = 0b00000000_00000000_00001111_10000000;
        const mask_opcode    = 0b00000000_00000000_00000000_01111111;
    }

    impl StypeInst: u32 {
        const mask_imm_11_5  = 0b11111110_00000000_00000000_00000000;
        const mask_rs2       = 0b00000001_11110000_00000000_00000000;
        const mask_rs1       = 0b00000000_00001111_10000000_00000000;
        const mask_funct3    = 0b00000000_00000000_01110000_00000000;
        const mask_imm_4_0   = 0b00000000_00000000_00001111_10000000;
        const mask_opcode    = 0b00000000_00000000_00000000_01111111;
    }

    impl SBtypeInst: u32 {
        const mask_imm_12    = 0b10000000_00000000_00000000_00000000;
        const mask_imm_10_5  = 0b01111110_00000000_00000000_00000000;
        const mask_rs2       = 0b00000001_11110000_00000000_00000000;
        const mask_rs1       = 0b00000000_00001111_10000000_00000000;
        const mask_funct3    = 0b00000000_00000000_01110000_00000000;
        const mask_imm_4_1   = 0b00000000_00000000_00001111_00000000;
        const mask_imm_11    = 0b00000000_00000000_00000000_10000000;
        const mask_opcode    = 0b00000000_00000000_00000000_01111111;
    }

    impl UtypeInst: u32 {
        const mask_imm_31_12 = 0b11111111_11111111_11110000_00000000;
        const mask_rd        = 0b00000000_00000000_00001111_10000000;
        const mask_opcode    = 0b00000000_00000000_00000000_01111111;
    }

    impl UJtypeInst: u32 {
        const mask_imm_20    = 0b10000000_00000000_00000000_00000000;
        const mask_imm_10_1  = 0b01111111_11100000_00000000_00000000;
        const mask_imm_11    = 0b00000000_00010000_00000000_00000000;
        const mask_imm_19_12 = 0b00000000_00001111_11110000_00000000;
        const mask_rd        = 0b00000000_00000000_00001111_10000000;
        const mask_opcode    = 0b00000000_00000000_00000000_01111111;
    }
}

pub const /*OPCODE  */ OPCODE_MASK: u32 = 0b0000000_00000_00000_000_00000_1111111;
pub const /*RD IMM  */ RD_MASK:     u32 = 0b0000000_00000_00000_000_11111_0000000;
pub const /*FUNCT3  */ FUNCT3_MASK: u32 = 0b0000000_00000_00000_111_00000_0000000;
pub const /*RS1     */ RS1_MASK:    u32 = 0b0000000_00000_11111_000_00000_0000000;
pub const /*RS2     */ RS2_MASK:    u32 = 0b0000000_11111_00000_000_00000_0000000;
pub const /*RS3     */ RS3_MASK:    u32 = 0b1111100_00000_00000_000_00000_0000000;
pub const /*FUNCT2  */ FUNCT2_MASK: u32 = 0b0000011_00000_00000_000_00000_0000000;
pub const /*FUNCT7  */ FUNCT7_MASK: u32 = 0b1111111_00000_00000_000_00000_0000000;
pub const /*FUNCT6  */ FUNCT6_MASK: u32 = 0b1111110_00000_00000_000_00000_0000000;
pub const /*IMM_11_0*/ IMM110_MASK: u32 = 0b1111111_11111_00000_000_00000_0000000;
pub const /*IMM_HIGH*/ IMMHIGH_MASK:u32 = 0b1111111_11111_11111_111_00000_0000000;
pub const /*SHIFT64 */ SHIFT64_MASK:u32 = 0b0000001_11111_00000_000_00000_0000000;

pub const RD_SHIFT: usize = 7;
pub const FUNCT3_SHIFT: usize = 12;
pub const RS1_SHIFT: usize = 15;
pub const RS2_SHIFT: usize = 20;
pub const RS3_SHIFT: usize = 27;
pub const FUNCT2_SHIFT: usize = 25;
pub const FUNCT7_SHIFT: usize = 25;
pub const FUNCT6_SHIFT: usize = 26;
pub const IMM110_SHIFT: usize = 20;
pub const IMMHIGH_SHIFT: usize = 12;
pub const SHIFT64_SHIFT: usize = 20;

pub fn opcode(inst: u32) -> u32 {
    inst & OPCODE_MASK
}

pub fn rd(inst: u32) -> u8 {
    ((inst & RD_MASK) >> RD_SHIFT) as u8
}

pub fn rs1(inst: u32) -> u8 {
    ((inst & RS1_MASK) >> RS1_SHIFT) as u8
}

pub fn rs2(inst: u32) -> u8 {
    ((inst & RS2_MASK) >> RS2_SHIFT) as u8
}

pub fn rs3(inst: u32) -> u8 {
    ((inst & RS3_MASK) >> RS3_SHIFT) as u8
}

pub fn funct3(inst: u32) -> u8 {
    ((inst & FUNCT3_MASK) >> FUNCT3_SHIFT) as u8
}

pub fn funct7(inst: u32) -> u8 {
    ((inst & FUNCT7_MASK) >> FUNCT7_SHIFT) as u8
}

pub fn funct2(inst: u32) -> u8 {
    ((inst & FUNCT2_MASK) >> FUNCT2_SHIFT) as u8
}

pub fn funct6(inst: u32) -> u8 {
    ((inst & FUNCT6_MASK) >> FUNCT6_SHIFT) as u8
}

#[allow(non_snake_case)]
pub fn shift64_I(inst: u32) -> u64 {
    ((inst & SHIFT64_MASK) >> SHIFT64_SHIFT) as u64
}

#[allow(non_snake_case)]
pub fn imm_I(inst: u32) -> u64 {
    ((inst & IMM110_MASK) >> IMM110_SHIFT) as u64
}

#[allow(non_snake_case)]
pub fn imm_S(inst: u32) -> u64 {
    let imm_11_5 = ((inst & FUNCT7_MASK) >> FUNCT7_SHIFT) << 5;
    let imm_4_0 = (inst & RD_MASK) >> RD_SHIFT;
    (imm_11_5 | imm_4_0) as u64
}

#[allow(non_snake_case)]
pub fn imm_SB(inst: u32) -> u64 {
    let imm_12_10_5 = (inst & FUNCT7_MASK) >> FUNCT7_SHIFT;
    let imm_4_1_11 = (inst & RD_MASK) >> RD_SHIFT;

    let imm_4_1 = imm_4_1_11 & 0b11110_u32;
    let bit_11 = (imm_4_1_11 & 1) << 11;

    let bit_12 = (imm_12_10_5 & 0b1000000_u32) << 6; // >> 6 << 12
    let imm_10_5 = (imm_12_10_5 & 0b0111111_u32) << 5;

    (bit_12 | bit_11 | imm_10_5 | imm_4_1) as u64
}

#[allow(non_snake_case)]
pub fn imm_U(inst: u32) -> u64 {
    ((inst & IMMHIGH_MASK) >> IMMHIGH_SHIFT) as u64
}

#[allow(non_snake_case)]
pub fn imm_UJ(inst: u32) -> u64 {
    let imm_20_10_1_11 = inst & (FUNCT7_MASK | RS2_MASK);
    let imm_19_12 = inst & (RS1_MASK | FUNCT3_MASK);

    let bit_20 = (imm_20_10_1_11 & 0b1000000_00000_00000_000_00000_0000000) >> 11; // >> 31 << 20
    let bit_11 = (imm_20_10_1_11 & 0b0000000_00001_00000_000_00000_0000000) >> 9; // >> 20 << 11
    let imm_10_1 = (imm_20_10_1_11 & 0b0111111_11110_00000_000_00000_0000000) >> 20; // >> 21 << 1

    (bit_20 | imm_19_12 | bit_11 | imm_10_1) as u64
}

#[allow(non_upper_case_globals)]
pub mod inst_64_opcode {
    pub const /* I  */ LOAD: u32 = 0b00_000_11;
    pub const /* I  */ LOAD_FP: u32 = 0b00_001_11;
    pub const custom_0: u32 = 0b00_010_11; /* used by custom extensions */
    pub const /* I  */ MISC_MEM: u32 = 0b00_011_11;
    pub const /* I  */ OP_IMM: u32 = 0b00_100_11;
    pub const /* U  */ AUIPC: u32 = 0b00_101_11;
    pub const /* R  */ OP_IMM_32: u32 = 0b00_110_11;
    pub const inst48b_0: u32 = 0b00_111_11; /* reserved for 48 bits inst */

    pub const /* S  */ STORE: u32 = 0b01_000_11;
    pub const /* S  */ STORE_FP: u32 = 0b01_001_11;
    pub const custom_1: u32 = 0b01_010_11; /* used by custom extensions */
    pub const /* R  */ AMO:  u32 = 0b01_011_11;
    pub const /* R  */ OP: u32 = 0b01_100_11;
    pub const /* U  */ LUI: u32 = 0b01_101_11;
    pub const /* R  */ OP_32: u32 = 0b01_110_11;
    pub const inst64b: u32 = 0b01_111_11; /* reserved for 64 bits inst */

    pub const /* R4 */ MADD: u32 = 0b10_000_11;
    pub const /* R4 */ MSUB: u32 = 0b10_001_11;
    pub const /* R4 */ NMSUB: u32 = 0b10_010_11;
    pub const /* R4 */ NMADD: u32 = 0b10_011_11;
    pub const /* R  */ OP_FP: u32 = 0b10_100_11;
    pub const reserved_0: u32 = 0b10_101_11; /* reserved for future standard extensions */
    pub const custom_2_rv128: u32 = 0b10_110_11; /* used by custom extensions under RV32/RV64, reserved for 128 bits standard extensions */
    pub const inst48b_1: u32 = 0b10_111_11; /* reserved for 48 bits inst */

    pub const /* SB */ BRANCH: u32 = 0b11_000_11;
    pub const /* I  */ JALR: u32 = 0b11_001_11;
    pub const reserved_1: u32 = 0b11_010_11; /* reserved for future standard extensions */
    pub const /* UJ */ JAL: u32 = 0b11_011_11;
    pub const /* I  */ SYSTEM: u32 = 0b11_100_11;
    pub const reserved_2: u32 = 0b11_101_11; /* reserved for future standard extensions */
    pub const custom_3_rv128: u32 = 0b11_110_11; /* used by custom extensions under RV32/RV64, reserved for 128 bits standard extensions */
    pub const inst80b: u32 = 0b11_111_11; /* reserved for 80 bits inst */
}

pub enum Inst64Format {
    R,
    R4,
    I,
    U,
    UJ,
    S,
    SB,
    Unknown,
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Inst64 {
    unknown,

    add,
    addi,
    addiw,
    addw,
    and,
    andi,
    auipc,
    beq,
    bge,
    bgeu,
    blt,
    bltu,
    bne,

    csrrc,
    csrrci,
    csrrs,
    csrrsi,
    csrrw,
    csrrwi,

    div,
    divu,
    divuw,
    divw,
    ebreak,
    ecall,

    fence,
    fence_i,

    jal,
    jalr,

    lb,
    lbu,
    ld,
    lh,
    lhu,
    lui,
    lw,
    lwu,
    mret,
    mul,
    mulh,
    mulhsu,
    mulhu,
    mulw,

    or,
    ori,
    rem,
    remu,
    remuw,
    remw,

    sb,
    sd,
    sh,
    sll,
    slli,
    slliw,
    sllw,
    slt,
    slti,
    sltiu,
    sltu,
    sra,
    srai,
    sraiw,
    sraw,
    sret,
    srl,
    srli,
    srliw,
    srlw,
    sub,
    subw,
    sw,

    wfi,

    xor,
    xori,
}

#[macro_export]
macro_rules! pinst {
    // SYSTEM
    ($pc:ident, $inst:tt) => {
        format!("{:8x}:\t{}", $pc, stringify!($inst))
    };
    ($pc:ident, $inst:tt, $t1:ident) => {
        format!("{:8x}:\t{}\t{}", $pc, stringify!($inst), REGNAME[$t1 as usize])
    };
    ($pc:ident, $inst:tt, $t1:ident, $t2:ident) => {
        format!(
            "{:8x}:\t{}\t{},{}",
            $pc,
            stringify!($inst),
            REGNAME[$t1 as usize],
            REGNAME[$t2 as usize]
        )
    };
    // OP, OP_32
    ($pc:ident, $inst:tt, $rd:ident, $rs1:ident, $rs2:ident) => {
        format!(
            "{:8x}:\t{}\t{},{},{}",
            $pc,
            stringify!($inst),
            REGNAME[$rd as usize],
            REGNAME[$rs1 as usize],
            REGNAME[$rs2 as usize]
        )
    };
    // BRANCH
    ($pc:ident, $inst:tt, $rs1:ident, $rs2:ident, $offset:ident=>offset) => {
        format!(
            "{:8x}:\t{}\t{},{},{:x}",
            $pc,
            stringify!($inst),
            REGNAME[$rs1 as usize],
            REGNAME[$rs2 as usize],
            $offset
        )
    };
    // JAL
    ($pc:ident, $inst:tt, $rd:ident, $offset:ident=>offset) => {
        format!(
            "{:8x}:\t{}\t{},{:x}",
            $pc,
            stringify!($inst),
            REGNAME[$rd as usize],
            $offset
        )
    };
    // AUIPC
    ($pc:ident, $inst:tt, $t1:ident, $imm:ident=>imm) => {
        format!(
            "{:8x}:\t{}\t{},{:#x}",
            $pc,
            stringify!($inst),
            REGNAME[$t1 as usize],
            $imm
        )
    };
    // OP_IMM, OP_IMM_32
    ($pc:ident, $inst:tt, $t1:ident, $t2:ident, $imm:ident=>imm) => {
        format!(
            "{:8x}:\t{}\t{},{},{}",
            $pc,
            stringify!($inst),
            REGNAME[$t1 as usize],
            REGNAME[$t2 as usize],
            $imm
        )
    };
    // MEM
    ($pc:ident, $inst:tt, $t1:ident, $imm:ident($t2:ident)) => {
        format!(
            "{:8x}:\t{}\t{},{:x}({})",
            $pc,
            stringify!($inst),
            REGNAME[$t1 as usize],
            $imm,
            REGNAME[$t2 as usize],
        )
    };
}

pub struct ExecInternal {
    pub inst: Inst64,
    pub rs1: u8,   // source register 1 index
    pub rs2: u8,   // source register 2 index
    pub rs3: u8,   // source register 3 index
    pub rd: u8,    // destination register index
    pub imm: u64,  // immediate number, which is real number
    pub pc: u64,   // new pc calculated
    pub src1: u64, // oprand 1
    pub src2: u64, // oprand 2
    pub src3: u64, // oprand 3
}

impl Default for ExecInternal {
    fn default() -> Self {
        ExecInternal {
            inst: Inst64::unknown,
            rs1: 0,
            rs2: 0,
            rs3: 0,
            rd: 0,
            imm: 0,
            pc: 0,
            src1: 0,
            src2: 0,
            src3: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_shift_right() {
        let num: u32 = 0b1011011_00000_00000_000_00000_0000000;
        let get: u32 = (num & FUNCT7_MASK) >> FUNCT7_SHIFT;
        assert_eq!(get, 0b0000000_00000_00000_000_00000_1011011);
    }
}
