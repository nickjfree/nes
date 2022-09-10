extern crate sdl2;

use std::time::{Duration};
use std::error::Error;

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;
// use std::time::Duration;

use std::cell::RefCell;
use std::rc::Rc;
use nes::cpu::{ CPU };
use nes::board::{ Signal };
use nes::cartridge::Cartridge;
use nes::ppu::PPU;
use nes::controller::Controller;


// 1.79 cycle per ms
// const CPU_FREQ: f32 = 1.79;


fn main() -> Result<(), Box<dyn Error>> {

    // *** emulation setup ***

    // nmi signal line
    let nmi = Signal::default();
    // load cartridge data
    let cartridge = Cartridge::load("nestest.nes").expect("load cartridge error");
    let cartridge = Rc::new(RefCell::new(cartridge));
    // controller
    let controller = Rc::new(RefCell::new(Controller::new()));
    // create ppu
    let ppu = PPU::new(Rc::clone(&cartridge), Rc::clone(&nmi));
    let ppu = Rc::new(RefCell::new(ppu));
    // create a cpu for test
    let mut cpu = CPU::new(Rc::clone(&ppu), Rc::clone(&cartridge), Rc::clone(&controller), Rc::clone(&nmi));
    cpu.power_up();

    // test
    {
        let mut p = ppu.borrow_mut();
        // p.load_test_data();
        p.reset();
        // p.write_u8(0x2000, 0x90);
        // p.write_u8(0x2001, 0x1e);
        // p.write_u8(0x2005, 0x00);
        // p.write_u8(0x2005, 0x00);
    }

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

    // create a texture
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 256, 240)
        .map_err(|e| e.to_string())?;


    let mut event_pump = sdl_context.event_pump().unwrap();
    // let mut prev_time = Instant::now();
    'running: loop {
        // handle event
        for event in event_pump.poll_iter() {

            let mut controller = controller.borrow_mut();

            match event {
                Event::Quit {..} => {
                    break 'running
                },
                Event::KeyDown { keycode, .. } => {
                    controller.key_down(keycode);
                },
                Event::KeyUp { keycode, .. } => {
                    controller.key_up(keycode);
                },
                _ => {}
            }
        }
        // emulation for one frame
        let mut present: u8 = 0;
        for _ in 0..29800 {
            cpu.tick();
            let mut ppu = ppu.borrow_mut();
            present |= ppu.tick() | ppu.tick() | ppu.tick();
        }       
        match present {
            0 => (),
            _ => {               
                // time to refresh
                let ppu = ppu.borrow();
                let output = ppu.get_output();
                texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
                    for y in 0..240 {
                        for x in 0..256 {
                            let offset = y * pitch + x * 3;
                            buffer[offset] = output[offset];
                            buffer[offset + 1] = output[offset + 1];
                            buffer[offset + 2] = output[offset + 2];
                        }
                    }
                })?;
                canvas.clear();
                canvas.copy(&texture, None, None)?;
                canvas.present();
            },
        }        
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    println!("byte!");
    Ok(())
}
