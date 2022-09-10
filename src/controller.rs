use sdl2::keyboard::Keycode;
use std::collections::HashMap;


const RIGHT     : u8        = 0b10000000;
const LEFT      : u8        = 0b01000000;
const DOWN      : u8        = 0b00100000;
const UP        : u8        = 0b00010000;
const START     : u8        = 0b00001000;
const SELECT    : u8        = 0b00000100;
const BUTTON_B  : u8        = 0b00000010;
const BUTTON_A  : u8        = 0b00000001;

#[derive(Default)]
pub struct Controller {
    strobe: bool,
    shift: u8,
    data: u8,
    key_mapping: HashMap<Keycode, u8>,
}


impl Controller {

    pub fn new() -> Self {
        let mut key_mapping = HashMap::new();
        key_mapping.insert(Keycode::W, UP);
        key_mapping.insert(Keycode::A, LEFT);
        key_mapping.insert(Keycode::S, DOWN);
        key_mapping.insert(Keycode::D, RIGHT);
        key_mapping.insert(Keycode::K, BUTTON_A);
        key_mapping.insert(Keycode::J, BUTTON_B);
        key_mapping.insert(Keycode::Return, START);
        key_mapping.insert(Keycode::RShift, SELECT);
        Self {
            data: 0,
            key_mapping: key_mapping,
            shift: 0,
            strobe: false,
        }
    }


    pub fn key_down(&mut self, key: Option<Keycode>) {
        match &key {
            None => (),
            Some(k) => {
                let pressed = self.key_mapping.get(k).unwrap_or(&0);
                self.data |= pressed;
            }
        }
    }

    pub fn key_up(&mut self, key: Option<Keycode>) {
        match &key {
            None => (),
            Some(k) => {
                let released = self.key_mapping.get(k).unwrap_or(&0);
                self.data &= !released;
            }
        }
    }


    pub fn read_u8(&mut self, _addr: u16) -> u8 {
        let ret = match self.shift & self.data {
            0 => 0,
            _ => 1,
        };
        let result = match self.shift {
            0 => 1,
            _ => ret,
        };
        self.shift <<= 1;
        // println!("read controller ret {:#010b}", result);
        result
    }

    pub fn write_u8(&mut self, _addr: u16, val: u8) -> u32 {
        match val & 0x01 {
            0 => {
                self.strobe = false;
            },
            _ => {
                self.strobe = true;
                self.shift = 0x01;
            }
        }
        0
    }
}