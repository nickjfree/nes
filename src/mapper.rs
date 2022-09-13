use std::fmt::Debug;
use crate::board::Memory;

pub const PRG_BANK_SIZE: usize = 16 * 1024;
pub const CHR_BANK_SIZE: usize = 8 * 1024;



pub trait Mapper {
	fn read_u8(&mut self, addr: u16) -> u8;
	fn write_u8(&mut self, addr: u16, val: u8);
}


#[derive(Debug)]
pub enum MirroMode {
	Vertical,
	Horizontal,
	FourScreen,
}


#[derive(Debug)]
struct NameTable {
	name_table: Memory,
	mode: MirroMode,
}


impl NameTable {

	fn new(mode: MirroMode) -> Self {
		Self {
			name_table: Memory::new(4096),
			mode: mode,
		}
	}

	fn tanslate_addr(&self, addr: u16) -> u16 {
		match self.mode {
			MirroMode::Horizontal => addr & 0x0bff,
			MirroMode::Vertical => addr & 0x07ff,
			MirroMode::FourScreen => addr & 0x0fff,
		}
	}
}


impl NameTable {

	fn read_u8(&mut self, addr: u16) -> u8 {
		//println!("read nt {:#06x}", addr);
		let addr = self.tanslate_addr(addr);
		self.name_table.read_u8(addr)
	}

	fn write_u8(&mut self, addr: u16, val: u8) {
		//println!("write nt {:#06x}", addr);
		let addr = self.tanslate_addr(addr);
		self.name_table.write_u8(addr, val);
	}
}


// mapper 0
#[derive(Debug)]
pub struct NRom {

	// cpu prg rom
	prg: Memory,

	// ppu pattern table
	chr: Memory,

	// nametable
	name_table: NameTable,
}


impl NRom {
	pub fn new(prg: Memory, chr: Memory, mode: MirroMode) -> Self {
		Self {
			prg: prg,
			chr: chr,
			name_table: NameTable::new(mode),
		}
	}
}


impl Mapper for NRom {

	fn read_u8(&mut self, addr: u16) -> u8 {
		match addr {
			0x0000..=0x1fff => self.chr.read_u8(addr),
			0x2000..=0x3eff => self.name_table.read_u8(addr - 0x2000),
			0x8000..=0xffff => {
				let addr = addr % (self.prg.size() as u16);
				self.prg.read_u8(addr)
			}
			_ => 0,
		}
	}

	fn write_u8(&mut self, addr: u16, val: u8) {
		match addr {
			0x0000..=0x1fff => self.chr.write_u8(addr, val),
			0x2000..=0x3eff => self.name_table.write_u8(addr - 0x2000, val),
			0x8000..=0xffff => {
				let addr = addr % (self.prg.size() as u16);
				self.prg.write_u8(addr, val);
			}
			_ => (),
		}
	}
}
