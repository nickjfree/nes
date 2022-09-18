extern crate sdl2;

use std::error::Error;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;

use nes::cpu::{ CPU };
use nes::board::{ Signal };
use nes::cartridge::Cartridge;
use nes::ppu::PPU;
use nes::controller::Controller;
use nes::clock::Clock;


fn main() -> Result<(), Box<dyn Error>> {

    // *** emulation setup ***

    // nmi signal line
    let nmi = Signal::default();
    // load cartridge data
    let mapper = Cartridge::load("/mnt/f/proj/nes/roms/Contra (U).nes").expect("load cartridge error").to_mapper();
    // controller
    let controller = Rc::new(RefCell::new(Controller::new()));
    // create ppu
    let ppu = PPU::new(Rc::clone(&mapper), Rc::clone(&nmi));
    let ppu = Rc::new(RefCell::new(ppu));
    // create a cpu for test
    let mut cpu = CPU::new(Rc::clone(&ppu), Rc::clone(&mapper), Rc::clone(&controller), Rc::clone(&nmi));
    cpu.power_up();

    // reset ppu
    {
        let mut p = ppu.borrow_mut();
        p.reset();
    }
    // clock
    let mut clock = Clock::new();

    // **** gui setup  ****

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("nes", 256 * 4, 240 * 4)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().accelerated().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    // create a texture
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 256, 240)
        .map_err(|e| e.to_string())?;


    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut prev_time = Instant::now();
    // game loop
    'running: loop {
        // handle key event
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
        // emulation for one loop
        let mut end_frame: u8 = 0;
        let cycles = clock.get_cycles_past();
        for _ in 0..cycles {
            cpu.tick();
            let mut ppu = ppu.borrow_mut();
            end_frame |= ppu.tick();
            end_frame |= ppu.tick();
            end_frame |= ppu.tick();
        }
        if end_frame != 0 {
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
            println!("{:?} {}", prev_time.elapsed().as_millis(), cycles);
            prev_time = Instant::now();
        }
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 120));
    }
    println!("bye!");
    Ok(())
}
