use std::ops::{ Deref, DerefMut};
use std::cell::RefCell;
use std::rc::Rc;
use crate::cartridge::Cartridge;

// memory
#[derive(Debug)]
pub struct Ram {
	data: Vec<u8>,
}


impl Ram {
	pub fn new(size: usize) -> Self {
		Self {
			data: vec![0; size],
		}
	}
}


impl Deref for Ram {

	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl DerefMut for Ram {


	fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

impl Ram {

	pub fn reset(&mut self) {
		self.data.iter_mut().map(|x| *x = 0).count();
	}
}


trait Mapper{}


#[derive(Default)]
pub struct IORegisters {
	data: Vec<u8>,
}


impl Deref for IORegisters {

	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl DerefMut for IORegisters {

	fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}


impl IORegisters {
	pub fn new(size: usize) -> Self {
		Self {
			data: vec![0; size],
		}
	}
}


// the bus
#[derive(Default)]
pub struct CPUBus {
	// 0000-07FF 2K * 4
	internal_ram: Option<Ram>,
	// 0800-0fff
	// 1000-17ff
	// 1800-1fff

	// 2000-2007
	ppu_regs: Rc<RefCell<IORegisters>>,

	// 4020-5fff 8K-20h
	rom: Option<Ram>,
	// 6000-7fff 8K
	sram: Option<Ram>,
	// 8000-bfff 16K
	// c000-ffff 16K
	cartridge: Rc<RefCell<Cartridge>>,
}


impl CPUBus {

	pub fn new(ppu_regs: Rc<RefCell<IORegisters>>, cartridge: Rc<RefCell<Cartridge>>) -> Self {
		// internal ram
		Self {
			internal_ram: Some(Ram::new(8192)),
			ppu_regs: ppu_regs,
			rom: Some(Ram::new(8159)),
			sram: Some(Ram::new(8192)),
			cartridge: cartridge,
		}
	}

	// load address
	pub fn read_u8(&self, addr: u16) -> u8 {

		match addr {
			0x0000..=0x1fff => {
				if let Some(mem) = &self.internal_ram {
					mem[usize::from(addr&0x7ff)]
				} else {
					0
				}
			},
			0x2000..=0x3fff => {
				let addr = (addr - 0x2000) & 0x0F;
				self.ppu_regs.borrow()[usize::from(addr)]
			},
			0x4020..=0x5fff => {
				if let Some(mem) = &self.rom {
					mem[usize::from(addr-0x4020)]
				} else {
					0
				}
			},
			0x6000..=0x7fff => {
				if let Some(mem) = &self.sram {
					mem[usize::from(addr-0x6000)]
				} else {
					0
				}
			},
			0x8000..=0xbfff => {
				self.cartridge.borrow().program(0)[usize::from(addr-0x8000)]
			},
			0xc000..=0xffff => {
				self.cartridge.borrow().program(1)[usize::from(addr-0xc000)]
			},
			_=> panic!("addr not mapped yet {:#04x}", addr),
		}
	}

	pub fn read_u16(&self, addr: u16) -> u16 {
		let l = self.read_u8(addr);
		let h = self.read_u8(addr.wrapping_add(1));
		(h as u16) << 8 | l as u16
	}

	pub fn write_u8(&mut self, addr: u16, data: u8) {
		match addr {
			0x0000..=0x1fff => {
				if let Some(mem) = &mut self.internal_ram {
					mem[usize::from(addr&0x7ff)] = data
				}
			},
			0x2000..=0x3fff => {
				let addr = (addr - 0x2000) & 0x0F;
				self.ppu_regs.borrow_mut()[usize::from(addr)] = data
			},
			0x4020..=0x5fff => {
				if let Some(mem) = &mut self.rom {
					mem[usize::from(addr-0x4020)] = data
				}
			},
			0x6000..=0x7fff => {
				if let Some(mem) = &mut self.sram {
					mem[usize::from(addr-0x6000)] = data
				}
			},
			0x8000..=0xbfff => {
				self.cartridge.borrow_mut().program_mut(0)[usize::from(addr-0x8000)] = data
			},
			0xc000..=0xffff => {
				self.cartridge.borrow_mut().program_mut(1)[usize::from(addr-0xc000)] = data
			},
			_=> panic!("addr not mapped yet {:#04x}", addr),
		}
	}

	pub fn load_data(&mut self, addr: u16, data: &[u8]) {
		let mut addr = addr;
		for d in data {
			self.write_u8(addr, *d);
			addr = addr.wrapping_add(1);
		}
	}

	pub fn reset(&mut self) {
		if let Some(ram) = &mut self.internal_ram {
			ram.reset();
		}
	}

}
