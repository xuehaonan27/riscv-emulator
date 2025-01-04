#![allow(unused)]
//! Instruction definition
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

/// Extract `opcode` field from the instruction.
pub fn opcode(inst: u32) -> u32 {
    inst & OPCODE_MASK
}

/// Extract `rd` field from the instruction.
pub fn rd(inst: u32) -> u8 {
    ((inst & RD_MASK) >> RD_SHIFT) as u8
}

/// Extract `rs1` field from the instruction.
pub fn rs1(inst: u32) -> u8 {
    ((inst & RS1_MASK) >> RS1_SHIFT) as u8
}

/// Extract `rs2` field from the instruction.
pub fn rs2(inst: u32) -> u8 {
    ((inst & RS2_MASK) >> RS2_SHIFT) as u8
}

/// Extract `rs3` field from the instruction.
pub fn rs3(inst: u32) -> u8 {
    ((inst & RS3_MASK) >> RS3_SHIFT) as u8
}

/// Extract `funct3` field from the instruction.
pub fn funct3(inst: u32) -> u8 {
    ((inst & FUNCT3_MASK) >> FUNCT3_SHIFT) as u8
}

/// Extract `funct7` field from the instruction.
pub fn funct7(inst: u32) -> u8 {
    ((inst & FUNCT7_MASK) >> FUNCT7_SHIFT) as u8
}

/// Extract `funct2` field from the instruction.
pub fn funct2(inst: u32) -> u8 {
    ((inst & FUNCT2_MASK) >> FUNCT2_SHIFT) as u8
}

/// Extract `funct6` field from the instruction.
pub fn funct6(inst: u32) -> u8 {
    ((inst & FUNCT6_MASK) >> FUNCT6_SHIFT) as u8
}

/// Extract `shamt` field from the instruction (RV64).
#[allow(non_snake_case)]
pub fn shift64_I(inst: u32) -> u64 {
    ((inst & SHIFT64_MASK) >> SHIFT64_SHIFT) as u64
}

/// Extract `imm` field from I type instruction.
#[allow(non_snake_case)]
pub fn imm_I(inst: u32) -> u64 {
    ((inst & IMM110_MASK) >> IMM110_SHIFT) as u64
}

/// Extract `imm` field from S type instruction.
#[allow(non_snake_case)]
pub fn imm_S(inst: u32) -> u64 {
    let imm_11_5 = ((inst & FUNCT7_MASK) >> FUNCT7_SHIFT) << 5;
    let imm_4_0 = (inst & RD_MASK) >> RD_SHIFT;
    (imm_11_5 | imm_4_0) as u64
}

/// Extract `imm` field from SB type instruction.
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

/// Extract `imm` field from U type instruction.
#[allow(non_snake_case)]
pub fn imm_U(inst: u32) -> u64 {
    ((inst & IMMHIGH_MASK) >> IMMHIGH_SHIFT) as u64
}

/// Extract `imm` field from UJ type instruction.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inst64 {
    noop,

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
        format!(
            "{:8x}:\t{}\t{}",
            $pc,
            stringify!($inst),
            crate::core::reg::REGNAME[$t1 as usize]
        )
    };
    ($pc:ident, $inst:tt, $t1:ident, $t2:ident) => {
        format!(
            "{:8x}:\t{}\t{},{}",
            $pc,
            stringify!($inst),
            crate::core::reg::REGNAME[$t1 as usize],
            crate::core::reg::REGNAME[$t2 as usize]
        )
    };
    // OP, OP_32
    ($pc:ident, $inst:tt, $rd:ident, $rs1:ident, $rs2:ident) => {
        format!(
            "{:8x}:\t{}\t{},{},{}",
            $pc,
            stringify!($inst),
            crate::core::reg::REGNAME[$rd as usize],
            crate::core::reg::REGNAME[$rs1 as usize],
            crate::core::reg::REGNAME[$rs2 as usize]
        )
    };
    // BRANCH
    ($pc:ident, $inst:tt, $rs1:ident, $rs2:ident, $offset:ident=>offset) => {
        format!(
            "{:8x}:\t{}\t{},{},{:x}",
            $pc,
            stringify!($inst),
            crate::core::reg::REGNAME[$rs1 as usize],
            crate::core::reg::REGNAME[$rs2 as usize],
            $offset
        )
    };
    // JAL
    ($pc:ident, $inst:tt, $rd:ident, $offset:ident=>offset) => {
        format!(
            "{:8x}:\t{}\t{},{:x}",
            $pc,
            stringify!($inst),
            crate::core::reg::REGNAME[$rd as usize],
            $offset
        )
    };
    // AUIPC
    ($pc:ident, $inst:tt, $t1:ident, $imm:ident=>imm) => {
        format!(
            "{:8x}:\t{}\t{},{:#x}",
            $pc,
            stringify!($inst),
            crate::core::reg::REGNAME[$t1 as usize],
            $imm
        )
    };
    // OP_IMM, OP_IMM_32
    ($pc:ident, $inst:tt, $t1:ident, $t2:ident, $imm:ident=>imm) => {
        format!(
            "{:8x}:\t{}\t{},{},{}",
            $pc,
            stringify!($inst),
            crate::core::reg::REGNAME[$t1 as usize],
            crate::core::reg::REGNAME[$t2 as usize],
            $imm
        )
    };
    // MEM
    ($pc:ident, $inst:tt, $t1:ident, $imm:ident($t2:ident)) => {
        format!(
            "{:8x}:\t{}\t{},{}({})",
            $pc,
            stringify!($inst),
            crate::core::reg::REGNAME[$t1 as usize],
            $imm,
            crate::core::reg::REGNAME[$t2 as usize],
        )
    };
}

pub struct ExecInternal {
    pub raw_inst: u32,
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
            raw_inst: 0,
            inst: Inst64::noop,
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

pub const BYTE_BITWIDTH: u8 = 8;
pub const HALF_BITWIDTH: u8 = 16;
pub const WORD_BITWIDTH: u8 = 32;

pub const I_TYPE_IMM_BITWIDTH: u8 = 12; // imm[11:0]
pub const S_TYPE_IMM_BITWIDTH: u8 = 12; // imm[11:5] imm[4:0]
pub const B_TYPE_IMM_BITWIDTH: u8 = 13; // imm[12|10:5] imm[4:1|11]
pub const U_TYPE_IMM_BITWIDTH: u8 = 20; // imm[31:12]
pub const J_TYPE_IMM_BITWIDTH: u8 = 21; // imm[20|10:1|11|19:12]

/// Signed-extent to 64 bit.
/// Immediate number could be 12-bit, 13-bit, 20-bit, 21bit.
pub fn sext(imm: u64, bit_width: u8) -> i64 {
    assert!(bit_width < 64, "bit_width too long");
    // Suppose bit_width = 5. Highest bit is sign-bit.
    // Numbers for example
    // Signed Imm:   00011101
    // Unsigned Imm: 00001101

    // Sign bit mask.
    // 00010000
    let sign_bit_mask: i64 = 1 << (bit_width - 1);

    // Mask for the immediate.  Typed as i64 to perform 2-complement substraction.
    // 00011111
    let mask: i64 = (1i64 << bit_width) - 1;

    // Get sign-bit for imm.
    // Signed Imm:   00011101 => 00010000
    // Unsigned Imm: 00001101 => 00000000
    let sign_bit: i64 = (imm as i64) & sign_bit_mask;

    // Get extended bits.
    // Signed Imm:   00010000 => 00100000 => 00011111 => 11100000
    // Unsigned Imm: 00000000 => 00000000 => 11111111 => 00000000
    let extended_bits: i64 = !((sign_bit << 1) - 1);

    // Final result.
    // Signed Imm:   00011101 | 11100000 => 11111101
    // Unsigned Imm: 00001101 | 00000000 => 00001101
    ((imm as i64) & mask) | extended_bits
}

#[inline(always)]
pub fn trunc_to_32_bit(x: u64) -> u64 {
    const MASK: u64 = 0x00000000_FFFFFFFF;
    x & MASK
}

#[inline(always)]
pub fn trunc_to_16_bit(x: u64) -> u64 {
    const MASK: u64 = 0x00000000_0000FFFF;
    x & MASK
}

#[inline(always)]
pub fn trunc_to_8_bit(x: u64) -> u64 {
    const MASK: u64 = 0x00000000_000000FF;
    x & MASK
}

#[inline(always)]
pub fn trunc_to_6_bit(x: u64) -> u64 {
    const MASK: u64 = 0x00000000_0000003F;
    x & MASK
}

#[inline(always)]
pub fn trunc_to_5_bit(x: u64) -> u64 {
    const MASK: u64 = 0x00000000_0000001F;
    x & MASK
}

#[inline(always)]
pub fn trunc_to_5_bit_and_check(x: u64) -> (u64, bool) {
    const MASK: u64 = 0x00000000_0000001F;
    const LEGAL_MASK: u64 = 0x00000000_00000020;
    (x & MASK, (x & LEGAL_MASK) == 0)
}

#[inline(always)]
pub fn get_high_64_bit(x: u128) -> u64 {
    const MASK: u128 = 0xFFFFFFFF_FFFFFFFF_00000000_00000000;
    ((x & MASK) >> 64) as u64
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
