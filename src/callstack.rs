use std::collections::{HashMap, VecDeque};
use std::iter;

use log::trace;

use crate::elf::LoadElfInfo;

pub struct CallStack<'a> {
    symbol_map: &'a HashMap<u64, String>,
    call_stack: VecDeque<(u64, String)>,
    pub ftrace: bool,
}

impl<'a> CallStack<'a> {
    pub fn new(symbol_map: &HashMap<u64, String>, ftrace: bool) -> CallStack {
        CallStack {
            symbol_map,
            call_stack: VecDeque::new(),
            ftrace,
        }
    }

    pub fn from_elf_info(info: &LoadElfInfo, ftrace: bool) -> CallStack {
        CallStack::new(info.symbol_map(), ftrace)
    }

    pub fn call(&mut self, pc: u64, target_pc: u64) {
        if let Some(func_name) = self.symbol_map.get(&target_pc) {
            let len = self.call_stack.len();
            if self.ftrace {
                trace!(
                    "{:x}:{} call [{func_name}@{:#x}]",
                    pc,
                    iter::repeat(' ').take(len).collect::<String>(),
                    target_pc
                );
            }
            self.call_stack.push_back((pc, func_name.clone()));
        }
    }

    pub fn ret(&mut self, pc: u64) {
        if let Some((_, func_name)) = self.call_stack.pop_back() {
            let len = self.call_stack.len();
            trace!(
                "{:x}:{} ret [{func_name}]",
                pc,
                iter::repeat(' ').take(len).collect::<String>()
            );
        }
    }

    pub fn backtrace(&self) {
        for (i, (pc, func_name)) in self.call_stack.iter().enumerate() {
            println!("{} {:#x}: {}", i, pc, func_name);
        }
    }
}
