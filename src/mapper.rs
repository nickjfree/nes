use std::fmt::Debug;
use crate::board::Memory;
use crate::board::{ Signal };


pub const PRG_BANK_SIZE: usize = 16 * 1024;
pub const CHR_BANK_SIZE: usize = 8 * 1024;



pub trait Mapper {
	fn read_u8(&mut self, addr: u16) -> u8;
	fn write_u8(&mut self, addr: u16, val: u8);
}


#[derive(Debug)]
pub enum MirroMode {
	Single,
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
			MirroMode::Single => addr & 0x03ff,
			MirroMode::Horizontal => (addr & 0x03ff) | ((addr & 0x0800) >> 1),
			MirroMode::Vertical => addr & 0x07ff,
			MirroMode::FourScreen => addr & 0x0fff,
		}
	}
}


impl NameTable {

	fn read_u8(&mut self, addr: u16) -> u8 {
		let addr = self.tanslate_addr(addr);
		self.name_table.read_u8(addr)
	}

	fn write_u8(&mut self, addr: u16, val: u8) {
		let addr = self.tanslate_addr(addr);
		self.name_table.write_u8(addr, val);
	}

	fn set_mirror_mode(&mut self, mode: MirroMode) {
		self.mode = mode
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
			// some nrom rom has CHR ram. so make CHR writable
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


// mapper 2
#[derive(Debug)]
pub struct UxRom {

	// CPU $8000-$BFFF: 16 KB switchable PRG ROM bank
	// CPU $C000-$FFFF: 16 KB PRG ROM bank, fixed to the last bank
	prg: Memory,

	// ppu pattern table
	chr: Memory,

	// bank select register
	select: u8,

	// total banks
	banks: usize,

	// nametable
	name_table: NameTable,

}


impl UxRom {
	pub fn new(prg: Memory, _chr: Memory, mode: MirroMode) -> Self {
		let banks = prg.size() / PRG_BANK_SIZE;
		Self {
			prg: prg,
			chr: Memory::new(8192),
			name_table: NameTable::new(mode),
			select: 0,
			banks: banks,
		}
	}
}


impl Mapper for UxRom {

	fn read_u8(&mut self, addr: u16) -> u8 {
		match addr {
			0x0000..=0x1fff => self.chr.read_u8(addr),
			0x2000..=0x3eff => self.name_table.read_u8(addr - 0x2000),
			0x8000..=0xbfff => {
				let addr = (addr - 0x8000) as usize | ((self.select as usize) << 14);
				self.prg[addr]
			},
			0xc000..=0xffff => {
				let addr = ((self.banks - 1) * PRG_BANK_SIZE) | (addr - 0xc000) as usize;
				self.prg[addr]
			}
			_ => 0,
		}
	}

	fn write_u8(&mut self, addr: u16, val: u8) {
		match addr {
			0x0000..=0x1fff => self.chr.write_u8(addr, val),
			0x2000..=0x3eff => self.name_table.write_u8(addr - 0x2000, val),
			0x8000..=0xffff => {
				self.select = ((val as usize) % self.banks) as u8;
			}
			_ => (),
		}
	}
}


// mapper 4
#[derive(Debug)]
pub struct MMC3 {
	// prg rom
	prg: Memory,
	prg_banks: usize,
	// ppu pattern table
	chr: Memory,
	// nametable
	name_table: NameTable,
	// register select
	reg_select: u8,
	// chr and prg bank selectors
	regs: [u8; 8],
	// prg inversion
	prg_bank_mode: u8,
	// chr A12 inversion
	chr_inversion: bool,
	// irq signal line
	irq: Signal,
	// irq functions
	irq_reload_value: u8,
	irq_counter: u8,
	irq_enabled: bool,
	prev_a12: u16,
}


impl MMC3 {
	pub fn new(prg: Memory, chr: Memory, mode: MirroMode, irq: Signal) -> Self {
		let banks = prg.size() / 8192;
		Self {
			prg: prg,
			prg_banks: banks,
			chr: chr,
			name_table: NameTable::new(mode),
			reg_select: 0,
			regs: [0; 8],
			prg_bank_mode: 0,
			chr_inversion: false,
			irq: irq,
			irq_reload_value: 0,
			irq_counter: 0,
			irq_enabled: false,
			prev_a12: 0,
		}
	}
}


impl Mapper for MMC3 {

	fn read_u8(&mut self, addr: u16) -> u8 {
		match addr {
			// pattern_table
			0x0000..=0x1fff => {
				// irq A12 handle
				let a12 = addr & 0x1000;
				if self.prev_a12 == 0 && a12 != 0 {
					// a12 low -> high
					match self.irq_counter {
						0 => self.irq_counter = self.irq_reload_value,
						_ => {
							self.irq_counter -= 1;
							if self.irq_counter == 0 && self.irq_enabled {
								*self.irq.borrow_mut() = 1;
							}
						},
					}
				}
				self.prev_a12 = a12;

				let real_addr: usize = if !self.chr_inversion {
					match addr {
						// 0
						0x0000..=0x07ff => (addr - 0x0000) as usize + ((self.regs[0] as usize) << 10),
						0x0800..=0x0fff => (addr - 0x0800) as usize + ((self.regs[1] as usize) << 10),
						// 1
						0x1000..=0x13ff => (addr - 0x1000) as usize + ((self.regs[2] as usize) << 10),
						0x1400..=0x17ff => (addr - 0x1400) as usize + ((self.regs[3] as usize) << 10),
						0x1800..=0x1bff => (addr - 0x1800) as usize + ((self.regs[4] as usize) << 10),
						0x1c00..=0x1fff => (addr - 0x1c00) as usize + ((self.regs[5] as usize) << 10),
						_ => panic!("bad mmc3 addr {:#06x}", addr),
					}
				} else {
					match addr {
						// 0
						0x0000..=0x03ff => (addr - 0x0000) as usize + ((self.regs[2] as usize) << 10),
						0x0400..=0x07ff => (addr - 0x0400) as usize + ((self.regs[3] as usize) << 10),
						0x0800..=0x0bff => (addr - 0x0800) as usize + ((self.regs[4] as usize) << 10),
						0x0c00..=0x0fff => (addr - 0x0c00) as usize + ((self.regs[5] as usize) << 10),
						// 1
						0x1000..=0x17ff => (addr - 0x1000) as usize + ((self.regs[0] as usize) << 10),
						0x1800..=0x1fff => (addr - 0x1800) as usize + ((self.regs[1] as usize) << 10),
						_ => panic!("bad mmc3 addr {:#06x}", addr),
					}
				};
				self.chr[real_addr]
			},
			0x2000..=0x3eff => self.name_table.read_u8(addr - 0x2000),
			0x8000..=0xffff => {
				let real_addr: usize = if self.prg_bank_mode == 0 {
					match addr {
						// 0
						0x8000..=0x9fff => (addr - 0x8000) as usize + ((self.regs[6] as usize) << 13),
						0xa000..=0xbfff => (addr - 0xa000) as usize + ((self.regs[7] as usize) << 13),
						// 1
						0xc000..=0xdfff => (addr - 0xc000) as usize + ((self.prg_banks - 2) << 13),
						0xe000..=0xffff => (addr - 0xe000) as usize + ((self.prg_banks - 1) << 13),
						_ => panic!("bad mmc3 addr {:#06x}", addr),
					}
				} else {
					match addr {
						// 0
						0x8000..=0x9fff => (addr - 0x8000) as usize + ((self.prg_banks - 2) << 13),
						0xa000..=0xbfff => (addr - 0xa000) as usize + ((self.regs[7] as usize) << 13),
						// 1
						0xc000..=0xdfff => (addr - 0xc000) as usize + ((self.regs[6] as usize) << 13),
						0xe000..=0xffff => (addr - 0xe000) as usize + ((self.prg_banks - 1) << 13),
						_ => panic!("bad mmc3 addr {:#06x}", addr),
					}
				};
				self.prg[real_addr]
			},
			_ => 0,
		}
	}

	fn write_u8(&mut self, addr: u16, val: u8) {
		match addr {
			0x0000..=0x1fff => (),
			0x2000..=0x3eff => self.name_table.write_u8(addr - 0x2000, val),
			_ => {
				let even = addr % 2 == 0;
				if even {
					match addr {
						// Bank select ($8000-$9FFE, even)
						0x8000..=0x9ffe => {
							self.reg_select = val & 0x07;
							self.prg_bank_mode = (val & 0x40) >> 6;
							self.chr_inversion = (val & 0x80) != 0;
						},
						// Mirroring ($A000-$BFFE, even)
						0xa000..=0xbffe => {
							let mirror_mode = match val & 0x01 {
								0 => MirroMode::Vertical,
								_ => MirroMode::Horizontal,
							};
							self.name_table.set_mirror_mode(mirror_mode);
						},
						// IRQ latch ($C000-$DFFE, even)
						0xc000..=0xdffe => {
							self.irq_reload_value = val;
							// self.irq_counter = 0;
						}
						// IRQ disable ($E000-$FFFE, even)
						0xe000..=0xfffe => {
							self.irq_enabled = false;
						},
						_ => (),
					}
				} else {
					match addr {
						// Bank data ($8001-$9FFF, odd)
						0x8001..=0x9fff => {
							// R6 and R7 will ignore the top two bits
							// R0 and R1 ignore the bottom bit
							let val = match self.reg_select {
								6 | 7 => val & 0x3f,
								0 | 1 => val & 0xfe,
								_ => val,
							};
							self.regs[self.reg_select as usize] = val;
						},
						// PRG RAM protect ($A001-$BFFF, odd)
						0xa001..=0xbfff => {
							// do nothing for now
						},
						// IRQ reload ($C001-$DFFF, odd)
						0xc001..=0xdfff => {
							self.irq_counter = 0;
						},
						// IRQ enable ($E001-$FFFF, odd)
						0xe001..=0xffff => {
							self.irq_enabled = true;
						},
						_ => (),
					}
				}
			}
		}	
	}
}
