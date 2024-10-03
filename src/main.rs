use std::path;

use clap::Parser;
use cpu::CPU;
use elf::read_elf;
use log::info;
use reg::RegisterFile;
use vm::VirtualMemory;

mod alu;
mod cpu;
mod decode;
mod elf;
mod error;
mod insts;
mod logger;
mod reg;
mod vm;

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Args {
    /// Path to the program to be loaded
    #[arg(short, long)]
    input: String,

    /// Enable debug mode. Not set to enable batch mode.
    #[arg(short, long)]
    debug: bool,
}

fn main() {
    // log4rs::init_file("config/log4rs.yaml", Default::default())
    //     .expect("Fail to load logger configuration");
    logger::init();

    let args = Args::parse();
    let file_path = path::PathBuf::from(&args.input);

    info!("Loading file: {file_path:?}");

    // Parse ELF file
    let elf_info = read_elf(&file_path).expect("Fail to load ELF");

    // Load the file into virtual memory
    let mut vm = VirtualMemory::from_elf_info(&elf_info);

    let mut cpu = CPU::new(&mut vm);

    cpu.init_elfinfo_64(&elf_info);

    cpu.cpu_exec().expect("Failed to execute the program");

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
