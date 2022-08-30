
use std::cell::RefCell;
use std::rc::Rc;
use nes::cpu::CPU;
use nes::board::{CPUBus, IORegisters};
use nes::cartridge::Cartridge;

fn main(){

	// create ppu io registers
	let ppu_regs = Rc::new(RefCell::new(IORegisters::new(8)));

	// load cartridge data
	let cartridge = Cartridge::load("nestest.nes").expect("load cartridge error");

	// println!("loaded cartridge {:?}", cartridge);
	// create cpubus
	let bus = CPUBus::new(ppu_regs, Rc::new(RefCell::new(cartridge)));
	// create a cpu for test
	let mut cpu = CPU::new(bus);

	cpu.power_up();

	loop {
		cpu.step();
	}
}
