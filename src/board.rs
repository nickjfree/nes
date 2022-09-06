use std::ops::{ Deref, DerefMut};
use std::cell::RefCell;
use std::rc::Rc;


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

pub type Signal = Rc<RefCell<u8>>;
