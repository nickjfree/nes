
use std::cell::RefCell;
use std::rc::Rc;
use nes::cpu::CPU;
use nes::board::CPUBus;
use nes::cartridge::Cartridge;
use nes::ppu::PPU;

fn main(){

    // create ppu io registers
    let ppu = Rc::new(RefCell::new(PPU::default()));

    // load cartridge data
    let cartridge = Cartridge::load("nestest.nes").expect("load cartridge error");

    let cartridge = Rc::new(RefCell::new(cartridge));

    // println!("loaded cartridge {:?}", cartridge);
    // create cpubus with cartridge and ppu
    let bus = CPUBus::new(Rc::clone(&ppu), Rc::clone(&cartridge));
    // create a cpu for test
    let mut cpu = CPU::new(bus);

    cpu.power_up();

    loop {
        cpu.step();
    }
}
