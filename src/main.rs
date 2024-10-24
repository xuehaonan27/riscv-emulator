use callstack::CallStack;
use clap::{Parser, ValueEnum};
use core::vm::VirtualMemory;
use elf::read_elf;
use log::info;
use multi_stage::cpu::{ControlPolicy, DataHazardPolicy};
use std::path;

mod callstack;
mod core;
mod elf;
mod error;
mod logger;
mod multi_stage;
mod single_cycle;

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Args {
    /// Path to the program to be loaded
    #[arg(short, long)]
    input: String,

    /// CPU mode
    #[arg(short, long)]
    cpu_mode: CPUMode,

    /// Enable debug mode. Not set to enable batch mode.
    #[arg(short, long)]
    debug: bool,

    /// Enable itrace.
    #[arg(long)]
    itrace: bool,

    /// Enable mtrace.
    #[arg(long)]
    mtrace: bool,

    /// Enable ftrace.
    #[arg(long)]
    ftrace: bool,

    /// Data hazard policy
    #[arg(long)]
    data_hazard_policy: DataHazardPolicy,

    /// Control policy
    #[arg(long)]
    control_policy: ControlPolicy,

    // Pre-execution pipeline register info
    #[arg(long)]
    pre_pipeline_info: bool,

    // Pipeline info
    #[arg(long)]
    pipeline_info: bool,

    // Post-execution pipeline register info
    #[arg(long)]
    post_pipeline_info: bool,

    // Control hazard info
    #[arg(long)]
    control_hazard_info: bool,

    // Data hazard info
    #[arg(long)]
    data_hazard_info: bool,
}

#[derive(Debug, Clone, ValueEnum)]
enum CPUMode {
    Single,
    Multi,
    Pipeline,
}

fn main() {
    // log4rs::init_file("config/log4rs.yaml", Default::default())
    //     .expect("Fail to load logger configuration");
    logger::init();

    let args = Args::parse();
    let file_path = path::PathBuf::from(&args.input);
    let enable_debug_mode = args.debug;
    let itrace = args.itrace;
    let mtrace = args.mtrace;
    let ftrace = args.ftrace;
    let cpu_mode = args.cpu_mode;
    let data_hazard_policy = args.data_hazard_policy;
    let control_policy = args.control_policy;
    let pre_pipeline_info = args.pre_pipeline_info;
    let pipeline_info = args.pipeline_info;
    let post_pipeline_info = args.post_pipeline_info;
    let control_hazard_info = args.control_hazard_info;
    let data_hazard_info = args.data_hazard_info;
    info!("Loading file: {file_path:?}");

    // Parse ELF file
    let elf_info = read_elf(&file_path).expect("Fail to load ELF");

    // Load the file into virtual memory
    let mut vm = VirtualMemory::from_elf_info(&elf_info, mtrace);

    // Create call stack for the running process on the CPU
    let mut callstack = CallStack::from_elf_info(&elf_info, ftrace);

    match cpu_mode {
        CPUMode::Single => {
            use single_cycle::{cpu::CPU, debug::REDB};
            let mut cpu = CPU::new(&mut vm, &mut callstack, itrace);

            cpu.init_elfinfo_64(&elf_info);

            if !enable_debug_mode {
                cpu.cpu_exec(None).expect("Failed to execute the program");
            } else {
                let mut redb = REDB::new(&mut cpu);
                redb.run();
            }
        }
        CPUMode::Multi => {}
        CPUMode::Pipeline => {
            use multi_stage::{cpu::CPU, debug::REDB};
            let mut cpu = CPU::new(
                &mut vm,
                &mut callstack,
                itrace,
                data_hazard_policy,
                control_policy,
                pre_pipeline_info,
                pipeline_info,
                post_pipeline_info,
                control_hazard_info,
                data_hazard_info,
            );

            cpu.init_elfinfo_64(&elf_info);

            if !enable_debug_mode {
                cpu.cpu_exec(None).expect("Failed to execute the program");
                cpu.print_info();
            } else {
                let mut redb = REDB::new(&mut cpu);
                redb.run();
            }
        }
    }

    // Atomatically drop all resources
}

#[macro_export]
macro_rules! check {
    ($x:expr, $fmt: expr $(, $($arg: tt)+)?) => {
        if !($x) {
            log::error!($fmt);
        }
    };
}
