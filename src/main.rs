extern crate sdl2;

use std::time::{Duration, Instant};

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::render::Texture;
use sdl2::surface::Surface;
use sdl2::rect::Rect;
// use sdl2::keyboard::Keycode;
// use std::time::Duration;

use std::cell::RefCell;
use std::rc::Rc;
use nes::cpu::CPU;
use nes::board::CPUBus;
use nes::cartridge::Cartridge;
use nes::ppu::PPU;


fn main(){

    // *** emulation setip ***

    // create ppu io registers
    let ppu = Rc::new(RefCell::new(PPU::new()));

    // load cartridge data
    let cartridge = Cartridge::load("nestest.nes").expect("load cartridge error");
    let cartridge = Rc::new(RefCell::new(cartridge));

    // println!("loaded cartridge {:?}", cartridge);
    // create cpu bus with cartridge and ppu
    let bus = CPUBus::new(Rc::clone(&ppu), Rc::clone(&cartridge));
    // create a cpu for test
    let mut cpu = CPU::new(bus);
    cpu.power_up();

    let mut p = ppu.borrow_mut();
    p.load_test_data();
    p.reset();
    p.write_u8(0x2001, 0x0f);
    drop(p);


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

    let mut prev_time = Instant::now();
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
        // cpu.step();
        // ppu 3 times
        let mut ppu = ppu.borrow_mut();
        match ppu.step() | ppu.step() |  ppu.step() {
            0 => (),
            _ => {               
                let duration = prev_time.elapsed();
                // println!("Time elapsed in one frame is: {:?}", duration);
                println!("{:?}", ppu.get_output());
                // time to refresh
                canvas.present();
                prev_time = Instant::now();
            },
        }        
        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    println!("quit");
}
