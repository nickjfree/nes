
use std::error::Error;
use std::fs::File;
use byteorder::ReadBytesExt;
use crate::board::Ram;

// cartridge header
#[derive(Default, Debug)]
struct CartridgeHeader {
    magic:    [u8; 4],
    num_prg:  u8,
    num_chr:  u8,
    flag1:    u8,
    flag2:    u8,
    num_ram:  u8,
    reserved: [u8; 7],
}

impl CartridgeHeader {

    // load cartridge header from reader
    fn read<T: ReadBytesExt>(reader: &mut T)  -> Result<Self, Box<dyn Error>> {
        let mut header = CartridgeHeader::default();
        reader.read_exact(&mut header.magic)?;
        header.num_prg = reader.read_u8()?;
        header.num_chr = reader.read_u8()?;
        header.flag1 = reader.read_u8()?;
        header.flag2 = reader.read_u8()?;
        header.num_ram = reader.read_u8()?;
        reader.read_exact(&mut header.reserved)?;

        Ok(header)
    }
}

// cartridge
#[derive(Default, Debug)]
pub struct Cartridge {
    header: CartridgeHeader,
    prg_roms: Vec<Ram>,
    chr_roms: Vec<Ram>,
}

impl Cartridge {

    // load cartridge data from reader
    fn read<T: ReadBytesExt>(reader: &mut T) -> Result<Self, Box<dyn Error>> {

        let mut cartridge = Cartridge::default();
        let header = CartridgeHeader::read(reader)?;

        for _n in 0..header.num_prg {
            let mut ram = Ram::new(16 * 1024);
            reader.read_exact(&mut ram)?;
            cartridge.prg_roms.push(ram)
        }
        for _n in 0..header.num_chr {
            let mut ram = Ram::new(8 * 1024);
            reader.read_exact(&mut ram)?;
            cartridge.chr_roms.push(ram)
        }
        cartridge.header = header;
        Ok(cartridge)
    }

    // load cartridge from nes file
    pub fn load(file: &str) -> Result<Cartridge, Box<dyn Error>> {
        let mut file = File::open(file)?;
        let cartridge = Cartridge::read(&mut file)?;
        Ok(cartridge)
    }

    // get reference of program rom 
    pub fn program(&self, index: usize) -> &Ram {
        let index = index % self.prg_roms.len();
        &self.prg_roms[index]
    }

    // get mutable reference of program rom 
    pub fn program_mut(&mut self, index: usize) -> &mut Ram {
        let index = index % self.prg_roms.len();
        &mut self.prg_roms[index]
    }
}
