use std::ops::{ Deref, DerefMut};

// the bus
#[derive(Default)]
pub struct CPUBus {
	// 0000-07FF 2K * 4
	internal_ram: Option<Ram>,
	// 0800-0fff
	// 1000-17ff
	// 1800-1fff

	// 2000-2007
	// ppu_regs: Option<&'a Memory>,

	// 4020-5fff 8K-20h
	rom: Option<Ram>,
	// 6000-7fff 8K
	sram: Option<Ram>,
	// 8000-bfff 16K
	prg1: Option<Ram>,
	// c000-ffff 16K
	prg2: Option<Ram>,
}


impl CPUBus {

	pub fn new() -> Self {
		// internal ram
		Self {
			internal_ram: Some(Ram::new(8192)),
			rom: Some(Ram::new(8159)),
			sram: Some(Ram::new(8192)),
			prg1: Some(Ram::new(16*1024)),
			prg2: Some(Ram::new(16*1024)),
		}
	}

	// load address
	pub fn load1(&self, addr: u16) -> u8 {

		match addr {
			0x0000..=0x1fff => {
				if let Some(mem) = &self.internal_ram {
					mem[usize::from(addr&0x7ff)]
				} else {
					0
				}
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
				if let Some(mem) = &self.prg1 {
					mem[usize::from(addr-0x8000)]
				} else {
					0
				}
			},
			0xc000..=0xffff => {
				if let Some(mem) = &self.prg2 {
					mem[usize::from(addr-0xc000)]
				} else {
					0
				}
			},
			_=> panic!("addr not mapped yet {:#04x}", addr),
		}
	}

	pub fn load2(&self, addr: u16) -> u16 {
		let l = self.load1(addr);
		let h = self.load1(addr.wrapping_add(1));
		(h as u16) << 8 | l as u16
	}

	pub fn save1(&mut self, addr: u16, data: u8) {
		match addr {
			0x0000..=0x1fff => {
				if let Some(mem) = &mut self.internal_ram {
					mem[usize::from(addr&0x7ff)] = data
				}
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
				if let Some(mem) = &mut self.prg1 {
					mem[usize::from(addr-0x8000)] = data
				}
			},
			0xc000..=0xffff => {
				if let Some(mem) = &mut self.prg2 {
					mem[usize::from(addr-0xc000)] = data
				}
			},
			_=> panic!("addr not mapped yet {:#04x}", addr),
		}
	}

	pub fn load_data(&mut self, addr: u16, data: &[u8]) {
		let mut addr = addr;
		for d in data {
			self.save1(addr, *d);
			addr = addr.wrapping_add(1);
		}
	}

	pub fn reset(&mut self) {
		if let Some(ram) = &mut self.internal_ram {
			ram.reset();
		}
	}

}


// memory
pub struct Ram {
	data: Vec<u8>,
}


impl Ram {
	fn new(size: usize) -> Self {
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
