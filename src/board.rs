use std::ops::{ Deref, DerefMut};


type Register = u8;


// the bus
pub struct Bus {
	// 0000-07FF
	internal_ram: Option<Ram>,
	// 0800-0fff
	// 1000-17ff
	// 1800-1fff
	// 2000-2007
	// ppu_regs: Option<&'a Memory>,
}


impl Bus {

	// load address
	pub fn load1(&self, addr: u16) -> u8 {
		if addr <= 0x1FFF {
			if let Some(mem) = &self.internal_ram {
				return mem[usize::from(addr & 0x7ff)]
			}
		}
		0
	}

	pub fn load2(&self, addr: u16) -> u16 {
		let l = self.load1(addr);
		let h = self.load1(addr.wrapping_add(1));
		(h as u16) << 8 | l as u16
	}

	pub fn save1(&mut self, addr: u16, data: u8) {
		if addr <= 0x07FF {
			if let Some(mem) = &mut self.internal_ram {
				mem[usize::from(addr & 0x7ff)] = data;
			}
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

}
