extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

use std::cell::RefCell;
use std::rc::Rc;
use nes::cpu::CPU;
use nes::board::CPUBus;
use nes::cartridge::Cartridge;
use nes::ppu::PPU;

fn main(){

    // *** emulation setip ***

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



    // **** gui setup  ****

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("nes", 256, 240)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();


    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    println!("break1");
                    break 'running
                },
                _ => {}
            }
        }
        // emulation logics
        cpu.step();



        canvas.present();
        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    println!("quit");
}
