//! Mono-core CPU

use std::{
    fmt::Display,
    ops::{BitAnd, BitOr, BitXor},
};

use log::{error, info, trace};

use crate::{
    callstack::CallStack,
    check,
    core::{
        insts::*,
        reg::{ProgramCounter, RegisterFile},
        vm::VirtualMemory,
    },
    elf::LoadElfInfo,
    error::{Error, Exception, Result},
    pinst,
};

use super::decode::decode;

pub struct CPU<'a> {
    // indicate whether the CPU is running
    running: bool,

    // General purpose register file
    reg_file: RegisterFile,

    // Program counter (PC) which is not included in general purpose register file.
    pc: ProgramCounter,

    // Reference to virtual memory
    vm: &'a mut VirtualMemory,

    // Reference to call stack
    callstack: &'a mut CallStack<'a>,

    // Itrace switch
    itrace: bool,
}

impl<'a> CPU<'a> {
    pub fn new(
        vm: &'a mut VirtualMemory,
        callstack: &'a mut CallStack<'a>,
        itrace: bool,
    ) -> CPU<'a> {
        // x0 already set to 0
        let reg_file = RegisterFile::empty();
        let pc = ProgramCounter::new();

        CPU {
            running: false,
            reg_file,
            pc,
            vm,
            callstack,
            itrace,
        }
    }

    /// Initialize CPU with ELF info
    pub fn init_elfinfo_64(&mut self, info: &LoadElfInfo) {
        // make sure we are running a ELF64 executable
        assert!(info.is_64_bit());

        self.reg_file.init_elfinfo_64(info);

        // Load program counter
        self.pc.write(info.entry_point());
    }

    /// Run the cpu.
    /// steps: how many steps should be run, [`None`] means run until end or
    /// exception raised.
    pub fn cpu_exec(&mut self, steps: Option<i32>) -> Result<()> {
        self.running = true;
        let mut i = 0;

        while self.running {
            if steps.is_some_and(|n| i >= n) {
                break;
            }
            self.exec_once()?;
            i += 1;
        }

        Ok(())
    }

    ///  Simulate on instruction level
    pub fn exec_once(&mut self) -> Result<()> {
        // Fetch
        let pc = self.pc.read();
        let inst = self.fetch_inst(pc);

        // Decode
        let exec_internal = decode(inst)?;

        // Execute
        self.exec_inst(exec_internal)?;

        // Memory

        // Write Back

        Ok(())
    }

    pub fn fetch_inst(&mut self, pc: u64) -> u32 {
        check!(pc != 0, "PC is zero.");
        self.vm.fetch_inst(pc as usize)
    }

    /// Simulate 5-stage in-order CPU
    #[allow(unused)]
    pub fn exec_microarchitecture(&mut self) -> Result<()> {
        // Fetch

        // Decode

        // Execute

        // Memory Access

        // Write Back
        todo!()
    }
}

impl<'a> CPU<'a> {
    /// Instruction level simulation
    pub fn exec_inst(&mut self, mut exec_itrnl: ExecInternal) -> Result<()> {
        use crate::core::insts::Inst64;
        // get pc
        exec_itrnl.pc = self.pc.read();
        let pc = exec_itrnl.pc; // read pc into intermediate register
        let mut use_new_pc = false;

        // Get source from register
        let reg_file = &mut self.reg_file;
        let src1 = reg_file.read(exec_itrnl.rs1);
        let src2 = reg_file.read(exec_itrnl.rs2);
        #[allow(unused)]
        let src3 = reg_file.read(exec_itrnl.rs3); // TODO: float instructions
        let imm = exec_itrnl.imm;

        let rs1 = exec_itrnl.rs1;
        let rs2 = exec_itrnl.rs2;
        #[allow(unused)]
        let rs3 = exec_itrnl.rs3; // TODO: float instructions
        let rd = exec_itrnl.rd;

        // Calculation
        match exec_itrnl.inst {
            Inst64::add => {
                // R x[rd] = x[rs1] + x[rs2]
                if self.itrace {
                    trace!("{}", pinst!(pc, add, rd, rs1, rs2));
                }
                let result = src1.wrapping_add(src2); // ignore overflow
                reg_file.write(rd, result);
            }
            Inst64::addi => {
                if self.itrace {
                    trace!("{}", pinst!(pc, addi, rd, rs1, imm=>imm));
                }
                // I x[rd] = x[rs1] + sext(immediate)
                let result = src1.wrapping_add(sext(imm, I_TYPE_IMM_BITWIDTH) as u64);
                reg_file.write(rd, result);
            }
            Inst64::addiw => {
                // I x[rd] = sext((x[rs1] + sext(immediate))[31:0])
                if self.itrace {
                    trace!("{}", pinst!(pc, addiw, rd, rs1, imm=>imm));
                }
                let result = src1.wrapping_add(sext(imm, I_TYPE_IMM_BITWIDTH) as u64);
                let result = sext(trunc_to_32_bit(result), WORD_BITWIDTH);
                reg_file.write(rd, result as u64);
            }
            Inst64::addw => {
                // R x[rd] = sext((x[rs1] + x[rs2])[31:0])
                if self.itrace {
                    trace!("{}", pinst!(pc, addw, rd, rs1, rs2));
                }
                let result = src1.wrapping_add(src2);
                let result = sext(trunc_to_32_bit(result), WORD_BITWIDTH);
                reg_file.write(rd, result as u64);
            }
            Inst64::and => {
                // R x[rd] = x[rs1] & x[rs2]
                if self.itrace {
                    trace!("{}", pinst!(pc, and, rd, rs1, rs2));
                }
                let result = src1.bitand(src2);
                reg_file.write(rd, result);
            }
            Inst64::andi => {
                // I x[rd] = x[rs1] & sext(immediate)
                if self.itrace {
                    trace!("{}", pinst!(pc, andi, rd, rs1, imm=>imm));
                }
                let result = src1.bitand(sext(imm, I_TYPE_IMM_BITWIDTH) as u64);
                reg_file.write(rd, result);
            }
            Inst64::auipc => {
                // U x[rd] = pc + sext(immediate[31:12] << 12)
                if self.itrace {
                    trace!("{}", pinst!(pc, auipc, rd, imm=>imm));
                }
                let result = pc.wrapping_add((sext(imm, U_TYPE_IMM_BITWIDTH) as u64) << 12);
                reg_file.write(rd, result);
            }
            Inst64::beq => {
                // B if (rs1 == rs2) pc += sext(offset)
                if self.itrace {
                    trace!("{}", pinst!(pc, beq, rs1, rs2, imm=>offset));
                }
                if src1 == src2 {
                    exec_itrnl.pc = pc.wrapping_add(sext(imm, B_TYPE_IMM_BITWIDTH) as u64);
                    use_new_pc = true;
                }
            }
            Inst64::bge => {
                // B if (rs1 >= rs2) pc += sext(offset)
                if self.itrace {
                    trace!("{}", pinst!(pc, bge, rs1, rs2, imm=>offset));
                }
                if (src1 as i64) >= (src2 as i64) {
                    exec_itrnl.pc = pc.wrapping_add(sext(imm, B_TYPE_IMM_BITWIDTH) as u64);
                    use_new_pc = true;
                }
            }
            Inst64::bgeu => {
                // B if (rs1 >= rs2) pc += sext(offset)
                if self.itrace {
                    trace!("{}", pinst!(pc, bgeu, rs1, rs2, imm=>offset));
                }
                if (src1 as u64) >= (src2 as u64) {
                    exec_itrnl.pc = pc.wrapping_add(sext(imm, B_TYPE_IMM_BITWIDTH) as u64);
                    use_new_pc = true;
                }
            }
            Inst64::blt => {
                // B if (rs1 < rs2) pc += sext(offset)
                if self.itrace {
                    trace!("{}", pinst!(pc, blt, rs1, rs2, imm=>offset));
                }
                if (src1 as i64) < (src2 as i64) {
                    exec_itrnl.pc = pc.wrapping_add(sext(imm, B_TYPE_IMM_BITWIDTH) as u64);
                    use_new_pc = true;
                }
            }
            Inst64::bltu => {
                // B if (rs1 < rs2) pc += sext(offset)
                if self.itrace {
                    trace!("{}", pinst!(pc, bltu, rs1, rs2, imm=>offset));
                }
                if (src1 as u64) < (src2 as u64) {
                    exec_itrnl.pc = pc.wrapping_add(sext(imm, B_TYPE_IMM_BITWIDTH) as u64);
                    use_new_pc = true;
                }
            }
            Inst64::bne => {
                // B if (rs1 != rs2) pc += sext(offset)
                if self.itrace {
                    trace!("{}", pinst!(pc, bne, rs1, rs2, imm=>offset));
                }
                if src1 != src2 {
                    exec_itrnl.pc = pc.wrapping_add(sext(imm, B_TYPE_IMM_BITWIDTH) as u64);
                    use_new_pc = true;
                }
            }

            Inst64::div => {
                // R x[rd] = x[rs1] ÷s x[rs2]
                if self.itrace {
                    trace!("{}", pinst!(pc, div, rd, rs1, rs2));
                }
                if src2 == 0 {
                    return Err(Error::Exception(Exception::DividedByZero));
                }
                let result = (src1 as i64).wrapping_div(src2 as i64);
                reg_file.write(rd, result as u64);
            }
            Inst64::divu => {
                // R x[rd] = x[rs1] ÷u x[rs2]
                if self.itrace {
                    trace!("{}", pinst!(pc, divu, rd, rs1, rs2));
                }
                if src2 == 0 {
                    return Err(Error::Exception(Exception::DividedByZero));
                }
                let result = src1.wrapping_div(src2);
                reg_file.write(rd, result);
            }
            Inst64::divuw => {
                // R x[rd] = sext(x[rs1][31:0] ÷u x[rs2][31:0])
                if self.itrace {
                    trace!("{}", pinst!(pc, divuw, rd, rs1, rs2));
                }
                if trunc_to_32_bit(src2) == 0 {
                    return Err(Error::Exception(Exception::DividedByZero));
                }
                let result = trunc_to_32_bit(src1).wrapping_div(trunc_to_32_bit(src2));
                let result = sext(trunc_to_32_bit(result), WORD_BITWIDTH);
                reg_file.write(rd, result as u64);
            }
            Inst64::divw => {
                // R x[rd] = sext(x[rs1][31:0] ÷s x[rs2][31:0])
                if self.itrace {
                    trace!("{}", pinst!(pc, divw, rd, rs1, rs2));
                }
                if trunc_to_32_bit(src2) == 0 {
                    return Err(Error::Exception(Exception::DividedByZero));
                }
                let result =
                    (trunc_to_32_bit(src1) as i32).wrapping_div(trunc_to_32_bit(src2) as i32);
                let result = sext(trunc_to_32_bit(result as u64), WORD_BITWIDTH);
                reg_file.write(rd, result as u64);
            }
            Inst64::ebreak => {
                // I RaiseException(Breakpoint)
                // Temporary implementation: return exit code at x10.
                if self.itrace {
                    trace!("{}", pinst!(pc, ebreak));
                }
                let x10 = reg_file.read(10);
                self.halt(pc, x10); // HALT at current code.
                let msg = format!("ebreak at {:#x}, code {}", pc, x10);
                return Err(Error::Execute(msg)); // Simulate exception
            }
            Inst64::ecall => {
                todo!("ecall");
            }

            Inst64::jal => {
                // J x[rd] = pc+4; pc += sext(offset)
                if self.itrace {
                    trace!("{}", pinst!(pc, jal, rd, imm=>offset));
                }
                reg_file.write(rd, pc + 4); // rd default to x1
                exec_itrnl.pc = pc.wrapping_add(sext(imm, J_TYPE_IMM_BITWIDTH) as u64);

                // call
                let target_pc = exec_itrnl.pc;
                self.callstack.call(pc, target_pc);

                use_new_pc = true;
            }
            Inst64::jalr => {
                // I t=pc+4; pc=(x[rs1]+sext(offset))&∼1; x[rd]=t
                if self.itrace {
                    trace!("{}", pinst!(pc, jalr, rd, imm(rs1)));
                }

                // ret
                if exec_itrnl.raw_inst == 0x00008067 {
                    self.callstack.ret(pc);
                }

                exec_itrnl.pc = src1.wrapping_add(sext(imm, I_TYPE_IMM_BITWIDTH) as u64) & (!1);
                reg_file.write(rd, pc + 4); // rd default to x1
                use_new_pc = true;
            }

            Inst64::lb => {
                // I x[rd] = sext(M[x[rs1] + sext(offset)][31:0])
                if self.itrace {
                    trace!("{}", pinst!(pc, lb, rd, imm(rs1)));
                }
                let vaddr = src1.wrapping_add(sext(imm, I_TYPE_IMM_BITWIDTH) as u64);
                let result = self.vm.mread::<u8>(vaddr as usize);
                // SEXT in RV64I
                let result = sext(result as u64, BYTE_BITWIDTH);
                reg_file.write(rd, result as u64);
            }
            Inst64::lbu => {
                // I x[rd] = M[x[rs1] + sext(offset)][31:0]
                if self.itrace {
                    trace!("{}", pinst!(pc, lbu, rd, imm(rs1)));
                }
                let vaddr = src1.wrapping_add(sext(imm, I_TYPE_IMM_BITWIDTH) as u64);
                let result = self.vm.mread::<u8>(vaddr as usize);
                // ZERO extend: just as u64
                reg_file.write(rd, result as u64);
            }
            Inst64::ld => {
                // I x[rd] = M[x[rs1] + sext(offset)][63:0]
                if self.itrace {
                    trace!("{}", pinst!(pc, ld, rd, imm(rs1)));
                }
                let vaddr = src1.wrapping_add(sext(imm, I_TYPE_IMM_BITWIDTH) as u64);
                let result = self.vm.mread::<u64>(vaddr as usize);
                reg_file.write(rd, result);
            }
            Inst64::lh => {
                // I x[rd] = sext(M[x[rs1] + sext(offset)][15:0])
                if self.itrace {
                    trace!("{}", pinst!(pc, lh, rd, imm(rs1)));
                }
                let vaddr = src1.wrapping_add(sext(imm, I_TYPE_IMM_BITWIDTH) as u64);
                let result = self.vm.mread::<u16>(vaddr as usize);
                // SEXT in RV64I
                let result = sext(result as u64, HALF_BITWIDTH);
                reg_file.write(rd, result as u64);
            }
            Inst64::lhu => {
                // I x[rd] = M[x[rs1] + sext(offset)][31:0]
                if self.itrace {
                    trace!("{}", pinst!(pc, lhu, rd, imm(rs1)));
                }
                let vaddr = src1.wrapping_add(sext(imm, I_TYPE_IMM_BITWIDTH) as u64);
                let result = self.vm.mread::<u16>(vaddr as usize);
                // ZERO extend: just as u64
                reg_file.write(rd, result as u64);
            }
            Inst64::lui => {
                // U x[rd] = sext(immediate[31:12] << 12)
                if self.itrace {
                    trace!("{}", pinst!(pc, lui, rd, imm=>imm));
                }
                let mask: u64 = !0b1111_1111_1111;
                let result = ((sext(imm, U_TYPE_IMM_BITWIDTH) << 12) as u64) & mask;
                reg_file.write(rd, result);
            }
            Inst64::lw => {
                // I x[rd] = sext(M[x[rs1] + sext(offset)][31:0])
                if self.itrace {
                    trace!("{}", pinst!(pc, lw, rd, imm(rs1)));
                }
                let vaddr = src1.wrapping_add(sext(imm, I_TYPE_IMM_BITWIDTH) as u64);
                let result = self.vm.mread::<u32>(vaddr as usize);
                // SEXT in RV64I
                let result = sext(result as u64, WORD_BITWIDTH);
                reg_file.write(rd, result as u64);
            }
            Inst64::lwu => {
                // I x[rd] = M[x[rs1] + sext(offset)][31:0]
                if self.itrace {
                    trace!("{}", pinst!(pc, lwu, rd, imm(rs1)));
                }
                let vaddr = src1.wrapping_add(sext(imm, I_TYPE_IMM_BITWIDTH) as u64);
                let result = self.vm.mread::<u32>(vaddr as usize);
                // ZERO extend: just as u64
                reg_file.write(rd, result as u64);
            }
            Inst64::mret => {
                // R
                if self.itrace {
                    trace!("{}", pinst!(pc, mret));
                }
                todo!()
            }
            Inst64::mul => {
                // R x[rd] = x[rs1] × x[rs2]
                if self.itrace {
                    trace!("{}", pinst!(pc, mul, rd, rs1, rs2));
                }
                let result = src1.wrapping_mul(src2);
                reg_file.write(rd, result);
            }
            Inst64::mulh => {
                // R x[rd] = (x[rs1] s×s x[rs2]) >>s XLEN
                if self.itrace {
                    trace!("{}", pinst!(pc, mulh, rd, rs1, rs2));
                }
                // RV64
                let result = (src1 as i128).wrapping_mul(src2 as i128);
                let result = get_high_64_bit(result as u128);
                reg_file.write(rd, result);
            }
            Inst64::mulhsu => {
                // R x[rd] = (x[rs1] s×u x[rs2]) >>s XLEN
                if self.itrace {
                    trace!("{}", pinst!(pc, mulhsu, rd, rs1, rs2));
                }
                let t_src1 = src1 as i64;
                let t_src2 = src2 as u64;
                let result = (t_src1 as i128).wrapping_mul(t_src2 as i128);
                let result = get_high_64_bit(result as u128);
                reg_file.write(rd, result);
            }
            Inst64::mulhu => {
                // R x[rd] = (x[rs1] u×u x[rs2]) >>u XLEN
                if self.itrace {
                    trace!("{}", pinst!(pc, mulhu, rd, rs1, rs2));
                }
                let result = (src1 as u128).wrapping_mul(src2 as u128);
                let result = get_high_64_bit(result);
                reg_file.write(rd, result);
            }
            Inst64::mulw => {
                // R x[rd] = sext((x[rs1] × x[rs2])[31:0])
                if self.itrace {
                    trace!("{}", pinst!(pc, mulw, rd, rs1, rs2));
                }
                let result = src1.wrapping_mul(src2);
                let result = sext(trunc_to_32_bit(result), WORD_BITWIDTH);
                reg_file.write(rd, result as u64);
            }
            Inst64::or => {
                // R x[rd] = x[rs1] | x[rs2]
                if self.itrace {
                    trace!("{}", pinst!(pc, or, rd, rs1, rs2));
                }
                let result = src1.bitor(src2);
                reg_file.write(rd, result);
            }
            Inst64::ori => {
                // I x[rd] = x[rs1] | sext(immediate)
                if self.itrace {
                    trace!("{}", pinst!(pc, ori, rd, rs1, imm=>imm));
                }
                let result = src1.bitor(sext(imm, I_TYPE_IMM_BITWIDTH) as u64);
                reg_file.write(rd, result);
            }

            Inst64::rem => {
                // R x[rd] = x[rs1] %s x[rs2]
                if self.itrace {
                    trace!("{}", pinst!(pc, rem, rd, rs1, rs2));
                }
                if src2 == 0 {
                    return Err(Error::Exception(Exception::DividedByZero));
                }
                let result = (src1 as i64).wrapping_rem(src2 as i64);
                reg_file.write(rd, result as u64);
            }
            Inst64::remu => {
                // R x[rd] = x[rs1] %u x[rs2]
                if self.itrace {
                    trace!("{}", pinst!(pc, remu, rd, rs1, rs2));
                }
                if src2 == 0 {
                    return Err(Error::Exception(Exception::DividedByZero));
                }
                let result = src1.wrapping_rem(src2);
                reg_file.write(rd, result);
            }
            Inst64::remuw => {
                // R x[rd] = sext(x[rs1][31:0] %u x[rs2][31:0])
                if self.itrace {
                    trace!("{}", pinst!(pc, remuw, rd, rs1, rs2));
                }
                if src2 == 0 {
                    return Err(Error::Exception(Exception::DividedByZero));
                }
                let t_src1 = trunc_to_32_bit(src1);
                let t_src2 = trunc_to_32_bit(src2);
                let result = t_src1.wrapping_rem(t_src2);
                let result = sext(result, WORD_BITWIDTH);
                reg_file.write(rd, result as u64);
            }
            Inst64::remw => {
                // R x[rd] = sext(x[rs1][31:0] %s x[rs2][31:0])
                if self.itrace {
                    trace!("{}", pinst!(pc, remw, rd, rs1, rs2));
                }
                if src2 == 0 {
                    return Err(Error::Exception(Exception::DividedByZero));
                }
                let t_src1 = trunc_to_32_bit(src1);
                let t_src2 = trunc_to_32_bit(src2);
                let result = (t_src1 as i64).wrapping_rem(t_src2 as i64);
                reg_file.write(rd, result as u64);
            }
            Inst64::sb => {
                // S M[x[rs1] + sext(offset)] = x[rs2][7:0]
                if self.itrace {
                    trace!("{}", pinst!(pc, sb, rs2, imm(rs1)));
                }
                let vaddr = src1.wrapping_add(sext(imm, S_TYPE_IMM_BITWIDTH) as u64);
                let result = trunc_to_8_bit(src2);
                self.vm.mwrite::<u8>(vaddr as usize, result as u8);
            }
            Inst64::sd => {
                // S M[x[rs1] + sext(offset)] = x[rs2][63:0]
                if self.itrace {
                    trace!("{}", pinst!(pc, sd, rs2, imm(rs1)));
                }
                let vaddr = src1.wrapping_add(sext(imm, S_TYPE_IMM_BITWIDTH) as u64);
                self.vm.mwrite::<u64>(vaddr as usize, src2);
                // self.vm.mread::<u64>(vaddr as usize);
            }
            Inst64::sh => {
                // S M[x[rs1] + sext(offset)] = x[rs2][15:0]
                if self.itrace {
                    trace!("{}", pinst!(pc, sh, rs2, imm(rs1)));
                }
                let vaddr = src1.wrapping_add(sext(imm, S_TYPE_IMM_BITWIDTH) as u64);
                self.vm
                    .mwrite::<u16>(vaddr as usize, trunc_to_16_bit(src2) as u16);
            }
            Inst64::sll => {
                // R x[rd] = x[rs1] << x[rs2]
                if self.itrace {
                    trace!("{}", pinst!(pc, sll, rd, rs1, rs2));
                }
                // let t_src2 = trunc_to_5_bit(src2); // RV32
                let t_src2 = trunc_to_6_bit(src2); // RV64
                let result = src1.wrapping_shl(t_src2 as u32);
                reg_file.write(rd, result);
            }
            Inst64::slli => {
                // I x[rd] = x[rs1] << shamt
                if self.itrace {
                    trace!("{}", pinst!(pc, slli, rd, rs1, imm=>imm));
                }
                // RV32I
                // let (shamt, legal) = trunc_to_5_bit_and_check(imm);
                // if !legal {
                //     return Err(Error::Exception(Exception::IllegalInstruction));
                // }
                // RV64I
                let shamt = trunc_to_6_bit(imm);
                let result = src1.wrapping_shl(shamt as u32);
                reg_file.write(rd, result);
            }
            Inst64::slliw => {
                // I x[rd] = x[rs1] << shamt
                if self.itrace {
                    trace!("{}", pinst!(pc, slliw, rd, rs1, imm=>imm));
                }
                let (shamt, legal) = trunc_to_5_bit_and_check(imm);
                if !legal {
                    return Err(Error::Exception(Exception::IllegalInstruction));
                }
                let result = src1.wrapping_shl(shamt as u32);
                let result = sext(trunc_to_32_bit(result), WORD_BITWIDTH);
                reg_file.write(rd, result as u64);
            }
            Inst64::sllw => {
                // R x[rd] = sext((x[rs1] << x[rs2][4:0])[31:0])
                if self.itrace {
                    trace!("{}", pinst!(pc, sllw, rd, rs1, rs2));
                }
                let t_src1 = trunc_to_32_bit(src1);
                let t_src2 = trunc_to_5_bit(src2);
                let result = t_src1.wrapping_shl(t_src2 as u32);
                let result = sext(trunc_to_32_bit(result), WORD_BITWIDTH);
                reg_file.write(rd, result as u64);
            }
            Inst64::slt => {
                // R x[rd] = x[rs1] <s x[rs2]
                if self.itrace {
                    trace!("{}", pinst!(pc, slt, rd, rs1, rs2));
                }
                let write_val = if (src1 as i64) < (src2 as i64) { 1 } else { 0 };
                reg_file.write(rd, write_val);
            }
            Inst64::slti => {
                // I x[rd] = x[rs1] <s sext(immediate)
                if self.itrace {
                    trace!("{}", pinst!(pc, slti, rd, rs1, imm=>imm));
                }
                let ext_imm = sext(imm, I_TYPE_IMM_BITWIDTH) as i64;
                let write_val = if (src1 as i64) < ext_imm { 1 } else { 0 };
                reg_file.write(rd, write_val);
            }
            Inst64::sltiu => {
                // I x[rd] = x[rs1] <u sext(immediate)
                if self.itrace {
                    trace!("{}", pinst!(pc, sltiu, rd, rs1, imm=>imm));
                }
                let ext_imm: u64 = sext(imm, I_TYPE_IMM_BITWIDTH) as u64;
                let write_val = if src1 < ext_imm { 1 } else { 0 };
                reg_file.write(rd, write_val);
            }
            Inst64::sltu => {
                // R x[rd] = x[rs1] <u x[rs2]
                if self.itrace {
                    trace!("{}", pinst!(pc, sltu, rd, rs1, rs2));
                }
                let write_val = if (src1 as u64) < (src2 as u64) { 1 } else { 0 };
                reg_file.write(rd, write_val);
            }
            Inst64::sra => {
                // R x[rd] = x[rs1] >>s x[rs2]
                if self.itrace {
                    trace!("{}", pinst!(pc, sra, rd, rs1, rs2));
                }
                // let t_src2 = trunc_to_5_bit(src2); // RV32
                let t_src2 = trunc_to_6_bit(src2); // RV64
                                                   // i64 shr automatically fill high bits with sign-bit
                let result = (src1 as i64).wrapping_shr(t_src2 as u32);
                reg_file.write(rd, result as u64);
            }
            Inst64::srai => {
                // I x[rd] = x[rs1] >>s shamt
                if self.itrace {
                    trace!("{}", pinst!(pc, srai, rd, rs1, imm=>imm));
                }
                // RV32I
                // let (shamt, legal) = trunc_to_5_bit_and_check(imm);
                // if !legal {
                //     return Err(Error::Exception(Exception::IllegalInstruction))
                // }
                // RV64I
                let shamt = trunc_to_6_bit(imm);
                let result = (src1 as i64).wrapping_shr(shamt as u32);
                reg_file.write(rd, result as u64);
            }
            Inst64::sraiw => {
                // I x[rd] = sext(x[rs1][31:0] >>s shamt)
                if self.itrace {
                    trace!("{}", pinst!(pc, sraiw, rd, rs1, imm=>imm));
                }
                let t_src1: i64 = sext(trunc_to_32_bit(src1), WORD_BITWIDTH);
                let (shamt, legal) = trunc_to_5_bit_and_check(imm);
                if !legal {
                    return Err(Error::Exception(Exception::IllegalInstruction));
                }
                let result = t_src1.wrapping_shr(shamt as u32);
                reg_file.write(rd, result as u64);
            }
            Inst64::sraw => {
                // R x[rd] = x[rs1] >>s x[rs2]
                if self.itrace {
                    trace!("{}", pinst!(pc, sraw, rd, rs1, rs2));
                }
                let t_src1: i64 = sext(trunc_to_32_bit(src1), WORD_BITWIDTH);
                let t_src2 = trunc_to_5_bit(src2);
                let result = t_src1.wrapping_shr(t_src2 as u32);
                let result = sext(result as u64, WORD_BITWIDTH);
                reg_file.write(rd, result as u64);
            }
            Inst64::sret => {
                // R
                if self.itrace {
                    trace!("{}", pinst!(pc, sret));
                }
                todo!()
            }
            Inst64::srl => {
                // R x[rd] = x[rs1] >>u x[rs2]
                if self.itrace {
                    trace!("{}", pinst!(pc, srl, rd, rs1, rs2));
                }
                // let t_src2 = trunc_to_5_bit(src2); // RV32
                let t_src2 = trunc_to_6_bit(src2); // RV64
                                                   // i64 shr automatically fill high bits with 0-bit
                let result = (src1 as u64).wrapping_shr(t_src2 as u32);
                reg_file.write(rd, result as u64);
            }
            Inst64::srli => {
                // I x[rd] = x[rs1] >>s shamt
                if self.itrace {
                    trace!("{}", pinst!(pc, srli, rd, rs1, imm=>imm));
                }
                // RV32I
                // let (shamt, legal) = trunc_to_5_bit_and_check(imm);
                // if !legal {
                //     return Err(Error::Exception(Exception::IllegalInstruction))
                // }
                // RV64I
                let shamt = trunc_to_6_bit(imm);
                let result = (src1 as u64).wrapping_shr(shamt as u32);
                reg_file.write(rd, result as u64);
            }
            Inst64::srliw => {
                // I x[rd] = sext(x[rs1][31:0] >>s shamt)
                if self.itrace {
                    trace!("{}", pinst!(pc, srliw, rd, rs1, imm=>imm));
                }
                let t_src1: u64 = trunc_to_32_bit(src1);
                let (shamt, legal) = trunc_to_5_bit_and_check(imm);
                if !legal {
                    return Err(Error::Exception(Exception::IllegalInstruction));
                }
                let result = sext(
                    trunc_to_32_bit(t_src1.wrapping_shr(shamt as u32)),
                    WORD_BITWIDTH,
                );
                reg_file.write(rd, result as u64);
            }
            Inst64::srlw => {
                // R x[rd] = x[rs1] >>s x[rs2]
                if self.itrace {
                    trace!("{}", pinst!(pc, srlw, rd, rs1, rs2));
                }
                let t_src1: u64 = trunc_to_32_bit(src1);
                let t_src2 = trunc_to_5_bit(src2);
                let result = t_src1.wrapping_shr(t_src2 as u32);
                let result = sext(result as u64, WORD_BITWIDTH);
                reg_file.write(rd, result as u64);
            }
            Inst64::sub => {
                // R x[rd] = x[rs1] - x[rs2]
                if self.itrace {
                    trace!("{}", pinst!(pc, sub, rd, rs1, rs2));
                }
                let result = src1.wrapping_sub(src2);
                reg_file.write(rd, result);
            }
            Inst64::subw => {
                // R x[rd] = sext((x[rs1] - x[rs2])[31:0])
                if self.itrace {
                    trace!("{}", pinst!(pc, subw, rd, rs1, rs2));
                }
                let result = trunc_to_32_bit(src1.wrapping_sub(src2));
                let result = sext(result, WORD_BITWIDTH);
                reg_file.write(rd, result as u64);
            }
            Inst64::sw => {
                // S M[x[rs1] + sext(offset)] = x[rs2][31:0]
                if self.itrace {
                    trace!("{}", pinst!(pc, sw, rs2, imm(rs1)));
                }
                let vaddr = src1.wrapping_add(sext(imm, S_TYPE_IMM_BITWIDTH) as u64);
                let write_val = trunc_to_32_bit(src2);
                self.vm.mwrite::<u32>(vaddr as usize, write_val as u32);
                // self.vm.mread::<u64>(vaddr as usize);
            }

            Inst64::xor => {
                // R x[rd] = x[rs1] ˆ x[rs2]
                if self.itrace {
                    trace!("{}", pinst!(pc, xor, rd, rs1, rs2));
                }
                let result = src1.bitxor(src2);
                reg_file.write(rd, result);
            }
            Inst64::xori => {
                // I x[rd] = x[rs1] ˆ sext(immediate)
                if self.itrace {
                    trace!("{}", pinst!(pc, xori, rd, rs1, imm=>imm));
                }
                let result = src1.bitxor(sext(imm, I_TYPE_IMM_BITWIDTH) as u64);
                reg_file.write(rd, result);
            }

            _ => error!("Unknown inst {:?}", exec_itrnl.inst),
        }

        // write pc back
        self.pc
            .write(if use_new_pc { exec_itrnl.pc } else { pc + 4 });

        // reset x0 to 0
        reg_file.write(0, 0);

        Ok(())
    }
}

impl<'a> CPU<'a> {
    pub fn halt(&mut self, pc: u64, code: u64) {
        if code != 0 {
            error!("HIT BAD TRAP!\n");
        } else {
            info!("HIT GOOD TRAP!\n");
        }
        self.running = false;
        info!("Program ended at pc {:#x}, with exit code {}", pc, code);
    }

    pub fn mread<T: Sized + Display>(&self, vaddr: u64) -> T {
        self.vm.mread(vaddr as usize)
    }

    pub fn backtrace(&self) {
        self.callstack.backtrace();
    }
}

impl<'a> CPU<'a> {
    pub fn pc(&self) -> u64 {
        self.pc.read()
    }

    pub fn reg_val_by_name(&self, name: &str) -> Result<u64> {
        let idx = match name {
            "zero" | "x0" => 0,
            "ra" | "x1" => 1,
            "sp" | "x2" => 2,
            "gp" | "x3" => 3,
            "tp" | "x4" => 4,
            "t0" | "x5" => 5,
            "t1" | "x6" => 6,
            "t2" | "x7" => 7,
            "s0" | "x8" => 8,
            "s1" | "x9" => 9,
            "a0" | "x10" => 10,
            "a1" | "x11" => 11,
            "a2" | "x12" => 12,
            "a3" | "x13" => 13,
            "a4" | "x14" => 14,
            "a5" | "x15" => 15,
            "a6" | "x16" => 16,
            "a7" | "x17" => 17,
            "s2" | "x18" => 18,
            "s3" | "x19" => 19,
            "s4" | "x20" => 20,
            "s5" | "x21" => 21,
            "s6" | "x22" => 22,
            "s7" | "x23" => 23,
            "s8" | "x24" => 24,
            "s9" | "x25" => 25,
            "s10" | "x26" => 26,
            "s11" | "x27" => 27,
            "t3" | "x28" => 28,
            "t4" | "x29" => 29,
            "t5" | "x30" => 30,
            "t6" | "x31" => 31,
            "pc" => return Ok(self.pc()),
            _ => return Err(Error::InvalidRegName(name.into())),
        };
        Ok(self.reg_file.read(idx))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn print_minus1_as_u64() {
        println!("print_minus1_as_u64:");
        let a = -1i64;
        let b = -1i64 as u64;
        println!("{:b}\n{:b}", a, b);
    }

    #[test]
    fn test_5_bit_sext() {
        let signed_imm: u64 = 0b00011101;
        let unsigned_imm: u64 = 0b00001101;
        let bit_width = 5;
        let ext_s_imm: i64 = sext(signed_imm, bit_width);
        let ext_u_imm: i64 = sext(unsigned_imm, bit_width);

        println!("ext_s_imm (as i64) = {:64b}", ext_s_imm);
        println!("ext_s_imm (as u64) = {:64b}", ext_s_imm as u64);
        println!("ext_u_imm (as i64) = {:64b}", ext_u_imm);
        println!("ext_u_imm (as u64) = {:64b}", ext_u_imm as u64);

        assert_eq!(ext_s_imm as u64, 0xFFFFFFFF_FFFFFFFD);
        assert_eq!(ext_u_imm as u64, 0x00000000_0000000D);
    }
    #[test]
    fn test_12_bit_sext() {
        let signed_imm: u64 = 0b110101011110;
        let unsigned_imm: u64 = 0b010101011110;
        let bit_width = I_TYPE_IMM_BITWIDTH; // 12
        let ext_s_imm: i64 = sext(signed_imm, bit_width);
        let ext_u_imm: i64 = sext(unsigned_imm, bit_width);
        assert_eq!(ext_s_imm as u64, 0xFFFFFFFF_FFFFFD5E);
        assert_eq!(ext_u_imm as u64, 0x00000000_0000055E);
    }
    #[test]
    fn test_12_bit_exceeding_sext() {
        let signed_imm: u64 = 0b11_110101011110;
        let unsigned_imm: u64 = 0b11_010101011110;
        let bit_width = I_TYPE_IMM_BITWIDTH; // 12
        let ext_s_imm: i64 = sext(signed_imm, bit_width);
        let ext_u_imm: i64 = sext(unsigned_imm, bit_width);
        assert_eq!(ext_s_imm as u64, 0xFFFFFFFF_FFFFFD5E);
        assert_eq!(ext_u_imm as u64, 0x00000000_0000055E);
    }
    #[test]
    fn test_truncate_64_to_32() {
        let imm: u64 = 0xFFFFFFFF_FFFFFFFF;
        let result: u64 = trunc_to_32_bit(imm);
        assert_eq!(result, 0x00000000_FFFFFFFF);
    }
}
