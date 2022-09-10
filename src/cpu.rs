use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;
use crate::ppu::PPU;
use crate::board::{ Ram, Signal };
use crate::cartridge::Cartridge;
use crate::controller::Controller;

// flags
const STATUS_CARRAY  :u8 = 0x01;
const STATUS_ZERO    :u8 = 0x02;
const STATUS_INTERUPT:u8 = 0x04;
const STATUS_DEC     :u8 = 0x08;
const STATUS_B2      :u8 = 0x10;
const STATUS_B1      :u8 = 0x20;
const STATUS_OVERFLOW:u8 = 0x40;
const STATUS_NEG     :u8 = 0x80;

const STACK_BASE     :u16 = 0x0100;

#[derive(Default, Debug)]
struct Registers {
    // accumulator
    acc: u8,
    // x and y registers
    x: u8,
    y: u8,
    // pc
    pc: u16,
    // stack pointer
    sp: u8,
    // status register
    status: u8,
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A: {:#04x}, X: {:#04x}, Y: {:#04x}, PC: {:#06x}, S: {:#04x}, P: {:#010b}",
            self.acc, self.x, self.y, self.pc, self.sp, self.status)?;
        Ok(())
    }
}


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
    // TODO: apu registers, 4000-401f
    // 4016-4017 controller
    controller: Rc<RefCell<Controller>>,
    apu: Option<Ram>,
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
    pub fn new(ppu: Rc<RefCell<PPU>>, cartridge: Rc<RefCell<Cartridge>>, controller: Rc<RefCell<Controller>>) -> Self {
        // internal ram
        Self {
            internal_ram: Some(Ram::new(8192)),
            ppu: ppu,
            apu: Some(Ram::new(32)),
            rom: Some(Ram::new(8159)),
            sram: Some(Ram::new(8192)),
            cartridge: cartridge,
            controller: controller,
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
            // apu registers
            0x4000..=0x4013 => {
                if let Some(mem) = &self.apu {
                    mem[usize::from(addr-0x4000)]
                } else {
                    0
                }
            },
            // oam dma
            0x4014 => {
                0
            },
            0x4016..=0x4017 => {
                self.controller.borrow_mut().read_u8(addr)
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
             _ => 0,
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
                let addr = ((addr - 0x2000) & 0x07) + 0x2000;
                self.ppu.borrow_mut().write_u8(addr, data);
            },
            // oam dma
            0x4014 => {
                // oam dma will stale cpu for 513+ cycles
                delayed_cycles += self.ppu.borrow_mut().write_u8(addr, data);
            },
            // apu registers
            0x4000..=0x4013 => {
                if let Some(mem) = &mut self.apu {
                    mem[usize::from(addr-0x4000)] = data
                }
            },
            0x4016..=0x4017 => {
                self.controller.borrow_mut().write_u8(addr, data);
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
            _ => (),
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


// cpu
#[derive(Default)]
pub struct CPU {
    // registers
    regs: Registers,  

    // current opcode
    opcode: u8,
    // cycles_delay
    cycles_delay: u32,
    // tmp operand address
    op_addr: u16,
    // immediate
    is_immediate: bool,
    // bus
    bus: CPUBus,
    // nmi signal
    nmi: Signal,
}


impl CPU {
    // new cpu
    pub fn new(ppu: Rc<RefCell<PPU>>, cartridge: Rc<RefCell<Cartridge>>, controller: Rc<RefCell<Controller>>, nmi: Signal) -> Self {
        Self {
            bus: CPUBus::new(ppu, cartridge, controller),
            nmi: nmi,
            ..CPU::default()
        }
    }

    // common ops

    fn mem_read_u8(&mut self, addr: u16) -> u8 {
        self.bus.read_u8(addr)
    }

    fn mem_read_u16(&mut self, addr: u16) -> u16 {
        let l = self.mem_read_u8(addr);
        let h = self.mem_read_u8(addr.wrapping_add(1));
        (h as u16) << 8 | l as u16
    }

    fn mem_write_u8(&mut self, addr: u16, val: u8) {
        self.cycles_delay += self.bus.write_u8(addr, val)
    }

    fn push_u8(&mut self, val: u8) {
        self.mem_write_u8(self.regs.sp as u16 + STACK_BASE, val);
        self.regs.sp = self.regs.sp.wrapping_sub(1);
    }

    fn push_u16(&mut self, val: u16) {
        let l: u8 = val as u8 & 0xff;
        let h: u8 = (val >> 8) as u8 & 0xff;
        self.push_u8(h);
        self.push_u8(l);
    }

    fn pop_u8(&mut self) -> u8 {
        self.regs.sp = self.regs.sp.wrapping_add(1);
        self.mem_read_u8(self.regs.sp as u16 + STACK_BASE)
    }

    fn pop_u16(&mut self) -> u16 {
        let l = self.pop_u8();
        let h = self.pop_u8();
        (h as u16) << 8 | l as u16
    }

    fn fetch_u8(&mut self) -> u8 {
        let d = self.mem_read_u8(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        d
    }

    fn fetch_u16(&mut self) -> u16 {
        let l = self.fetch_u8();
        let h = self.fetch_u8();
        (h as u16) << 8 | l as u16 
    }

    // addressing mode

    fn handle_cross_page(&mut self, a: u16, b: u16) {
        if (a >> 8) & 0xff != (b >> 8) & 0xff {
            self.cycles_delay += 1
        }
    }

    fn addressing_none(&mut self) {
        self.op_addr = 0;
        self.is_immediate = false;
    }

    fn implied(&mut self) {}

    fn accumulator(&mut self) {}

    fn immediate(&mut self) {
        self.is_immediate = true;
    }

    fn zero_page(&mut self) {
        self.op_addr = u16::from(self.fetch_u8() & 0xff);
    }

    fn zero_page_x(&mut self) {
        // load operand
        let d = self.fetch_u8();
        self.op_addr = (d.wrapping_add(self.regs.x) & 0xFF).into();
    }

    fn zero_page_y(&mut self) {
        // load operand
        let d = self.fetch_u8();
        self.op_addr = (d.wrapping_add(self.regs.y) & 0xFF).into();
    }

    fn absolute(&mut self) {
        self.op_addr = self.fetch_u16();
    }

    fn absolute_x(&mut self) {
        let d = self.fetch_u16();
        self.op_addr = d.wrapping_add(self.regs.x as u16);
        self.handle_cross_page(self.op_addr, d);
    }

    fn absolute_y(&mut self) {
        let d = self.fetch_u16();
        self.op_addr = d.wrapping_add(self.regs.y as u16);
        self.handle_cross_page(self.op_addr, d);
    }

    fn indirect_x(&mut self) {
        let d = self.fetch_u8();
        let l = self.mem_read_u8(d.wrapping_add(self.regs.x).into()) as u16;
        let h = self.mem_read_u8(d.wrapping_add(self.regs.x).wrapping_add(1).into()) as u16;
        self.op_addr = (h << 8) | l;
    }

    fn indirect_y(&mut self) {
        let d = self.fetch_u8();
        let l = self.mem_read_u8(d.into()) as u16;
        let h = self.mem_read_u8(d.wrapping_add(1).into()) as u16;
        self.op_addr = ((h << 8) | l).wrapping_add(u16::from(self.regs.y));
        self.handle_cross_page(self.op_addr, (h << 8) | l);
    }

    fn relative(&mut self) {
        let d = self.fetch_u8();
        let rel: u16 = (d as i8) as u16;
        let base = self.regs.pc;
        self.op_addr = base.wrapping_add(rel);
        self.handle_cross_page(self.op_addr, base);
    }

    fn indirect(&mut self) {
        self.op_addr = self.fetch_u16();
        self.op_addr = self.mem_read_u16(self.op_addr);
    }

    fn op_val(&mut self) -> u8 {
        match self.is_immediate {
            true => self.fetch_u8(),
            false => self.mem_read_u8(self.op_addr),
        }
    }

    // flages
    fn flag_nz(&mut self, val: u8) {
        match val {
            0 => self.regs.status |= STATUS_ZERO,
            _ => self.regs.status &= !STATUS_ZERO,
        }
        match val & 0x80 {
            0 => self.regs.status &= !STATUS_NEG,
            _ => self.regs.status |= STATUS_NEG,
        }
    }

    // transfer instructions

    fn lda(&mut self) {
        self.regs.acc = self.op_val();
        self.flag_nz(self.regs.acc);
    }

    fn ldx(&mut self) {
        self.regs.x = self.op_val();
        self.flag_nz(self.regs.x);
    }

    fn ldy(&mut self) {
        self.regs.y = self.op_val();
        self.flag_nz(self.regs.y);
    }

    fn sta(&mut self) {
        self.mem_write_u8(self.op_addr, self.regs.acc);
    }

    fn stx(&mut self) {
        self.mem_write_u8(self.op_addr, self.regs.x);
    }

    fn sty(&mut self) {
        self.mem_write_u8(self.op_addr, self.regs.y);
    }

    fn tax(&mut self) {
        self.regs.x = self.regs.acc;
        self.flag_nz(self.regs.x);
    }

    fn tay(&mut self) {
        self.regs.y = self.regs.acc;
        self.flag_nz(self.regs.y);
    }

    fn tsx(&mut self) {
        self.regs.x = self.regs.sp;
        self.flag_nz(self.regs.x);
    }

    fn txa(&mut self) {
        self.regs.acc = self.regs.x;
        self.flag_nz(self.regs.acc);
    }

    fn txs(&mut self) {
        self.regs.sp = self.regs.x;
    }

    fn tya(&mut self) {
        self.regs.acc = self.regs.y;
        self.flag_nz(self.regs.acc);
    }


    // stack instructions

    fn pha(&mut self) {
        self.push_u8(self.regs.acc);
    }

    fn php(&mut self) {
        let status = self.regs.status | STATUS_B1 | STATUS_B2;
        self.push_u8(status);
    }

    fn pla(&mut self) {
        self.regs.acc = self.pop_u8();
        self.flag_nz(self.regs.acc);
    }

    fn plp(&mut self) {
        self.regs.status = self.pop_u8() & !STATUS_B1 & ! STATUS_B2;
    }


    // descrements & increments
    fn dec(&mut self) {
        let mut oprand = self.op_val();
        oprand = oprand.wrapping_sub(1);
        self.mem_write_u8(self.op_addr, oprand);
        self.flag_nz(oprand);
    }

    fn dex(&mut self) {
        self.regs.x = self.regs.x.wrapping_sub(1);
        self.flag_nz(self.regs.x);
    }

    fn dey(&mut self) {
        self.regs.y = self.regs.y.wrapping_sub(1);
        self.flag_nz(self.regs.y);
    }

    fn inc(&mut self) {
        let mut oprand = self.op_val();
        oprand = oprand.wrapping_add(1);
        self.mem_write_u8(self.op_addr, oprand);
        self.flag_nz(oprand);
    }

    fn inx(&mut self) {
        self.regs.x = self.regs.x.wrapping_add(1);
        self.flag_nz(self.regs.x);
    }

    fn iny(&mut self) {
        self.regs.y = self.regs.y.wrapping_add(1);
        self.flag_nz(self.regs.y);
    }

    // arithmetic instructions

    fn adc(&mut self) {
        let oprand = self.op_val();
        let result: u16 = self.regs.acc as u16 + oprand as u16 + (self.regs.status & STATUS_CARRAY) as u16;
        match result & 0xFF00 {
            0 => self.regs.status &= !STATUS_CARRAY,
            _ => self.regs.status |= STATUS_CARRAY,
        }
        let result = (result & 0xff) as u8;
        match (self.regs.acc ^ result) & (oprand ^ result) & 0x80 {
            0 => self.regs.status &= !STATUS_OVERFLOW,
            _ => self.regs.status |= STATUS_OVERFLOW,
        }
        self.regs.acc = result;
        self.flag_nz(self.regs.acc);
    }

    fn sbc(&mut self) {
        let oprand = !self.op_val();
        let result: u16 = self.regs.acc as u16 + oprand as u16 + (self.regs.status & STATUS_CARRAY) as u16;
        match result & 0xff00 {
            0 => self.regs.status &= !STATUS_CARRAY,
            _ => self.regs.status |= STATUS_CARRAY,
        }
        let result = (result & 0xff) as u8;
        match (self.regs.acc ^ result) & (oprand ^ result) & 0x80 {
            0 => self.regs.status &= !STATUS_OVERFLOW,
            _ => self.regs.status |= STATUS_OVERFLOW,
        }
        self.regs.acc = result;
        self.flag_nz(self.regs.acc);
    }

    // logic operations

    fn and(&mut self) {
        self.regs.acc &= self.op_val();
        self.flag_nz(self.regs.acc);
    }

    fn eor(&mut self) {
        self.regs.acc ^= self.op_val();
        self.flag_nz(self.regs.acc);
    }

    fn ora(&mut self) {
        self.regs.acc |= self.op_val();
        self.flag_nz(self.regs.acc);
    }

    // shift & rotate instrcutions
    fn asl(&mut self) {
        let mut oprand = self.op_val();
        match oprand & 0x80  {
            0 => self.regs.status &= !STATUS_CARRAY,
            _ => self.regs.status |= STATUS_CARRAY,
        }
        oprand <<= 1;
        self.mem_write_u8(self.op_addr, oprand);
        self.flag_nz(oprand);
    }

    fn asla(&mut self) {
        match self.regs.acc & 0x80  {
            0 => self.regs.status &= !STATUS_CARRAY,
            _ => self.regs.status |= STATUS_CARRAY,
        }
        self.regs.acc = self.regs.acc << 1;
        self.flag_nz(self.regs.acc);
    }

    fn lsr(&mut self) {
        let mut oprand = self.op_val();
        match oprand & 0x01 {
            0 => self.regs.status &= !STATUS_CARRAY,
            _ => self.regs.status |= STATUS_CARRAY,
        }
        oprand >>= 1;
        self.mem_write_u8(self.op_addr, oprand);
        self.flag_nz(oprand);
    }

    fn lsra(&mut self) {
        match self.regs.acc & 0x01 {
            0 => self.regs.status &= !STATUS_CARRAY,
            _ => self.regs.status |= STATUS_CARRAY,
        }
        self.regs.acc = self.regs.acc >> 1;
        self.flag_nz(self.regs.acc);
    }

    fn rol(&mut self) {
        let old = self.regs.status & STATUS_CARRAY;
        let mut oprand = self.op_val();
        match oprand & 0x80  {
            0 => self.regs.status &= !STATUS_CARRAY,
            _ => self.regs.status |= STATUS_CARRAY,
        }
        oprand = (oprand << 1).wrapping_add(old);
        self.mem_write_u8(self.op_addr, oprand);
        self.flag_nz(oprand)
    }


    fn rola(&mut self) {
        let old = self.regs.status & STATUS_CARRAY;
        match self.regs.acc & 0x80  {
            0 => self.regs.status &= !STATUS_CARRAY,
            _ => self.regs.status |= STATUS_CARRAY,
        }
        self.regs.acc = (self.regs.acc << 1).wrapping_add(old);
        self.flag_nz(self.regs.acc);
    }

    fn ror(&mut self) {
        let old = (self.regs.status & STATUS_CARRAY) << 7;
        let mut oprand = self.op_val();
        match oprand & 0x01  {
            0 => self.regs.status &= !STATUS_CARRAY,
            _ => self.regs.status |= STATUS_CARRAY,
        }
        oprand = (oprand >> 1).wrapping_add(old);
        self.mem_write_u8(self.op_addr, oprand);
        self.flag_nz(oprand)
    }

    fn rora(&mut self) {
        let old = (self.regs.status & STATUS_CARRAY) << 7;
        match self.regs.acc & 0x81  {
            0 => self.regs.status &= !STATUS_CARRAY,
            _ => self.regs.status |= STATUS_CARRAY,
        }
        self.regs.acc = (self.regs.acc >> 1).wrapping_add(old);
        self.flag_nz(self.regs.acc);
    }

    // flag instructions

    fn clc(&mut self) {
        self.regs.status &= !STATUS_CARRAY;
    }

    fn cld(&mut self) {
        self.regs.status &= !STATUS_DEC;
    }

    fn cli(&mut self) {
        self.regs.status &= !STATUS_INTERUPT;
    }

    fn clv(&mut self) {
        self.regs.status &= !STATUS_OVERFLOW;
    }

    fn sec(&mut self) {
        self.regs.status |= STATUS_CARRAY;
    }

    fn sed(&mut self) {
        self.regs.status |= STATUS_DEC;
    }

    fn sei(&mut self) {
        self.regs.status |= STATUS_INTERUPT;
    }

    // comparisions
    fn cmp(&mut self) {
        let oprand = !self.op_val();
        let result = self.regs.acc as u16 + oprand as u16 + 1;
        match result & 0xff00 {
            0 => self.regs.status &= !STATUS_CARRAY,
            _ => self.regs.status |= STATUS_CARRAY,
        }
        self.flag_nz(result as u8);
    }

    fn cpx(&mut self) {
        let oprand = !self.op_val();
        let result = self.regs.x as u16 + oprand as u16 + 1;
        match result & 0xff00 {
            0 => self.regs.status &= !STATUS_CARRAY,
            _ => self.regs.status |= STATUS_CARRAY,
        }
        self.flag_nz(result as u8);
    }

    fn cpy(&mut self) {
        let oprand = !self.op_val();
        let result = self.regs.y as u16 + oprand as u16 + 1;
        match result & 0xff00 {
            0 => self.regs.status &= !STATUS_CARRAY,
            _ => self.regs.status |= STATUS_CARRAY,
        }
        self.flag_nz(result as u8);
    }

    // branch

    fn bcc(&mut self) {
        match self.regs.status & STATUS_CARRAY {
            0 => {
                self.regs.pc = self.op_addr;
                self.cycles_delay += 1;
            },
            _ => (),
        }
    }

    fn bcs(&mut self) {
        match self.regs.status & STATUS_CARRAY {
            0 => (),
            _ => {
                self.regs.pc = self.op_addr;
                self.cycles_delay += 1;
            }
        }
    }

    fn beq(&mut self) {
        match self.regs.status & STATUS_ZERO {
            0 => (),
            _ => {
                self.regs.pc = self.op_addr;
                self.cycles_delay += 1;
            }
        }
    }

    fn bmi(&mut self) {
        match self.regs.status & STATUS_NEG {
            0 => (),
            _ => {
                self.regs.pc = self.op_addr;
                self.cycles_delay += 1;
            }
        }
    }

    fn bne(&mut self) {
        match self.regs.status & STATUS_ZERO {
            0 => {
                self.regs.pc = self.op_addr;
                self.cycles_delay += 1;
            },
            _ => (),
        }
    }

    fn bpl(&mut self) {
        match self.regs.status & STATUS_NEG {
            0 => {
                self.regs.pc = self.op_addr;
                self.cycles_delay += 1;
            },
            _ => (),
        }
    }

    fn bvc(&mut self) {
        match self.regs.status & STATUS_OVERFLOW {
            0 => {
                self.regs.pc = self.op_addr;
                self.cycles_delay += 1;
            },
            _ => (),
        }
    }

    fn bvs(&mut self) {
        match self.regs.status & STATUS_OVERFLOW {
            0 => (),
            _ => {
                self.regs.pc = self.op_addr;
                self.cycles_delay += 1;
            },
        }
    }

    // jumps & subroutines

    fn jmp(&mut self) {
        self.regs.pc = self.op_addr;
    }

    fn jsr(&mut self) {
        self.push_u16(self.regs.pc.wrapping_sub(1));
        self.regs.pc = self.op_addr;
    }

    fn rts(&mut self) {
        self.regs.pc = self.pop_u16().wrapping_add(1);
    }

    // interupts

    fn nmi(&mut self) {
        // set I flag
        let status = self.regs.status | STATUS_INTERUPT | STATUS_B1;
        self.push_u16(self.regs.pc);
        self.push_u8(status);
        //println!("before nmi {}", self.regs);
        self.regs.pc = self.bus.read_u16(0xfffa);
        //println!("after nmi {}", self.regs);
        self.cycles_delay += 7;
    }

    fn irq(&mut self) {
        // set I flag
        let status = self.regs.status | STATUS_INTERUPT | STATUS_B1;
        self.push_u16(self.regs.pc);
        self.push_u8(status);
        self.regs.pc = self.bus.read_u16(0xfffc);
        self.regs.status |= STATUS_INTERUPT;
        self.cycles_delay += 7;
    }

    fn brk(&mut self) {
        // read next instruction byte (and throw it away)
        self.fetch_u8();
        // set I flag
        let status = self.regs.status | STATUS_INTERUPT | STATUS_B1 | STATUS_B2;
        self.push_u16(self.regs.pc);
        self.push_u8(status);
        // println!("before brk {}", self.regs);
        self.regs.pc = self.bus.read_u16(0xfffe);
    }

    fn rti(&mut self) {
        self.regs.status = self.pop_u8() & !STATUS_B1 & !STATUS_B2;
        // println!("before rti {}", self.regs);
        self.regs.pc = self.pop_u16();
        // println!("after rti {}", self.regs);
    }

    // others

    fn bit(&mut self) {
        let oprand = self.op_val();
        let result = self.regs.acc & oprand;
        self.flag_nz(result);
        match oprand & STATUS_NEG {
            0 => self.regs.status &= !STATUS_NEG,
            _ => self.regs.status |= STATUS_NEG,
        }
        match oprand & STATUS_OVERFLOW {
            0 => self.regs.status &= !STATUS_OVERFLOW,
            _ => self.regs.status |= STATUS_OVERFLOW,
        }
    }

    fn nop(&mut self) {

    }

    fn handle_interupt(&mut self) -> bool {
        // check nmi
        let has_nmi = {
            let mut nmi = self.nmi.borrow_mut();
            match *nmi {
                0 => false,
                _ => { *nmi = 0; true},
            }
        };
        if has_nmi {
            self.nmi();
        }
        has_nmi
    }

    // status
    pub fn power_up(&mut self) {
        self.regs.acc = 0;
        self.regs.x = 0;
        self.regs.y = 0;
        self.regs.status |= STATUS_INTERUPT | STATUS_B1 | STATUS_B2;
        self.regs.sp = 0xfd;
        self.regs.pc = self.bus.read_u16(0xfffc);
    }

    pub fn reset(&mut self) {
        // reset interupt. but write is diabled
        self.regs.sp = self.regs.sp.wrapping_sub(3);
        self.regs.status |= STATUS_INTERUPT;
        self.regs.pc = self.bus.read_u16(0xfffc);
    }

    pub fn load_data(&mut self, addr: u16, data: &[u8]) {    
        self.bus.load_data(addr, data)
    }

    // step simulation
    pub fn tick(&mut self) -> u32 {

        if self.cycles_delay <= 0 {
            
            if self.handle_interupt() {
                return 0;
            }
            // load next opcode
            self.opcode = self.fetch_u8();
            // clear addressing mode
            self.addressing_none();
            // debug
            println!("opcode: {:#02x} regs: {} ", self.opcode, self.regs);
            match self.opcode { 
                0x00 => { self.implied();       self.brk();     self.cycles_delay+=7; },
                0x01 => { self.indirect_x();    self.ora();     self.cycles_delay+=6; },
                0x04 => { self.zero_page();     self.nop();     self.cycles_delay+=3; },
                0x05 => { self.zero_page();     self.ora();     self.cycles_delay+=3; },
                0x06 => { self.zero_page();     self.asl();     self.cycles_delay+=5; },
                0x08 => { self.implied();       self.php();     self.cycles_delay+=3; },
                0x09 => { self.immediate();     self.ora();     self.cycles_delay+=2; },
                0x0A => { self.accumulator();   self.asla();    self.cycles_delay+=2; },
                0x0C => { self.absolute();      self.nop();     self.cycles_delay+=4; },
                0x0D => { self.absolute();      self.ora();     self.cycles_delay+=4; },
                0x0E => { self.absolute();      self.asl();     self.cycles_delay+=6; },
                0x10 => { self.relative();      self.bpl();     self.cycles_delay+=2; },
                0x11 => { self.indirect_y();    self.ora();     self.cycles_delay+=5; },
                0x14 => { self.zero_page_x();   self.nop();     self.cycles_delay+=4; },
                0x15 => { self.zero_page_x();   self.ora();     self.cycles_delay+=4; },
                0x16 => { self.zero_page_x();   self.asl();     self.cycles_delay+=6; },
                0x18 => { self.implied();       self.clc();     self.cycles_delay+=2; },
                0x19 => { self.absolute_y();    self.ora();     self.cycles_delay+=4; },
                0x1A => { self.implied();       self.nop();     self.cycles_delay+=2; },
                0x1C => { self.absolute_x();    self.nop();     self.cycles_delay+=4; },
                0x1D => { self.absolute_x();    self.ora();     self.cycles_delay+=4; },
                0x1E => { self.absolute_x();    self.asl();     self.cycles_delay+=7; },
                0x20 => { self.absolute();      self.jsr();     self.cycles_delay+=6; },
                0x21 => { self.indirect_x();    self.and();     self.cycles_delay+=6; },
                0x24 => { self.zero_page();     self.bit();     self.cycles_delay+=3; },
                0x25 => { self.zero_page();     self.and();     self.cycles_delay+=3; },
                0x26 => { self.zero_page();     self.rol();     self.cycles_delay+=5; },
                0x28 => { self.implied();       self.plp();     self.cycles_delay+=3; },
                0x29 => { self.immediate();     self.and();     self.cycles_delay+=2; },
                0x2A => { self.accumulator();   self.rola();    self.cycles_delay+=2; },
                0x2C => { self.absolute();      self.bit();     self.cycles_delay+=4; },
                0x2D => { self.absolute();      self.and();     self.cycles_delay+=4; },
                0x2E => { self.absolute();      self.rol();     self.cycles_delay+=6; },
                0x30 => { self.relative();      self.bmi();     self.cycles_delay+=2; },
                0x31 => { self.indirect_y();    self.and();     self.cycles_delay+=5; },
                0x34 => { self.zero_page_x();   self.nop();     self.cycles_delay+=4; },
                0x35 => { self.zero_page_x();   self.and();     self.cycles_delay+=4; },
                0x36 => { self.zero_page_x();   self.rol();     self.cycles_delay+=6; },
                0x38 => { self.implied();       self.sec();     self.cycles_delay+=2; },
                0x39 => { self.absolute_y();    self.and();     self.cycles_delay+=4; },
                0x3A => { self.implied();       self.nop();     self.cycles_delay+=2; },
                0x3C => { self.absolute_x();    self.nop();     self.cycles_delay+=4; },
                0x3D => { self.absolute_x();    self.and();     self.cycles_delay+=4; },
                0x3E => { self.absolute_x();    self.rol();     self.cycles_delay+=7; },
                0x40 => { self.implied();       self.rti();     self.cycles_delay+=6; },
                0x41 => { self.indirect_x();    self.eor();     self.cycles_delay+=6; },
                0x44 => { self.zero_page();     self.nop();     self.cycles_delay+=3; },
                0x45 => { self.zero_page();     self.eor();     self.cycles_delay+=3; },
                0x46 => { self.zero_page();     self.lsr();     self.cycles_delay+=5; },
                0x48 => { self.implied();       self.pha();     self.cycles_delay+=3; },
                0x49 => { self.immediate();     self.eor();     self.cycles_delay+=2; },
                0x4A => { self.accumulator();   self.lsra();    self.cycles_delay+=2; },
                0x4C => { self.absolute();      self.jmp();     self.cycles_delay+=3; },
                0x4D => { self.absolute();      self.eor();     self.cycles_delay+=4; },
                0x4E => { self.absolute();      self.lsr();     self.cycles_delay+=6; },
                0x50 => { self.relative();      self.bvc();     self.cycles_delay+=2; },
                0x51 => { self.indirect_y();    self.eor();     self.cycles_delay+=5; },
                0x54 => { self.zero_page_x();   self.nop();     self.cycles_delay+=4; },
                0x55 => { self.zero_page_x();   self.eor();     self.cycles_delay+=4; },
                0x56 => { self.zero_page_x();   self.lsr();     self.cycles_delay+=6; },
                0x58 => { self.implied();       self.cli();     self.cycles_delay+=6; },
                0x59 => { self.absolute_y();    self.eor();     self.cycles_delay+=4; },
                0x5A => { self.implied();       self.nop();     self.cycles_delay+=2; },
                0x5C => { self.absolute_x();    self.nop();     self.cycles_delay+=4; },
                0x5D => { self.absolute_x();    self.eor();     self.cycles_delay+=4; },
                0x5E => { self.absolute_x();    self.lsr();     self.cycles_delay+=7; },
                0x60 => { self.implied();       self.rts();     self.cycles_delay+=6; },
                0x61 => { self.indirect_x();    self.adc();     self.cycles_delay+=6; },
                0x64 => { self.zero_page();     self.nop();     self.cycles_delay+=3; },
                0x65 => { self.zero_page();     self.adc();     self.cycles_delay+=3; },
                0x66 => { self.zero_page();     self.ror();     self.cycles_delay+=5; },
                0x68 => { self.implied();       self.pla();     self.cycles_delay+=4; },
                0x69 => { self.immediate();     self.adc();     self.cycles_delay+=2; },
                0x6A => { self.accumulator();   self.rora();    self.cycles_delay+=2; },
                0x6C => { self.indirect();      self.jmp();     self.cycles_delay+=5; },
                0x6D => { self.absolute();      self.adc();     self.cycles_delay+=4; },
                0x6E => { self.absolute();      self.ror();     self.cycles_delay+=6; },
                0x70 => { self.relative();      self.bvs();     self.cycles_delay+=2; },
                0x71 => { self.indirect_y();    self.adc();     self.cycles_delay+=5; },
                0x74 => { self.zero_page();     self.nop();     self.cycles_delay+=3; },
                0x75 => { self.zero_page_x();   self.adc();     self.cycles_delay+=4; },
                0x76 => { self.zero_page_x();   self.ror();     self.cycles_delay+=6; },
                0x78 => { self.implied();       self.sei();     self.cycles_delay+=2; },
                0x79 => { self.absolute_y();    self.adc();     self.cycles_delay+=4; },
                0x7A => { self.implied();       self.nop();     self.cycles_delay+=2; },
                0x7C => { self.absolute_x();    self.nop();     self.cycles_delay+=4; },
                0x7D => { self.absolute_x();    self.adc();     self.cycles_delay+=4; },
                0x7E => { self.absolute_x();    self.ror();     self.cycles_delay+=7; },
                0x80 => { self.immediate();     self.nop();     self.cycles_delay+=2; },
                0x81 => { self.indirect_x();    self.sta();     self.cycles_delay+=6; },
                0x84 => { self.zero_page();     self.sty();     self.cycles_delay+=3; },
                0x85 => { self.zero_page();     self.sta();     self.cycles_delay+=3; },
                0x86 => { self.zero_page();     self.stx();     self.cycles_delay+=3; },
                0x88 => { self.implied();       self.dey();     self.cycles_delay+=2; },
                0x8A => { self.implied();       self.txa();     self.cycles_delay+=2; },
                0x8C => { self.absolute();      self.sty();     self.cycles_delay+=4; },
                0x8D => { self.absolute();      self.sta();     self.cycles_delay+=4; },
                0x8E => { self.absolute();      self.stx();     self.cycles_delay+=4; },
                0x90 => { self.relative();      self.bcc();     self.cycles_delay+=2; },
                0x91 => { self.indirect_y();    self.sta();     self.cycles_delay+=6; },
                0x94 => { self.zero_page_x();   self.sty();     self.cycles_delay+=4; },
                0x95 => { self.zero_page_x();   self.sta();     self.cycles_delay+=4; },
                0x96 => { self.zero_page_y();   self.stx();     self.cycles_delay+=4; },
                0x98 => { self.implied();       self.tya();     self.cycles_delay+=2; },
                0x99 => { self.absolute_y();    self.sta();     self.cycles_delay+=5; },
                0x9A => { self.implied();       self.txs();     self.cycles_delay+=2; },
                0x9D => { self.absolute_x();    self.sta();     self.cycles_delay+=5; },
                0xA0 => { self.immediate();     self.ldy();     self.cycles_delay+=2; },
                0xA1 => { self.indirect_x();    self.lda();     self.cycles_delay+=6; },
                0xA2 => { self.immediate();     self.ldx();     self.cycles_delay+=2; },
                0xA4 => { self.zero_page();     self.ldy();     self.cycles_delay+=3; },
                0xA5 => { self.zero_page();     self.lda();     self.cycles_delay+=3; },
                0xA6 => { self.zero_page();     self.ldx();     self.cycles_delay+=3; },
                0xA8 => { self.implied();       self.tay();     self.cycles_delay+=3; },
                0xA9 => { self.immediate();     self.lda();     self.cycles_delay+=2; },
                0xAA => { self.implied();       self.tax();     self.cycles_delay+=2; },
                0xAC => { self.absolute();      self.ldy();     self.cycles_delay+=4; },
                0xAD => { self.absolute();      self.lda();     self.cycles_delay+=4; },
                0xAE => { self.absolute();      self.ldx();     self.cycles_delay+=4; },
                0xB0 => { self.relative();      self.bcs();     self.cycles_delay+=2; },
                0xB1 => { self.indirect_y();    self.lda();     self.cycles_delay+=5; },
                0xB4 => { self.zero_page_x();   self.ldy();     self.cycles_delay+=4; },
                0xB5 => { self.zero_page_x();   self.lda();     self.cycles_delay+=4; },
                0xB6 => { self.zero_page_y();   self.ldx();     self.cycles_delay+=4; },
                0xB8 => { self.implied();       self.clv();     self.cycles_delay+=2; },
                0xB9 => { self.absolute_y();    self.lda();     self.cycles_delay+=4; },
                0xBA => { self.implied();       self.tsx();     self.cycles_delay+=2; },
                0xBC => { self.absolute_x();    self.ldy();     self.cycles_delay+=4; },
                0xBD => { self.absolute_x();    self.lda();     self.cycles_delay+=4; },
                0xBE => { self.absolute_y();    self.ldx();     self.cycles_delay+=4; },
                0xC0 => { self.immediate();     self.cpy();     self.cycles_delay+=2; },
                0xC1 => { self.indirect_x();    self.cmp();     self.cycles_delay+=6; },
                0xC4 => { self.zero_page();     self.cpy();     self.cycles_delay+=3; },
                0xC5 => { self.zero_page();     self.cmp();     self.cycles_delay+=3; },
                0xC6 => { self.zero_page();     self.dec();     self.cycles_delay+=5; },
                0xC8 => { self.implied();       self.iny();     self.cycles_delay+=2; },
                0xC9 => { self.immediate();     self.cmp();     self.cycles_delay+=2; },
                0xCA => { self.implied();       self.dex();     self.cycles_delay+=2; },
                0xCC => { self.absolute();      self.cpy();     self.cycles_delay+=4; },
                0xCD => { self.absolute();      self.cmp();     self.cycles_delay+=4; },
                0xCE => { self.absolute();      self.dec();     self.cycles_delay+=6; },
                0xD0 => { self.relative();      self.bne();     self.cycles_delay+=2; },
                0xD1 => { self.indirect_y();    self.cmp();     self.cycles_delay+=5; },
                0xD4 => { self.zero_page_x();   self.nop();     self.cycles_delay+=4; },
                0xD5 => { self.zero_page_x();   self.cmp();     self.cycles_delay+=5; },
                0xD6 => { self.zero_page_x();   self.dec();     self.cycles_delay+=6; },
                0xD8 => { self.implied();       self.cld();     self.cycles_delay+=2; },
                0xD9 => { self.absolute_y();    self.cmp();     self.cycles_delay+=4; },
                0xDA => { self.implied();       self.nop();     self.cycles_delay+=2; },
                0xDC => { self.absolute_x();    self.nop();     self.cycles_delay+=4; },
                0xDD => { self.absolute_x();    self.cmp();     self.cycles_delay+=4; },
                0xDE => { self.absolute_x();    self.dec();     self.cycles_delay+=7; },
                0xE0 => { self.immediate();     self.cpx();     self.cycles_delay+=2; },
                0xE1 => { self.indirect_x();    self.sbc();     self.cycles_delay+=6; },
                0xE4 => { self.zero_page();     self.cpx();     self.cycles_delay+=3; },
                0xE5 => { self.zero_page();     self.sbc();     self.cycles_delay+=3; },
                0xE6 => { self.zero_page();     self.inc();     self.cycles_delay+=5; },
                0xE8 => { self.implied();       self.inx();     self.cycles_delay+=2; },
                0xE9 => { self.immediate();     self.sbc();     self.cycles_delay+=2; },
                0xEA => { self.implied();       self.nop();     self.cycles_delay+=2; },
                0xEC => { self.absolute();      self.cpx();     self.cycles_delay+=4; },
                0xED => { self.absolute();      self.sbc();     self.cycles_delay+=4; },
                0xEE => { self.absolute();      self.inc();     self.cycles_delay+=6; },
                0xF0 => { self.relative();      self.beq();     self.cycles_delay+=2; },
                0xF1 => { self.indirect_y();    self.sbc();     self.cycles_delay+=5; },
                0xF4 => { self.zero_page_x();   self.nop();     self.cycles_delay+=4; },
                0xF5 => { self.zero_page_x();   self.sbc();     self.cycles_delay+=4; },
                0xF6 => { self.zero_page_x();   self.inc();     self.cycles_delay+=6; },
                0xF8 => { self.implied();       self.sed();     self.cycles_delay+=2; },
                0xF9 => { self.absolute_y();    self.sbc();     self.cycles_delay+=4; },
                0xFA => { self.implied();       self.nop();     self.cycles_delay+=2; },
                0xFC => { self.absolute_x();    self.nop();     self.cycles_delay+=4; },
                0xFD => { self.absolute_x();    self.sbc();     self.cycles_delay+=4; },
                0xFE => { self.absolute_x();    self.inc();     self.cycles_delay+=7; },
                _=> panic!("unknow opcode {:#02x} {}", self.opcode, self.regs),
            }
        } else {
            self.cycles_delay -= 1;
        }
        // return cycles_delay
        self.cycles_delay
    }
}
