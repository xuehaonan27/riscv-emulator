use log::{debug, error, info, trace, warn};

use crate::{
    callstack::CallStack,
    core::{
        reg::{ProgramCounter, RegisterFile},
        utils::reg_name_by_id,
        vm::VirtualMemory,
    },
    elf::LoadElfInfo,
    error::Result,
};

use super::{
    decode::decode, exec::exec, fetch::fetch, hazard::HazardDetectionUnit, mem::mem, phases::*,
    writeback::writeback,
};

pub struct CPU<'a> {
    // indicate whether the CPU is running
    running: bool,

    // indicate whether the CPU should continue to fetch instruction
    continue_fetch: bool,

    // clock
    clock: u64,

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

    // IF / ID
    itl_f_d: InternalFetchDecode,

    // ID / Exec
    itl_d_e: InternalDecodeExec,

    // Exec / Mem
    itl_e_m: InternalExecMem,

    // Mem / Wb
    itl_m_w: InternalMemWb,

    // HazardResolveUnit
    hazard_detection_unit: HazardDetectionUnit,

    // Remaining stall clocks
    stall: u8,
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
            continue_fetch: true,
            clock: 0,
            reg_file,
            pc,
            vm,
            callstack,
            itrace,
            itl_f_d: InternalFetchDecode::default(),
            itl_d_e: InternalDecodeExec::default(),
            itl_e_m: InternalExecMem::default(),
            itl_m_w: InternalMemWb::default(),
            hazard_detection_unit: HazardDetectionUnit::default(),
            stall: 1,
        }
    }

    /// Initialize CPU with ELF info
    pub fn init_elfinfo_64(&mut self, info: &LoadElfInfo) {
        // make sure we are running a ELF64 executable
        assert!(info.is_64_bit());

        self.reg_file.init_elfinfo_64(info);

        let pc = info.entry_point();

        // Load program counter
        self.pc.write(pc);

        // Load pc into pipeline registers
        self.itl_f_d.pc = pc;
        self.itl_d_e.pc = pc;
        self.itl_e_m.pc = pc;
        self.itl_m_w.pc = pc;
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
            self.clock()?;
            i += 1;
        }

        Ok(())
    }

    fn stall(&mut self) {
        // self.itl_f_d.branch_flags.clear();
        // self.itl_f_d.decode_flags.clear();
        // self.itl_f_d.exec_flags.clear();
        // self.itl_f_d.mem_flags.clear();
        // self.itl_f_d.wb_flags.clear();
        // self.itl_f_d.pc = 0;
        self.itl_f_d = InternalFetchDecode::default();

        // self.itl_d_e.branch_flags.clear();
        // self.itl_d_e.exec_flags.clear();
        // self.itl_d_e.mem_flags.clear();
        // self.itl_d_e.wb_flags.clear();
        // self.itl_d_e.pc = 0;
        self.itl_d_e = InternalDecodeExec::default();
    }

    fn clock(&mut self) -> Result<()> {
        // DEBUG: input()
        input();

        // begin the clock
        self.clock += 1;
        debug!(
            "#################### CLOCK: {} ####################",
            self.clock
        );

        // detect control hazard
        {
            // debug!("Detecting control hazard");
            use crate::core::insts::Inst64::*;
            match self.itl_e_m.alu_op {
                i @ (beq | bge | bgeu | blt | bltu | bne | jal | jalr) => {
                    warn!("Control hazard detected: {i:?}");
                    warn!("  Stall 3 cycles");
                    self.stall = self.stall.max(3); // stall 3 cycles
                }
                _ => (),
            }
        }

        // decide whether CPU should stall
        let mut should_stall = self.stall != 0;
        self.stall -= if self.stall == 0 { 0 } else { 1 };
        if should_stall {
            warn!("CPU stall: {should_stall}");
        } else {
            debug!("CPU stall: {should_stall}");
        }

        // detect load-use hazard
        {
            // debug!("Detecting load-use hazard");
            if self.itl_d_e.mem_flags.mem_read
                && ((self.itl_d_e.rd == self.itl_f_d.rs1) || (self.itl_d_e.rd == self.itl_f_d.rs2))
            {
                // stall the pipeline
                // id est set the control bits in the EX,MEM, and WB control fields of the ID/EX
                // pipeline register to 0 (noop).
                warn!("Load-use hazard detected.");
                warn!(
                    "  IF/ID.rs1={}({})",
                    self.itl_d_e.rd,
                    reg_name_by_id(self.itl_d_e.rd)
                );
                warn!(
                    "  ID/EX.rd={}({})",
                    self.itl_f_d.rs1,
                    reg_name_by_id(self.itl_f_d.rs1)
                );
                warn!(
                    "  IF/ID.rs2={}({})",
                    self.itl_f_d.rs2,
                    reg_name_by_id(self.itl_f_d.rs2)
                );
                warn!("  Stall 1 cycle");
                should_stall = true;
                self.stall = self.stall.max(1);
            }
        }

        // detect memory-to-memory copy
        {
            // debug!("Detecting memory-to-memory hazard");
            let mem_wb_rd = self.itl_m_w.rd;
            let mem_wb_mem_read = self.itl_m_w.mem_read;
            let exec_mem_rd = self.itl_e_m.rd;
            let exec_mem_mem_write = self.itl_e_m.mem_flags.mem_write;
            if (mem_wb_rd == exec_mem_rd) && mem_wb_mem_read && exec_mem_mem_write {
                warn!("Memory-to-memory hazard detected");
                warn!("  Forwarding regval of MEM/WB");
                warn!(
                    "  MEM/WB.rd={}({}) to EXEC/MEM.rd={}({})",
                    mem_wb_rd,
                    reg_name_by_id(mem_wb_rd),
                    exec_mem_rd,
                    reg_name_by_id(exec_mem_rd)
                );
                self.itl_e_m.m2m_forward = true;
                self.itl_e_m.m2m_forward_val = self.itl_m_w.regval;
            }
        }

        // detect data hazards
        let ex_mem_forward = {
            // debug!("Detecting EX/MEM data hazards");
            // EX/MEM hazard
            let ex_mem_reg_write = self.itl_e_m.wb_flags.mem_to_reg;
            let ex_mem_rd = self.itl_e_m.rd;
            let id_ex_rs1 = self.itl_d_e.rs1;
            let id_ex_rs2 = self.itl_d_e.rs2;
            if ex_mem_reg_write && (ex_mem_rd != 0) && (ex_mem_rd == id_ex_rs1) {
                warn!("EX/MEM data hazard detected, for ALU SRC A");
                warn!("  EX/MEM.rd={}({})", ex_mem_rd, reg_name_by_id(ex_mem_rd));
                warn!("  ID/EX.rs1={}({})", id_ex_rs1, reg_name_by_id(id_ex_rs1));
                // forward A from EX/MEM
                self.itl_d_e.forward_a = 0b10;
            }
            if ex_mem_reg_write && (ex_mem_rd != 0) && (ex_mem_rd == id_ex_rs2) {
                warn!("EX/MEM data hazard detected, for ALU SRC B");
                warn!("  EX/MEM.rd={}({})", ex_mem_rd, reg_name_by_id(ex_mem_rd));
                warn!("  ID/EX.rs1={}({})", id_ex_rs2, reg_name_by_id(id_ex_rs2));
                // forward B from EX/MEM
                self.itl_d_e.forward_b = 0b10;
            }
            self.itl_e_m.alu_out
        };

        let mem_wb_forward = {
            // MEM/WB hazard
            let mem_wb_regwrite = self.itl_m_w.wb_flags.mem_to_reg;
            let mem_wb_rd = self.itl_m_w.rd;
            let ex_mem_rd = self.itl_e_m.rd;
            let id_ex_rs1 = self.itl_d_e.rs1;
            let id_ex_rs2 = self.itl_d_e.rs2;
            if mem_wb_regwrite
                && (mem_wb_rd != 0)
                && (ex_mem_rd != id_ex_rs1)
                && (mem_wb_rd == id_ex_rs1)
            {
                warn!("MEM/WB data hazard detected, for ALU SRC A");
                warn!("MEM/WB.rd={}({})", mem_wb_rd, reg_name_by_id(mem_wb_rd));
                warn!("ID/EX.rs1={}({})", id_ex_rs1, reg_name_by_id(id_ex_rs1));
                // forward A from MEM/WB
                assert_eq!(self.itl_d_e.forward_a, 0);
                self.itl_d_e.forward_a = 0b01;
            }
            if mem_wb_regwrite
                && (mem_wb_rd != 0)
                && (ex_mem_rd != id_ex_rs2)
                && (mem_wb_rd == id_ex_rs2)
            {
                warn!("MEM/WB data hazard detected, for ALU SRC B");
                warn!("MEM/WB.rd={}({})", mem_wb_rd, reg_name_by_id(mem_wb_rd));
                warn!("ID/EX.rs1={}({})", id_ex_rs2, reg_name_by_id(id_ex_rs2));
                // forward B from MEM/WB
                assert_eq!(self.itl_d_e.forward_b, 0);
                self.itl_d_e.forward_b = 0b01;
            }
            self.itl_m_w.regval
        };

        // hazard detect unit decides whether CPU should stall
        if should_stall {
            self.stall();
        }

        // print current CPU status

        // function units
        let running = writeback(&self.itl_m_w, &mut self.reg_file);
        // debug!("MEM/WB {:#x} {:#?}", self.itl_m_w.pc, self.itl_m_w.alu_op);

        let new_itl_m_w = mem(&self.itl_e_m, &mut self.vm);
        // debug!("EX/MEM {:#x} {:#?}", self.itl_e_m.pc, self.itl_e_m.alu_op);

        let (new_itl_e_m, ex_branch, pc_src, new_pc_0, new_pc_1) =
            exec(&self.itl_d_e, ex_mem_forward, mem_wb_forward)?;
        // debug!(
        //     "ID/EX  {:#x} {:#?}",
        //     self.itl_d_e.pc, self.itl_d_e.exec_flags.alu_op
        // );
        let new_itl_d_e = decode(&self.reg_file, &self.itl_f_d);
        // debug!(
        //     "IF/ID  {:#x} {:#?}",
        //     self.itl_f_d.pc, self.itl_f_d.exec_flags.alu_op
        // );

        // decide the pc (by hazard unit)
        let real_pc = if should_stall {
            // if the CPU should stall, then Hazard Detect Unit should keep pc unchanged.
            self.pc.read()
        } else {
            if ex_branch {
                // if Exec phase is a branch instruction, then decide new pc according to `pc_src` flag.
                if pc_src {
                    new_pc_1
                } else {
                    // if do not use new pc, then we should recover the pc seen by the branch instruction!
                    new_pc_0
                }
            } else {
                assert!(!pc_src); // pc_src must be false!
                // if Exec phase is not a branch instruction, then new pc should add by itself,
                // instead of fetching `pc+4` from Exec phase instruction.
                self.pc.read().wrapping_add(4)
            }
        };

        // If using NaivePolicy (Stall 3 cycles) then nothing special need to be done.
        // Write back pc
        self.pc.write(real_pc);

        // Fetch code
        let new_itl_f_d = if self.continue_fetch {
            let (new_itl_f_d, continue_fetch) = fetch(&self.pc, &mut self.vm)?;
            self.continue_fetch = continue_fetch;
            new_itl_f_d
        } else {
            InternalFetchDecode::default()
        };

        info!("EX: PC decided {} {:#x}", pc_src, real_pc);

        // push pipeline forward
        self.itl_m_w = new_itl_m_w;
        self.itl_e_m = new_itl_e_m;
        self.itl_d_e = new_itl_d_e;
        self.itl_f_d = new_itl_f_d;

        // reset x0 to 0
        self.reg_file.write(0, 0);

        // decide whether continue to run
        self.running = running;
        Ok(())
    }
}

fn input() {
    use std::io;
    use std::io::{BufRead, Write};
    let mut buf = String::new();
    print!("(REDB)>>> ");
    io::stdout().flush().expect("Fail to flush");
    buf.clear();
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    stdin.read_line(&mut buf).unwrap();
}

pub fn halt(pc: u64, code: u64) {
    if code != 0 {
        error!("HIT BAD TRAP!\n");
    } else {
        info!("HIT GOOD TRAP!\n");
    }
    info!("Program ended at pc {:#x}, with exit code {}", pc, code);
}
