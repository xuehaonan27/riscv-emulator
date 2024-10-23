use callstack::CallStack;
use clap::{Parser, ValueEnum};
use core::vm::VirtualMemory;
use elf::read_elf;
use log::info;
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
            use multi_stage::cpu::CPU;
            let mut cpu = CPU::new(&mut vm, &mut callstack, itrace);

            cpu.init_elfinfo_64(&elf_info);

            cpu.cpu_exec(None).expect("Failed to execute the program");
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
