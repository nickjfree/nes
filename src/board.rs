use std::ops::{ Deref, DerefMut};
use std::cell::RefCell;
use std::rc::Rc;
use crate::cartridge::Cartridge;
use crate::ppu::PPU;


// memory
#[derive(Default, Debug)]
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

    pub fn read_u8(&mut self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) -> u32 {
        self.data[addr as usize] = val;
        0
    }

    pub fn reset(&mut self) {
        self.data.iter_mut().map(|x| *x = 0).count();
    }
}

pub type Signal = u8;


// the cpu bus
#[derive(Default)]
pub struct CPUBus {
    // 0000-07FF 2K * 4
    internal_ram: Option<Ram>,
    // 0800-0fff
    // 1000-17ff
    // 1800-1fff

    // 2000-2007
    ppu: Rc<RefCell<PPU>>,

    // 4020-5fff 8K-20h
    rom: Option<Ram>,
    // 6000-7fff 8K
    sram: Option<Ram>,
    // 8000-bfff 16K
    // c000-ffff 16K
    cartridge: Rc<RefCell<Cartridge>>,
}


// CPU bus
impl CPUBus {

    // new cpu bus
    pub fn new(ppu: Rc<RefCell<PPU>>, cartridge: Rc<RefCell<Cartridge>>) -> Self {
        // internal ram
        Self {
            internal_ram: Some(Ram::new(8192)),
            ppu: ppu,
            rom: Some(Ram::new(8159)),
            sram: Some(Ram::new(8192)),
            cartridge: cartridge,
        }
    }

    // load address
    pub fn read_u8(&self, addr: u16) -> u8 {
        // println!("read {:#02x}", addr);

        match addr {
            // internal_ram
            0x0000..=0x1fff => {
                if let Some(mem) = &self.internal_ram {
                    mem[usize::from(addr&0x7ff)]
                } else {
                    0
                }
            },
            // ppu registers
            0x2000..=0x3fff => {
                let addr = ((addr - 0x2000) & 0x07) + 0x2000;
                self.ppu.borrow_mut().read_u8(addr)
            },
            // oam dma
            0x4014 => {
                0
            },
            // cartridge rom
            0x4020..=0x5fff => {
                if let Some(mem) = &self.rom {
                    mem[usize::from(addr-0x4020)]
                } else {
                    0
                }
            },
            // cartridge sram
            0x6000..=0x7fff => {
                if let Some(mem) = &self.sram {
                    mem[usize::from(addr-0x6000)]
                } else {
                    0
                }
            },
            // cartridge program 0
            0x8000..=0xbfff => {
                self.cartridge.borrow().program(0)[usize::from(addr-0x8000)]
            },
            // cartridge program 1
            0xc000..=0xffff => {
                self.cartridge.borrow().program(1)[usize::from(addr-0xc000)]
            },
            _=> panic!("addr not mapped yet {:#04x}", addr),
        }
    }

    // read 2 byte as an address at addr
    pub fn read_u16(&self, addr: u16) -> u16 {
        let l = self.read_u8(addr);
        let h = self.read_u8(addr.wrapping_add(1));
        (h as u16) << 8 | l as u16
    }

    // write data to bus
    // 
    // delay: some write may stale cpu, delay is returned 
    pub fn write_u8(&mut self, addr: u16, data: u8) -> u32 {
        let mut delayed_cycles = 0;
        match addr {
            // internal_ram
            0x0000..=0x1fff => {
                if let Some(mem) = &mut self.internal_ram {
                    mem[usize::from(addr&0x7ff)] = data
                }
            },
            // ppu registers
            0x2000..=0x3fff => {
                let addr = (addr - 0x2000) & 0x07 + 0x2000;
                self.ppu.borrow_mut().write_u8(addr, data);
            },
            // oam dma
            0x4014 => {
                // oam dma will stale cpu for 513+ cycles
                delayed_cycles += self.ppu.borrow_mut().write_u8(addr, data);
            },
            // cartridge rom
            0x4020..=0x5fff => {
                if let Some(mem) = &mut self.rom {
                    mem[usize::from(addr-0x4020)] = data
                }
            },
            // cartridge sram
            0x6000..=0x7fff => {
                if let Some(mem) = &mut self.sram {
                    mem[usize::from(addr-0x6000)] = data
                }
            },
            // cartridge program 0
            0x8000..=0xbfff => {
                self.cartridge.borrow_mut().program_mut(0)[usize::from(addr-0x8000)] = data
            },
            // cartridge program 1
            0xc000..=0xffff => {
                self.cartridge.borrow_mut().program_mut(1)[usize::from(addr-0xc000)] = data
            },
            _=> panic!("addr not mapped yet {:#04x}", addr),
        }
        delayed_cycles
    }

    // load test data into bus
    pub fn load_data(&mut self, addr: u16, data: &[u8]) {
        let mut addr = addr;
        for d in data {
            self.write_u8(addr, *d);
            addr = addr.wrapping_add(1);
        }
    }

    // reset internal ram
    pub fn reset(&mut self) {
        if let Some(ram) = &mut self.internal_ram {
            ram.reset();
        }
    }

}
