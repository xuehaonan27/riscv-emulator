use clap::{Parser, Subcommand};
use std::io::{self, BufRead, Write};

use crate::cpu::CPU;
use crate::error::{Error, Result};

const REDB_BUF_SIZE: usize = 64;

pub struct REDB {
    // Command line input buffer
    buf: String,
    //
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about, disable_help_flag = true)]
struct DebugArgs {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(alias = "help")]
    Helping,
    #[clap(alias = "c")]
    Continue,
    #[clap(alias = "q")]
    Quit,
    #[clap(alias = "si")]
    Step {
        #[clap(default_value_t = 1)]
        n: i32,
    },
    Info {
        r: String,
    },
    #[clap(alias = "x")]
    Scan {
        n: u64,
        // #[clap(value_parser=maybe_hex::<u64>)]
        vaddr: u64,
    },
}

impl REDB {
    pub fn new() -> REDB {
        REDB {
            buf: String::with_capacity(REDB_BUF_SIZE),
        }
    }

    pub fn run(&mut self, cpu: &mut CPU) {
        loop {
            print!("(REDB)>>> ");
            io::stdout().flush().expect("Fail to flush");
            let cmd = match self.listen() {
                Ok(cmd) => cmd,
                Err(e) => {
                    println!("{}", e);
                    continue;
                }
            };
            if cmd.is_none() {
                continue;
            }
            match cmd.unwrap() {
                Commands::Helping => print_help_info(),
                Commands::Continue => match cpu.cpu_exec(None) {
                    Ok(_) => {
                        println!("REDB: CPU executed to end.");
                        break;
                    }
                    Err(e) => {
                        println!("REDB: CPU raised exception: {}", e);
                        continue;
                    }
                },
                Commands::Quit => {
                    println!("REDB: Exit REDB");
                    break;
                }
                Commands::Step { n } => {
                    if n.is_negative() {
                        println!("REDB: steps cannot be negative");
                    }
                    println!("REDB: execute {n} steps");
                    for i in 1..=n {
                        if let Err(e) = cpu.exec_once() {
                            println!("REDB: stopped after executed {i} steps");
                            println!("{e}");
                            break;
                        }
                    }
                    println!("REDB: executed {n} steps");
                }
                Commands::Info { r } => match cpu.reg_val_by_name(&r) {
                    Ok(reg) => {
                        println!("{}\t: {}", r, reg);
                    }
                    Err(e) => {
                        println!("REDB: {e}");
                    }
                },
                Commands::Scan { n, vaddr } => {
                    for i in vaddr..(vaddr + n) {
                        let val = cpu.mread::<u64>(vaddr);
                        println!("{:#x}: {:#x}", i, val);
                    }
                }
            }
        }
    }

    // Listen for user's input
    fn listen(&mut self) -> Result<Option<Commands>> {
        self.buf.clear();
        let stdin = io::stdin();
        let mut stdin = stdin.lock();
        stdin.read_line(&mut self.buf)?;

        let buf = self.buf.trim();
        if buf.is_empty() {
            return Ok(None);
        }

        let mut itr: Vec<&str> = self.buf.split_whitespace().collect();
        itr.insert(0, "DebugArgs");
        let dbargs = DebugArgs::try_parse_from(itr).map_err(|e| Error::DbgParse(e.to_string()))?;
        Ok(Some(dbargs.command))
    }
}

fn print_help_info() {
    println!("Help")
}
