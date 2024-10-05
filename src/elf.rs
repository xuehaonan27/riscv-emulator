use std::{fs, ops::Range, path::PathBuf};

use goblin::elf::{header, program_header, Elf};
use log::{debug, error, info};

use crate::error::{Error, Result};

pub struct LoadElfInfo {
    raw_data: Vec<u8>,
    is_64_bit: bool,
    entry_point: u64,
    vm_ranges: Vec<Range<usize>>,
    file_ranges: Vec<Range<usize>>,
    min_vaddr: usize,
    max_vaddr: usize,
}

impl LoadElfInfo {
    pub fn raw_data(&self) -> &Vec<u8> {
        &self.raw_data
    }

    pub fn is_64_bit(&self) -> bool {
        self.is_64_bit
    }

    pub fn entry_point(&self) -> u64 {
        self.entry_point
    }

    pub fn vm_ranges(&self) -> &Vec<Range<usize>> {
        &self.vm_ranges
    }

    pub fn file_ranges(&self) -> &Vec<Range<usize>> {
        &self.file_ranges
    }

    pub fn min_vaddr(&self) -> usize {
        self.min_vaddr
    }

    pub fn max_vaddr(&self) -> usize {
        self.max_vaddr
    }
}

pub fn read_elf(path: &PathBuf) -> Result<LoadElfInfo> {
    let raw_data = fs::read(path)?;
    let elf = Elf::parse(&raw_data)?;

    /*
    {
        trace!("ELF Header:");
        trace!("  Class: {}", elf.header.e_ident[4]);
        trace!("  Data: {}", elf.header.e_ident[5]);
        trace!("  Version: {}", elf.header.e_ident[6]);
        trace!("  OS/ABI: {}", elf.header.e_ident[7]);
        trace!("  ABI Version: {}", elf.header.e_ident[8]);
        trace!("  Type: {}", elf.header.e_type);
        trace!("  Machine: {}", elf.header.e_machine);
        trace!("  Entry point: {:#x}", elf.header.e_entry);
        trace!("  Program headers offset: {:#x}", elf.header.e_phoff);
        trace!("  Section headers offset: {:#x}", elf.header.e_shoff);
        trace!("  Flags: {:#x}", elf.header.e_flags);
        trace!("  Header size: {}", elf.header.e_ehsize);
        trace!("  Program header entry size: {}", elf.header.e_phentsize);
        trace!("  Program header count: {}", elf.header.e_phnum);
        trace!("  Section header entry size: {}", elf.header.e_shentsize);
        trace!("  Section header count: {}", elf.header.e_shnum);
        trace!("  String table index: {}", elf.header.e_shstrndx);
        trace!("");
    }
    {
        trace!("Program Headers:");
        for ph in &elf.program_headers {
            trace!("  Type: {:#x}", ph.p_type);
            trace!("  Offset: {:#x}", ph.p_offset);
            trace!("  Virtual Address: {:#x}", ph.p_vaddr);
            trace!("  Physical Address: {:#x}", ph.p_paddr);
            trace!("  File Size: {:#x}", ph.p_filesz);
            trace!("  Memory Size: {:#x}", ph.p_memsz);
            trace!("  Flags: {:#x}", ph.p_flags);
            trace!("  Alignment: {:#x}", ph.p_align);
            trace!("")
        }
    }
    {
        trace!("Section Headers:");
        for sh in &elf.section_headers {
            trace!(
                "  Name: {}",
                elf.shdr_strtab.get_at(sh.sh_name).unwrap_or(&"")
            );
            trace!("  Type: {:#x}", sh.sh_type);
            trace!("  Flags: {:#x}", sh.sh_flags);
            trace!("  Address: {:#x}", sh.sh_addr);
            trace!("  Offset: {:#x}", sh.sh_offset);
            trace!("  Size: {:#x}", sh.sh_size);
            trace!("  Link: {}", sh.sh_link);
            trace!("  Info: {}", sh.sh_info);
            trace!("  Address Alignment: {:#x}", sh.sh_addralign);
            trace!("  Entry Size: {:#x}", sh.sh_entsize);
            trace!("")
        }
    }
    */

    // Validity check
    if elf.header.e_machine != header::EM_RISCV {
        let msg = format!(
            "Not a RISC-V target ELF: {}, expected: {}",
            elf.header.e_machine,
            header::EM_RISCV
        );
        error!("{msg}");
        return Err(Error::InvalidElf(msg));
    }

    // Bits check
    let is_64_bit = elf.is_64;
    info!(
        "ELF file {path:?} is {} bit",
        if is_64_bit { "64" } else { "32" }
    );

    // entry point
    let entry_point = elf.header.e_entry;
    // fetch loadable ranges
    let mut vm_ranges = Vec::new();
    let mut file_ranges = Vec::new();
    let mut min_vaddr = usize::MAX;
    let mut max_vaddr = usize::MIN;
    let mut min_offset = usize::MAX;
    let mut max_offset = usize::MIN;
    for ph in &elf.program_headers {
        // Loadable section
        if ph.p_type == program_header::PT_LOAD {
            if ph.p_filesz == 0 {
                continue;
            }
            let vm_range = ph.vm_range();
            let file_range = ph.file_range();
            debug!("vm_range: {:#x?}", vm_range);
            debug!("file_range: {:#x?}", file_range);

            let start_vaddr = vm_range.start;
            let end_vaddr = vm_range.end;
            let start_offset = file_range.start;
            let end_offset = file_range.end;

            vm_ranges.push(vm_range);
            file_ranges.push(file_range);
            if start_vaddr < min_vaddr {
                min_vaddr = start_vaddr;
            }
            if end_vaddr > max_vaddr {
                max_vaddr = end_vaddr;
            }
            if start_offset < min_offset {
                min_offset = start_offset;
            }
            if end_offset > max_offset {
                max_offset = end_offset;
            }
        }
    }

    let info = LoadElfInfo {
        raw_data,
        is_64_bit,
        entry_point,
        vm_ranges,
        file_ranges,
        min_vaddr,
        max_vaddr,
    };
    Ok(info)
}
