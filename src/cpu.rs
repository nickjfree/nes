use crate::board::CPUBus;

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
    // tmp operand value
    op_val: u8,
    // bus
    bus: CPUBus,
}



impl CPU {
    //
    pub fn new(bus: CPUBus) -> Self {
        Self {
            bus: bus,
            ..CPU::default()
        }
    }

    // common ops

    fn push_u8(&mut self, val: u8) {
        self.bus.write_u8(self.regs.sp as u16 + STACK_BASE, val);
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
        self.bus.read_u8(self.regs.sp as u16 + STACK_BASE)
    }

    fn pop_u16(&mut self) -> u16 {
        let l = self.pop_u8();
        let h = self.pop_u8();
        (h as u16) << 8 | l as u16
    }

    fn fetch1(&mut self) -> u8 {
        let d = self.bus.read_u8(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        d
    }

    fn fetch2(&mut self) -> u16 {
        let l = self.fetch1();
        let h = self.fetch1();
        (h as u16) << 8 | l as u16 
    }

    // addressing mode

    fn handle_cross_page(&mut self, a: u16, b: u16) {
        if (a >> 8) & 0xff != (b >> 8) & 0xff {
            self.cycles_delay += 1
        }
    }

    fn implied(&mut self) -> u8 {
        0
    }

    fn accumulator(&mut self) -> u8 {
        0
    }

    fn immediate(&mut self) -> u8 {
        self.op_addr = 0xffff;
        self.op_val = self.fetch1();
        self.op_val
    }

    fn zero_page(&mut self) -> u8 {
        self.op_addr = u16::from(self.fetch1() & 0xff);
        self.op_val = self.bus.read_u8(self.op_addr);
        self.op_val
    }

    fn zero_page_x(&mut self) -> u8 {
        // load operand
        let d = self.fetch1();
        self.op_addr = (d.wrapping_add(self.regs.x) & 0xFF).into();
        self.op_val = self.bus.read_u8(self.op_addr);
        self.op_val
    }

    fn zero_page_y(&mut self) -> u8 {
        // load operand
        let d = self.fetch1();
        self.op_addr = (d.wrapping_add(self.regs.y) & 0xFF).into();
        self.op_val = self.bus.read_u8(self.op_addr);
        self.op_val
    }

    fn absolute(&mut self) -> u8 {
        self.op_addr = self.fetch2();
        self.op_val = self.bus.read_u8(self.op_addr);
        self.op_val
    }

    fn absolute_x(&mut self) -> u8 {
        let d = self.fetch2();
        self.op_addr = d.wrapping_add(self.regs.x as u16);
        self.handle_cross_page(self.op_addr, d);
        self.op_val = self.bus.read_u8(self.op_addr);
        self.op_val
    }

    fn absolute_y(&mut self) -> u8 {
        let d = self.fetch2();
        self.op_addr = d.wrapping_add(self.regs.y as u16);
        self.handle_cross_page(self.op_addr, d);
        self.op_val = self.bus.read_u8(self.op_addr);
        self.op_val
    }

    fn indirect_x(&mut self) -> u8 {
        let d = self.fetch1();
        let l = self.bus.read_u8(d.wrapping_add(self.regs.x).into());
        let h = self.bus.read_u8(d.wrapping_add(self.regs.x).wrapping_add(1).into());
        self.op_addr = (h as u16) << 8 | (l as u16);
        self.op_val = self.bus.read_u8(self.op_addr);
        self.op_val
    }

    fn indirect_y(&mut self) -> u8 {
        let d = self.fetch1();
        let l = self.bus.read_u8(d.into());
        let h = self.bus.read_u8(d.wrapping_add(1).into());
        self.op_addr = ((h as u16) << 8 | l as u16).wrapping_add(u16::from(self.regs.y));
        self.handle_cross_page(self.op_addr, (h as u16) << 8 | l as u16);
        self.op_val = self.bus.read_u8(self.op_addr);
        self.op_val
    }

    fn relative(&mut self) -> u8 {
        let d = self.fetch1();
        let rel: u16 = (d as i8) as u16;
        let base = self.regs.pc.wrapping_sub(1);
        self.op_addr = base.wrapping_add(rel);
        self.handle_cross_page(self.op_addr, base);
        0
    }

    fn indirect(&mut self) -> u8 {
        self.op_addr = self.fetch2();
        self.op_addr = self.bus.read_u16(self.op_addr);
        0
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
        self.regs.acc = self.op_val;
        self.flag_nz(self.regs.acc);
    }

    fn ldx(&mut self) {
        self.regs.x = self.op_val;
        self.flag_nz(self.regs.x);
    }

    fn ldy(&mut self) {
        self.regs.y = self.op_val;
        self.flag_nz(self.regs.y);
    }

    fn sta(&mut self) {
        self.bus.write_u8(self.op_addr, self.regs.acc);
    }

    fn stx(&mut self) {
        self.bus.write_u8(self.op_addr, self.regs.x);
    }

    fn sty(&mut self) {
        self.bus.write_u8(self.op_addr, self.regs.y);
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
        self.regs.acc = self.regs.sp;
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
        self.op_val = self.op_val.wrapping_sub(1);
        self.bus.write_u8(self.op_addr, self.op_val);
        self.flag_nz(self.op_val);
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
        self.op_val = self.op_val.wrapping_add(1);
        self.bus.write_u8(self.op_addr, self.op_val);
        self.flag_nz(self.op_val);
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
        let result: u16 = self.regs.acc as u16 + self.op_val as u16 + (self.regs.status & STATUS_CARRAY) as u16;
        match result | 0xFF00 {
            0 => self.regs.status &= !STATUS_CARRAY,
            _ => self.regs.status |= STATUS_CARRAY,
        }
        let result = (result & 0xff) as u8;
        match (self.regs.acc ^ result) & (self.op_val ^ result) & 0x80 {
            0 => self.regs.status &= !STATUS_OVERFLOW,
            _ => self.regs.status |= STATUS_OVERFLOW,
        }
        self.regs.acc = result;
        self.flag_nz(self.regs.acc);
    }

    fn sbc(&mut self) {
        self.op_val = 0_u8.wrapping_sub(self.op_val);
        self.adc();
    }

    // logic operations

    fn and(&mut self) {
        self.regs.acc &= self.op_val;
        self.flag_nz(self.regs.acc);
    }

    fn eor(&mut self) {
        self.regs.acc ^= self.op_val;
        self.flag_nz(self.regs.acc);
    }

    fn ora(&mut self) {
        self.regs.acc |= self.op_val;
        self.flag_nz(self.regs.acc);
    }

    // shift & rotate instrcutions
    fn asl(&mut self) {
        match self.op_val & 0x80  {
            0 => self.regs.status &= !STATUS_CARRAY,
            _ => self.regs.status |= STATUS_CARRAY,
        }
        self.op_val = self.op_val << 1;
        self.bus.write_u8(self.op_addr, self.op_val);
        self.flag_nz(self.op_val);
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
        match self.op_val & 0x01 {
            0 => self.regs.status &= !STATUS_CARRAY,
            _ => self.regs.status |= STATUS_CARRAY,
        }
        self.op_val = self.op_val >> 1;
        self.bus.write_u8(self.op_addr, self.op_val);
        self.flag_nz(self.op_val);
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
        match self.op_val & 0x80  {
            0 => self.regs.status &= !STATUS_CARRAY,
            _ => self.regs.status |= STATUS_CARRAY,
        }
        self.op_val = (self.op_val << 1).wrapping_add(old);
        self.bus.write_u8(self.op_addr, self.op_val);
        self.flag_nz(self.op_val)
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
        match self.op_val & 0x01  {
            0 => self.regs.status &= !STATUS_CARRAY,
            _ => self.regs.status |= STATUS_CARRAY,
        }
        self.op_val = (self.op_val >> 1).wrapping_add(old);
        self.bus.write_u8(self.op_addr, self.op_val);
        self.flag_nz(self.op_val)
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
        let m = 0_u8.wrapping_sub(self.op_val) as u16;
        let result = self.regs.acc as u16 + m;
        match result | 0xFF00 {
            0 => self.regs.status &= !STATUS_CARRAY,
            _ => self.regs.status |= STATUS_CARRAY,
        }
        let result = result as u8;
        self.flag_nz(result);
    }

    fn cpx(&mut self) {
        let m = 0_u8.wrapping_sub(self.op_val) as u16;
        let result = self.regs.x as u16 + m;
        match result | 0xFF00 {
            0 => self.regs.status &= !STATUS_CARRAY,
            _ => self.regs.status |= STATUS_CARRAY,
        }
        let result = result as u8;
        self.flag_nz(result);
    }

    fn cpy(&mut self) {
        let m = 0_u8.wrapping_sub(self.op_val) as u16;
        let result = self.regs.y as u16 + m;
        match result | 0xFF00 {
            0 => self.regs.status &= !STATUS_CARRAY,
            _ => self.regs.status |= STATUS_CARRAY,
        }
        let result = result as u8;
        self.flag_nz(result);
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
        self.push_u16(self.regs.pc);
        self.regs.pc = self.op_addr;
    }

    fn rts(&mut self) {
        self.regs.pc = self.pop_u16();
    }

    // interupts

    fn brk(&mut self) {
        // read next instruction byte (and throw it away)
        self.fetch1();
        // set I flag
        let status = self.regs.status | STATUS_INTERUPT | STATUS_B1 | STATUS_B2;
        self.push_u16(self.regs.pc);
        self.push_u8(status);
        self.regs.pc = self.bus.read_u16(0xfffe);
    }

    fn rti(&mut self) {
        self.regs.status = self.pop_u8() & !STATUS_B1 & !STATUS_B2;
        self.regs.pc = self.pop_u16();
    }

    // others

    fn bit(&mut self) {
        let result = self.regs.acc & self.op_val;
        self.flag_nz(result);
        match self.op_val & STATUS_NEG {
            0 => self.regs.status &= !STATUS_NEG,
            _ => self.regs.status |= STATUS_NEG,
        }
        match self.op_val & STATUS_OVERFLOW {
            0 => self.regs.status &= !STATUS_OVERFLOW,
            _ => self.regs.status |= STATUS_OVERFLOW,
        }
    }

    fn nop(&mut self) {

    }

    // status
    pub fn power_up(&mut self) {
        self.regs.acc = 0;
        self.regs.x = 0;
        self.regs.y = 0;
        self.regs.status |= STATUS_INTERUPT | STATUS_B1 | STATUS_B2;
        self.regs.sp = 0xfd;
        self.regs.pc = self.bus.read_u16(0xfffc);
        println!("pc after power_up {:#02x}", self.regs.pc);
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
    pub fn step(&mut self) -> u32 {

        if self.cycles_delay <= 0 {
            // handle interupt if there was any

            // load next opcode
            self.opcode = self.fetch1();
            // debug
            println!("opcode: {:#02x} regs: {:?}", self.opcode, self.regs);
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
                _=> panic!("unknow opcode {:#02x} {:?}", self.opcode, self.regs),
            }
        } else {
            self.cycles_delay -= 1;
        }
        // return cycles_delay
        self.cycles_delay
    }
}
