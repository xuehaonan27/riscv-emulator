use log::{debug, error, info, warn};

use crate::{
    callstack::CallStack,
    core::{
        insts::Inst64,
        reg::{ProgramCounter, RegisterFile, REGNAME},
        vm::VirtualMemory,
    },
    elf::LoadElfInfo,
    error::{Error, Result},
};

use super::{
    branch_predict::{BHT, BTB, RAS},
    decode::decode,
    exec::exec,
    fetch::fetch,
    mem::mem,
    phases::*,
    writeback::writeback,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PipelineState {
    Stall,
    Bubble,
    Normal,
}

impl PartialOrd for PipelineState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering::*;
        match (self, other) {
            (PipelineState::Stall, PipelineState::Stall) => Some(Equal),
            (PipelineState::Bubble, PipelineState::Bubble) => Some(Equal),
            (PipelineState::Normal, PipelineState::Normal) => Some(Equal),

            (PipelineState::Normal, PipelineState::Bubble) => Some(Less),
            (PipelineState::Normal, PipelineState::Stall) => Some(Less),
            (PipelineState::Bubble, PipelineState::Stall) => Some(Less),

            (PipelineState::Stall, PipelineState::Bubble) => Some(Greater),
            (PipelineState::Stall, PipelineState::Normal) => Some(Greater),
            (PipelineState::Bubble, PipelineState::Normal) => Some(Greater),
        }
    }
}

impl Ord for PipelineState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::*;
        match (self, other) {
            (PipelineState::Stall, PipelineState::Stall) => Equal,
            (PipelineState::Bubble, PipelineState::Bubble) => Equal,
            (PipelineState::Normal, PipelineState::Normal) => Equal,

            (PipelineState::Normal, PipelineState::Bubble) => Less,
            (PipelineState::Normal, PipelineState::Stall) => Less,
            (PipelineState::Bubble, PipelineState::Stall) => Less,

            (PipelineState::Stall, PipelineState::Bubble) => Greater,
            (PipelineState::Stall, PipelineState::Normal) => Greater,
            (PipelineState::Bubble, PipelineState::Normal) => Greater,
        }
    }
}

const PIPELINE_STATES_DEPTH: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum DataHazardPolicy {
    NaiveStall,  // just stall
    DataForward, // data forwarding
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ControlPolicy {
    AllStall,       // stall when branch instructions encountered
    AlwaysNotTaken, // static branch prediction: always not taken
    // AlwaysTaken,    // static branch prediction: always taken
    DynamicPredict, // dynamic branch prediction
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum PredictPolicy {
    OneBitPredict,
    TwoBitsPredict,
}

#[derive(Debug, Clone)]
pub struct CPUStatistics {
    data_hazard_count: u64,
    control_hazard_count: u64,
    data_hazard_delayed_cycles: u64,
    control_hazard_delayed_cycles: u64,
    executed_inst_count: u64,
}

impl Default for CPUStatistics {
    fn default() -> Self {
        Self {
            data_hazard_count: 0,
            control_hazard_count: 0,
            data_hazard_delayed_cycles: 0,
            control_hazard_delayed_cycles: 0,
            executed_inst_count: 0,
        }
    }
}

pub struct CPU<'a> {
    // indicate whether the CPU is running
    running: bool,

    // indicate whether the CPU should continue to fetch instruction
    // continue_fetch: bool,

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

    // IF / ID
    itl_f_d: InternalFetchDecode,

    // ID / Exec
    itl_d_e: InternalDecodeExec,

    // Exec / Mem
    itl_e_m: InternalExecMem,

    // Mem / Wb
    itl_m_w: InternalMemWb,

    // MEM/WB pipeline states
    m_w_pipeline_states: [PipelineState; PIPELINE_STATES_DEPTH],

    // EX/MEM pipeline states
    e_m_pipeline_states: [PipelineState; PIPELINE_STATES_DEPTH],

    // ID/EX pipeline states
    d_e_pipeline_states: [PipelineState; PIPELINE_STATES_DEPTH],

    // IF/ID pipeline states
    f_d_pipeline_states: [PipelineState; PIPELINE_STATES_DEPTH],

    // PC next states
    pc_next_states: [PipelineState; PIPELINE_STATES_DEPTH],

    // Data hazard policy
    data_hazard_policy: DataHazardPolicy,

    // Control policy
    control_policy: ControlPolicy,

    // Pre-execution pipeline register info
    pre_pipeline_info: bool,

    // Pipeline info
    pipeline_info: bool,

    // Post-execution pipeline register info
    post_pipeline_info: bool,

    // Control hazard info
    control_hazard_info: bool,

    // Data hazard info
    data_hazard_info: bool,

    // Clock info
    clock_info: bool,

    // Statistics data of CPU
    cpu_statistics: CPUStatistics,

    // Branch history table
    bht: Option<BHT>,

    // Branch target buffer
    btb: Option<BTB>,

    // Return address stack
    ras: RAS,
}

impl<'a> CPU<'a> {
    pub fn new(
        vm: &'a mut VirtualMemory,
        callstack: &'a mut CallStack<'a>,
        data_hazard_policy: DataHazardPolicy,
        control_policy: ControlPolicy,
        predict_policy: Option<PredictPolicy>,
        pre_pipeline_info: bool,
        pipeline_info: bool,
        post_pipeline_info: bool,
        control_hazard_info: bool,
        data_hazard_info: bool,
    ) -> CPU<'a> {
        // x0 already set to 0
        let reg_file = RegisterFile::empty();
        let pc = ProgramCounter::new();

        let bht = if let Some(predict_policy) = predict_policy {
            Some(BHT::new(predict_policy))
        } else {
            None
        };
        let btb = if let Some(_) = predict_policy {
            Some(BTB::new())
        } else {
            None
        };

        CPU {
            running: false,
            // continue_fetch: true,
            clock: 0,
            reg_file,
            pc,
            vm,
            callstack,
            itl_f_d: InternalFetchDecode::default(),
            itl_d_e: InternalDecodeExec::default(),
            itl_e_m: InternalExecMem::default(),
            itl_m_w: InternalMemWb::default(),
            m_w_pipeline_states: [PipelineState::Normal; PIPELINE_STATES_DEPTH],
            e_m_pipeline_states: [PipelineState::Normal; PIPELINE_STATES_DEPTH],
            d_e_pipeline_states: [PipelineState::Normal; PIPELINE_STATES_DEPTH],
            f_d_pipeline_states: [PipelineState::Normal; PIPELINE_STATES_DEPTH],
            pc_next_states: [PipelineState::Normal; PIPELINE_STATES_DEPTH],
            data_hazard_policy,
            control_policy,
            pre_pipeline_info,
            pipeline_info,
            post_pipeline_info,
            control_hazard_info,
            data_hazard_info,
            clock_info: pre_pipeline_info
                || pipeline_info
                || post_pipeline_info
                || control_hazard_info
                || data_hazard_info,
            cpu_statistics: CPUStatistics::default(),
            bht,
            btb,
            ras: RAS::new(),
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

    pub fn print_info(&self) {
        info!("CPU run clock: {}", self.clock);
        info!(
            "CPU data hazard count: {}",
            self.cpu_statistics.data_hazard_count
        );
        info!(
            "CPU data hazard delayed cycles: {}",
            self.cpu_statistics.data_hazard_delayed_cycles
        );
        info!(
            "CPU control hazard count: {}",
            self.cpu_statistics.control_hazard_count
        );
        info!(
            "CPU control hazard delayed cycles: {}",
            self.cpu_statistics.control_hazard_delayed_cycles
        );
        info!(
            "CPU executed valid instructions: {}",
            self.cpu_statistics.executed_inst_count
        );
        info!("CPI = {}", {
            let cycles = self.clock;
            let insts = self.cpu_statistics.executed_inst_count;
            (cycles as f64) / (insts as f64)
        });
    }

    pub(super) fn clock(&mut self) -> Result<()> {
        // begin the clock
        self.clock += 1;
        if self.clock_info {
            debug!(
                "#################### CLOCK: {} ####################",
                self.clock
            );
        }

        // detect load-use hazard
        let load_use_detected = {
            // debug!("Detecting load-use hazard");
            if self.itl_d_e.mem_flags.mem_read
                && ((self.itl_d_e.rd == self.itl_f_d.rs1) || (self.itl_d_e.rd == self.itl_f_d.rs2))
            {
                // stall the pipeline
                // id est set the control bits in the EX,MEM, and WB control fields of the ID/EX
                // pipeline register to 0 (noop).
                if self.data_hazard_info {
                    warn!("Load-use hazard detected.");
                    warn!(
                        "  IF/ID.rs1={}({})",
                        self.itl_d_e.rd, REGNAME[self.itl_d_e.rd as usize],
                    );
                    warn!(
                        "  ID/EX.rd={}({})",
                        self.itl_f_d.rs1, REGNAME[self.itl_f_d.rs1 as usize]
                    );
                    warn!(
                        "  IF/ID.rs2={}({})",
                        self.itl_f_d.rs2, REGNAME[self.itl_f_d.rs2 as usize]
                    );
                    warn!("  Stall 1 cycle");
                }
                self.cpu_statistics.data_hazard_count += 1;
                true
            } else {
                false
            }
        };

        // detect memory-to-memory copy
        {
            match self.data_hazard_policy {
                DataHazardPolicy::NaiveStall => {
                    // solved by EX/MEM NaiveInstall policy logic
                    let id_ex_rd = self.itl_d_e.rd;
                    let id_ex_mem_read = self.itl_d_e.mem_flags.mem_read;
                    let if_id_rs2 = self.itl_f_d.rs2;
                    let if_id_mem_write = self.itl_f_d.mem_flags.mem_write;
                    if (id_ex_rd != 0)
                        && (id_ex_rd == if_id_rs2)
                        && id_ex_mem_read
                        && if_id_mem_write
                    {
                        if self.data_hazard_info {
                            warn!("Memory-to-memory copy hazard detected");
                        }
                        self.cpu_statistics.data_hazard_count += 1;
                        self.cpu_statistics.data_hazard_delayed_cycles += 2;
                        self.d_e_pipeline_states_set(&mut [PipelineState::Bubble]);
                        self.f_d_pipeline_states_set(&mut [
                            PipelineState::Stall,
                            PipelineState::Bubble,
                        ]);
                        self.pc_next_states_set(&mut [PipelineState::Stall, PipelineState::Stall]);
                    }
                }
                DataHazardPolicy::DataForward => {
                    // debug!("Detecting memory-to-memory hazard");
                    let mem_wb_rd = self.itl_m_w.rd;
                    let mem_wb_mem_read = self.itl_m_w.mem_read;
                    let exec_mem_rs2 = self.itl_e_m.rs2;
                    let exec_mem_mem_write = self.itl_e_m.mem_flags.mem_write;
                    if (mem_wb_rd != 0)
                        && (mem_wb_rd == exec_mem_rs2)
                        && mem_wb_mem_read
                        && exec_mem_mem_write
                    {
                        if self.data_hazard_info {
                            warn!("Memory-to-memory hazard detected");
                            warn!("  Forwarding regval of MEM/WB");
                            warn!(
                                "  MEM/WB.rd={}({}) to EXEC/MEM.rs2={}({})",
                                mem_wb_rd,
                                REGNAME[mem_wb_rd as usize],
                                exec_mem_rs2,
                                REGNAME[exec_mem_rs2 as usize]
                            );
                        }
                        self.cpu_statistics.data_hazard_count += 1;
                        self.itl_e_m.m2m_forward = true;
                        self.itl_e_m.m2m_forward_val = self.itl_m_w.regval;
                    }
                }
            }
        }

        // detect data hazards
        match self.data_hazard_policy {
            DataHazardPolicy::NaiveStall => {
                let id_ex_reg_write = self.itl_d_e.wb_flags.mem_to_reg;
                let id_ex_rd = self.itl_d_e.rd;
                let if_id_rs1 = self.itl_f_d.rs1;
                let if_id_rs2 = self.itl_f_d.rs2;
                if id_ex_reg_write
                    && (id_ex_rd != 0)
                    && ((id_ex_rd == if_id_rs1) || (id_ex_rd == if_id_rs2))
                {
                    if self.data_hazard_info {
                        warn!("EX/MEM data hazard detected");
                    }
                    self.cpu_statistics.data_hazard_count += 1;
                    self.cpu_statistics.data_hazard_delayed_cycles += 2;
                    self.d_e_pipeline_states_set(&mut [PipelineState::Bubble]);
                    self.f_d_pipeline_states_set(&mut [
                        PipelineState::Stall,
                        PipelineState::Bubble,
                    ]);
                    self.pc_next_states_set(&mut [PipelineState::Stall, PipelineState::Stall]);
                }
            }
            DataHazardPolicy::DataForward => {
                let ex_mem_forward = {
                    // debug!("Detecting EX/MEM data hazards");
                    // EX/MEM hazard
                    let ex_mem_reg_write = self.itl_e_m.wb_flags.mem_to_reg;
                    let ex_mem_rd = self.itl_e_m.rd;
                    let id_ex_rs1 = self.itl_d_e.rs1;
                    let id_ex_rs2 = self.itl_d_e.rs2;
                    if ex_mem_reg_write && (ex_mem_rd != 0) && (ex_mem_rd == id_ex_rs1) {
                        if self.data_hazard_info {
                            warn!("EX/MEM data hazard detected, for ALU SRC A");
                            warn!("  EX/MEM.rd={}({})", ex_mem_rd, REGNAME[ex_mem_rd as usize]);
                            warn!("  ID/EX.rs1={}({})", id_ex_rs1, REGNAME[id_ex_rs1 as usize]);
                        }
                        // forward A from EX/MEM
                        self.itl_d_e.forward_a = 0b10;
                    }
                    if ex_mem_reg_write && (ex_mem_rd != 0) && (ex_mem_rd == id_ex_rs2) {
                        if self.data_hazard_info {
                            warn!("EX/MEM data hazard detected, for ALU SRC B");
                            warn!("  EX/MEM.rd={}({})", ex_mem_rd, REGNAME[ex_mem_rd as usize]);
                            warn!("  ID/EX.rs1={}({})", id_ex_rs2, REGNAME[id_ex_rs2 as usize]);
                        }
                        // forward B from EX/MEM
                        self.itl_d_e.forward_b = 0b10;
                    }

                    if ex_mem_reg_write
                        && (ex_mem_rd != 0)
                        && ((ex_mem_rd == id_ex_rs1) || (ex_mem_rd == id_ex_rs2))
                    {
                        self.cpu_statistics.data_hazard_count += 1;
                    }

                    self.itl_e_m.alu_out
                };
                self.itl_d_e.ex_mem_forward = ex_mem_forward;
            }
        }

        match self.data_hazard_policy {
            DataHazardPolicy::NaiveStall => {
                let ex_mem_regwrite = self.itl_e_m.wb_flags.mem_to_reg;
                let ex_mem_rd = self.itl_e_m.rd;
                let id_ex_rd = self.itl_d_e.rd;
                let if_id_rs1 = self.itl_f_d.rs1;
                let if_id_rs2 = self.itl_f_d.rs2;
                if ex_mem_regwrite
                    && (ex_mem_rd != 0)
                    && (((id_ex_rd != if_id_rs1) && (ex_mem_rd == if_id_rs1))
                        || ((id_ex_rd != if_id_rs2) && (ex_mem_rd == if_id_rs2)))
                {
                    if self.data_hazard_info {
                        warn!("MEM/WB data hazard detected");
                    }
                    self.cpu_statistics.data_hazard_count += 1;
                    self.cpu_statistics.data_hazard_delayed_cycles += 1;

                    self.d_e_pipeline_states_set(&mut [PipelineState::Bubble]); // ?
                    self.f_d_pipeline_states_set(&mut [PipelineState::Stall]);
                    self.pc_next_states_set(&mut [PipelineState::Stall]);
                }
            }
            DataHazardPolicy::DataForward => {
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
                        if self.data_hazard_info {
                            warn!("MEM/WB data hazard detected, for ALU SRC A");
                            warn!("  MEM/WB.rd={}({})", mem_wb_rd, REGNAME[mem_wb_rd as usize]);
                            warn!("  ID/EX.rs1={}({})", id_ex_rs1, REGNAME[id_ex_rs1 as usize]);
                        }
                        // forward A from MEM/WB
                        assert_eq!(self.itl_d_e.forward_a, 0);
                        self.itl_d_e.forward_a = 0b01;
                    }
                    if mem_wb_regwrite
                        && (mem_wb_rd != 0)
                        && (ex_mem_rd != id_ex_rs2)
                        && (mem_wb_rd == id_ex_rs2)
                    {
                        if self.data_hazard_info {
                            warn!("MEM/WB data hazard detected, for ALU SRC B");
                            warn!("  MEM/WB.rd={}({})", mem_wb_rd, REGNAME[mem_wb_rd as usize]);
                            warn!("  ID/EX.rs1={}({})", id_ex_rs2, REGNAME[id_ex_rs2 as usize]);
                        }
                        // forward B from MEM/WB
                        assert_eq!(self.itl_d_e.forward_b, 0);
                        self.itl_d_e.forward_b = 0b01;
                    }

                    if mem_wb_regwrite
                        && (mem_wb_rd != 0)
                        && ((ex_mem_rd != id_ex_rs1) && (mem_wb_rd == id_ex_rs1)
                            || (ex_mem_rd != id_ex_rs2) && (mem_wb_rd == id_ex_rs2))
                    {
                        self.cpu_statistics.data_hazard_count += 1;
                    }
                    self.itl_m_w.regval
                };
                self.itl_d_e.mem_wb_forward = mem_wb_forward;
            }
        }

        // function units
        if self.pre_pipeline_info {
            info!("MEM/WB {:#x} {:#?}", self.itl_m_w.pc, self.itl_m_w.alu_op);
            info!("EX/MEM {:#x} {:#?}", self.itl_e_m.pc, self.itl_e_m.alu_op);
            info!(
                "ID/EX  {:#x} {:#?}",
                self.itl_d_e.pc, self.itl_d_e.exec_flags.alu_op
            );
            info!(
                "IF/ID  {:#x} {:#?}",
                self.itl_f_d.pc, self.itl_f_d.exec_flags.alu_op
            );
        }
        let running = writeback(&self.itl_m_w, &mut self.reg_file, self.pipeline_info);
        let new_itl_m_w = mem(&self.itl_e_m, &mut self.vm, self.pipeline_info);
        let (new_itl_e_m, new_pc_0, new_pc_1) = exec(
            &self.itl_d_e,
            self.pipeline_info,
            &mut self.callstack,
            Some(&mut self.ras),
        )?;
        let new_itl_d_e = decode(&self.reg_file, &self.itl_f_d, self.pipeline_info);

        // fetch code
        let new_itl_f_d = fetch(
            &self.pc,
            &mut self.vm,
            self.pipeline_info,
            self.control_policy,
            self.bht.as_mut(),
            self.btb.as_ref(),
            Some(&mut self.ras),
        );

        // handle executed branch instruction
        let ex_branch = new_itl_e_m.branch_flags.branch;
        let pc_src = new_itl_e_m.branch_flags.pc_src;
        let predicted_src = new_itl_e_m.branch_flags.predicted_src;

        if ex_branch && self.control_policy == ControlPolicy::DynamicPredict {
            assert!(self.btb.is_some() && self.bht.is_some());
            // warn!(
            //     "UPDATE BTB: PC={:#x}, new_pc_1={:#x}",
            //     new_itl_e_m.pc, new_pc_1
            // );
            use crate::core::insts::Inst64::jalr;
            let is_jalr = new_itl_e_m.alu_op == jalr;
            // fill BTB with potential new entry
            // NOTE: branch target is calculated at EX phase.
            self.btb
                .as_mut()
                .unwrap()
                .add_entry(new_itl_e_m.pc, new_pc_1, is_jalr); // new_pc_1 is branch target
                                                               // update BHT
            self.bht
                .as_mut()
                .unwrap()
                .update_with_result(new_itl_e_m.pc, pc_src);
        }

        // debug!("Before checking misprediction:");
        // debug!("Current itl_e_m instruction: {:?}", new_itl_e_m.alu_op);
        // debug!("ex_branch={ex_branch}");
        // debug!("pc_src={pc_src}");
        // debug!("predicted_src={predicted_src}");
        // debug!("self.itl_e_m.is_ret()={}", new_itl_e_m.is_ret());
        // debug!("new_pc_1={:#x}",new_pc_1);
        // debug!("self.itl_e_m.branch_flags.predicted_target={:#x}",new_itl_e_m.branch_flags.predicted_target);

        // mispredict
        let mispredict = ex_branch
            && ((pc_src != predicted_src)
                || (new_itl_e_m.is_ret() && new_pc_1 != new_itl_e_m.branch_flags.predicted_target));
        if mispredict {
            // compulsory flush
            // so do not use self.x_y_pipeline_states_set
            // instead, set directly
            if self.control_hazard_info {
                warn!("Misprediction detected");
            }
            self.cpu_statistics.control_hazard_count += 1;
            self.cpu_statistics.control_hazard_delayed_cycles += 2;
            self.d_e_pipeline_states[0] = PipelineState::Bubble;
            self.f_d_pipeline_states[0] = PipelineState::Bubble;
            self.pc_next_states[0] = PipelineState::Normal;
        }

        // handle load-use hazard
        if load_use_detected {
            match self.data_hazard_policy {
                DataHazardPolicy::NaiveStall => {}
                DataHazardPolicy::DataForward => {
                    // e_pipeline_state = e_pipeline_state.max(PipelineState::Bubble);
                    self.d_e_pipeline_states_set(&mut [PipelineState::Bubble]);
                    // d_pipeline_state = d_pipeline_state.max(PipelineState::Stall);
                    self.f_d_pipeline_states_set(&mut [PipelineState::Stall]);
                    // f_pipeline_state = f_pipeline_state.max(PipelineState::Stall);
                    self.pc_next_states_set(&mut [PipelineState::Stall]);
                    self.cpu_statistics.data_hazard_delayed_cycles += 1;
                }
            }
        }

        let m_w_pipeline_state = self.m_w_pipeline_states[0];
        let e_m_pipeline_state = self.e_m_pipeline_states[0];
        let d_e_pipeline_state = self.d_e_pipeline_states[0];
        let f_d_pipeline_state = self.f_d_pipeline_states[0];
        let pc_next_state = self.pc_next_states[0];

        let new_itl_m_w = match m_w_pipeline_state {
            PipelineState::Normal => new_itl_m_w,
            PipelineState::Bubble => InternalMemWb::default(),
            PipelineState::Stall => self.itl_m_w,
        };

        let new_itl_e_m = match e_m_pipeline_state {
            PipelineState::Normal => new_itl_e_m,
            PipelineState::Bubble => InternalExecMem::default(),
            PipelineState::Stall => self.itl_e_m,
        };

        let new_itl_d_e = match d_e_pipeline_state {
            PipelineState::Normal => new_itl_d_e,
            PipelineState::Bubble => InternalDecodeExec::default(),
            PipelineState::Stall => self.itl_d_e,
        };

        let new_itl_f_d = match f_d_pipeline_state {
            PipelineState::Normal => new_itl_f_d,
            PipelineState::Bubble => InternalFetchDecode::default(),
            PipelineState::Stall => self.itl_f_d,
        };

        // mul/div/rem
        {
            use Inst64::*;
            match new_itl_e_m.alu_op {
                div | divw | divu | divuw => {
                    self.clock += 39;
                }
                r @ (rem | remw | remu | remuw) => match (r, new_itl_m_w.alu_op) {
                    (rem, div) | (remw, divw) | (remu, divu) | (remuw, divuw)
                        if new_itl_e_m.rs1 == new_itl_m_w.rs1
                            && new_itl_e_m.rs2 == new_itl_m_w.rs2 => {}
                    _ => {
                        self.clock += 39;
                    }
                },
                mul | mulh | mulhsu | mulhu | mulw => {
                    self.clock += 1;
                }
                _ => {}
            }
        }

        // whether executed a non-noop instruction
        if new_itl_e_m.alu_op != Inst64::noop {
            self.cpu_statistics.executed_inst_count += 1;
        }

        // push pipeline forward
        self.itl_m_w = new_itl_m_w;
        self.itl_e_m = new_itl_e_m;
        self.itl_d_e = new_itl_d_e;
        self.itl_f_d = new_itl_f_d;

        let next_pc = match pc_next_state {
            PipelineState::Stall => self.pc.read(),
            PipelineState::Bubble => unreachable!(),
            PipelineState::Normal => {
                if mispredict {
                    // rollback pc
                    // if new_itl_f_d is a branch inst, don't mind it.
                    // because that's a misfetched instruction.
                    if pc_src {
                        // if should taken
                        new_pc_1
                    } else {
                        new_pc_0
                    }
                } else {
                    // normal execution
                    if new_itl_f_d.branch_flags.predicted_src {
                        // Fetch phase decides that predicted
                        new_itl_f_d.branch_flags.predicted_target
                    } else {
                        self.pc.read().wrapping_add(4)
                    }
                }
            }
        };
        // If using NaivePolicy (Stall 3 cycles) then nothing special need to be done.
        // Write back pc
        self.pc.write(next_pc);

        if self.clock_info {
            info!("EX: PC decided {} {:#x}", pc_src, next_pc);
        }

        if self.post_pipeline_info {
            info!("MEM/WB {:#x} {:#?}", self.itl_m_w.pc, self.itl_m_w.alu_op);
            info!("EX/MEM {:#x} {:#?}", self.itl_e_m.pc, self.itl_e_m.alu_op);
            info!(
                "ID/EX  {:#x} {:#?}",
                self.itl_d_e.pc, self.itl_d_e.exec_flags.alu_op
            );
            info!(
                "IF/ID  {:#x} {:#?}",
                self.itl_f_d.pc, self.itl_f_d.exec_flags.alu_op
            );
        }

        // reset x0 to 0
        self.reg_file.write(0, 0);

        // decide whether continue to run
        self.running = running;

        self.m_w_pipeline_states.rotate_left(1);
        self.m_w_pipeline_states[PIPELINE_STATES_DEPTH - 1] = PipelineState::Normal;
        self.e_m_pipeline_states.rotate_left(1);
        self.e_m_pipeline_states[PIPELINE_STATES_DEPTH - 1] = PipelineState::Normal;
        self.d_e_pipeline_states.rotate_left(1);
        self.d_e_pipeline_states[PIPELINE_STATES_DEPTH - 1] = PipelineState::Normal;
        self.f_d_pipeline_states.rotate_left(1);
        self.f_d_pipeline_states[PIPELINE_STATES_DEPTH - 1] = PipelineState::Normal;
        self.pc_next_states.rotate_left(1);
        self.pc_next_states[PIPELINE_STATES_DEPTH - 1] = PipelineState::Normal;
        Ok(())
    }

    #[allow(unused)]
    fn m_w_pipeline_states_set(&mut self, states: &mut [PipelineState]) {
        (0..self.m_w_pipeline_states.len().min(states.len())).for_each(|i| {
            let x = &mut self.m_w_pipeline_states[i];
            *x = *x.max(&mut states[i]);
        });
    }

    #[allow(unused)]
    fn e_m_pipeline_states_set(&mut self, states: &mut [PipelineState]) {
        (0..self.e_m_pipeline_states.len().min(states.len())).for_each(|i| {
            let x = &mut self.e_m_pipeline_states[i];
            *x = *x.max(&mut states[i]);
        });
    }

    fn d_e_pipeline_states_set(&mut self, states: &mut [PipelineState]) {
        (0..self.d_e_pipeline_states.len().min(states.len())).for_each(|i| {
            let x = &mut self.d_e_pipeline_states[i];
            *x = *x.max(&mut states[i]);
        });
    }

    fn f_d_pipeline_states_set(&mut self, states: &mut [PipelineState]) {
        (0..self.f_d_pipeline_states.len().min(states.len())).for_each(|i| {
            let x = &mut self.f_d_pipeline_states[i];
            *x = *x.max(&mut states[i]);
        });
    }

    fn pc_next_states_set(&mut self, states: &mut [PipelineState]) {
        (0..self.pc_next_states.len().min(states.len())).for_each(|i| {
            let x = &mut self.pc_next_states[i];
            *x = *x.max(&mut states[i]);
        });
    }
}

impl<'a> CPU<'a> {
    pub(super) fn pc(&self) -> u64 {
        self.pc.read()
    }

    pub(super) fn mread<T: Sized + std::fmt::Display>(&self, vaddr: u64) -> T {
        self.vm.mread(vaddr as usize)
    }

    pub(super) fn backtrace(&self) {
        self.callstack.backtrace();
    }

    pub(super) fn reg_val_by_name(&self, name: &str) -> Result<u64> {
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

#[allow(unused)]
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

pub struct MultistageCPU<'a> {
    // indicate whether the CPU is running
    running: bool,

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

    cpu_statistics: CPUStatistics,

    last_inst_info: LastInstInfo,
}

struct LastInstInfo {
    alu_op: Inst64,
    rs1: u8,
    rs2: u8,
}

impl LastInstInfo {
    pub fn new() -> Self {
        Self {
            alu_op: Inst64::noop,
            rs1: 0,
            rs2: 0,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new()
    }
}

impl<'a> MultistageCPU<'a> {
    pub fn new(
        vm: &'a mut VirtualMemory,
        callstack: &'a mut CallStack<'a>,
        itrace: bool,
    ) -> MultistageCPU<'a> {
        // x0 already set to 0
        let reg_file = RegisterFile::empty();
        let pc = ProgramCounter::new();

        MultistageCPU {
            running: false,
            // continue_fetch: true,
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
            cpu_statistics: CPUStatistics::default(),
            last_inst_info: LastInstInfo::new(),
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
            self.exec_once()?;
            i += 1;
        }

        Ok(())
    }

    pub fn print_info(&self) {
        info!("CPU run clock: {}", self.clock);
        info!(
            "CPU data hazard count: {}",
            self.cpu_statistics.data_hazard_count
        );
        info!(
            "CPU data hazard delayed cycles: {}",
            self.cpu_statistics.data_hazard_delayed_cycles
        );
        info!(
            "CPU control hazard count: {}",
            self.cpu_statistics.control_hazard_count
        );
        info!(
            "CPU control hazard delayed cycles: {}",
            self.cpu_statistics.control_hazard_delayed_cycles
        );
        info!(
            "CPU executed valid instructions: {}",
            self.cpu_statistics.executed_inst_count
        );
        info!("CPI = {}", {
            let cycles = self.clock;
            let insts = self.cpu_statistics.executed_inst_count;
            (cycles as f64) / (insts as f64)
        });
    }

    pub(super) fn exec_once(&mut self) -> Result<()> {
        use crate::core::insts::Inst64::*;

        // fetch code
        self.clock += 1;
        let new_itl_f_d = fetch(
            &self.pc,
            &mut self.vm,
            self.itrace,
            ControlPolicy::AlwaysNotTaken,
            None,
            None,
            None,
        );
        self.itl_f_d = new_itl_f_d;

        self.clock += 1;
        let new_itl_d_e = decode(&self.reg_file, &self.itl_f_d, self.itrace);
        self.itl_d_e = new_itl_d_e;

        self.clock += 1;
        let (new_itl_e_m, new_pc_0, new_pc_1) =
            exec(&self.itl_d_e, self.itrace, &mut self.callstack, None)?;
        self.itl_e_m = new_itl_e_m;

        match new_itl_e_m.alu_op {
            d @ (div | divu | divuw | divw) => {
                self.last_inst_info.alu_op = d;
                self.last_inst_info.rs1 = new_itl_e_m.rs1;
                self.last_inst_info.rs2 = new_itl_e_m.rs2;
                self.clock += 39;
            }
            r @ (rem | remu | remuw | remw) => {
                match (self.last_inst_info.alu_op, r) {
                    (div, rem) | (divu, remu) | (divuw, remuw) | (divw, remw)
                        if self.last_inst_info.rs1 == new_itl_e_m.rs1
                            && self.last_inst_info.rs2 == new_itl_e_m.rs2 => {}
                    _ => {
                        self.clock += 39;
                    }
                }
                self.last_inst_info.clear();
            }
            mul | mulh | mulhsu | mulhu | mulw => {
                self.clock += 1;
                self.last_inst_info.clear();
            }
            _ => {
                self.last_inst_info.clear();
            }
        }
        // whether executed a non-noop instruction
        if new_itl_e_m.alu_op != Inst64::noop {
            self.cpu_statistics.executed_inst_count += 1;
        }

        if self.itl_e_m.mem_flags.mem_read || self.itl_e_m.mem_flags.mem_write {
            // begin the clock
            self.clock += 1;
        }
        let new_itl_m_w = mem(&self.itl_e_m, &mut self.vm, self.itrace);
        self.itl_m_w = new_itl_m_w;

        if self.itl_m_w.wb_flags.mem_to_reg {
            // begin the clock
            self.clock += 1;
        }
        let running = writeback(&self.itl_m_w, &mut self.reg_file, self.itrace);

        let next_pc = if new_itl_e_m.branch_flags.pc_src {
            new_pc_1
        } else {
            new_pc_0
        };

        self.pc.write(next_pc);

        // reset x0 to 0
        self.reg_file.write(0, 0);

        // decide whether continue to run
        self.running = running;

        Ok(())
    }
}
