
use std::error::Error;
use std::fs::File;
use std::rc::Rc;
use std::cell::RefCell;
use byteorder::ReadBytesExt;
use crate::board::Memory;
use crate::mapper::{MirroMode, Mapper, NRom, PRG_BANK_SIZE, CHR_BANK_SIZE};



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
#[derive(Default)]
pub struct Cartridge {
    header: CartridgeHeader,
    mapper: Option<Box<dyn Mapper>>,
}

impl Cartridge {

    fn default() -> Self {
        Self {
            header: CartridgeHeader::default(),
            mapper: None,
        }
    }


    // load cartridge data from reader
    fn read<T: ReadBytesExt>(reader: &mut T) -> Result<Self, Box<dyn Error>> {

        let mut cartridge = Cartridge::default();
        let header = CartridgeHeader::read(reader)?;

        let mut prg: Memory = Memory::new(PRG_BANK_SIZE * 1);
        let mut chr: Memory = Memory::new(CHR_BANK_SIZE * 1);
        if header.num_prg > 0 {
            prg = Memory::new(PRG_BANK_SIZE * header.num_prg as usize);
            reader.read_exact(&mut prg)?;
        }
        if header.num_chr > 0 {
            chr = Memory::new(CHR_BANK_SIZE * header.num_chr as usize);
            reader.read_exact(&mut chr)?;
        }
        let mirror_mode = match header.flag1 & 0x01 {
            0 => MirroMode::Horizontal,
            _ => MirroMode::Vertical,
        };
        cartridge.header = header;
        // we only create NROM for now
        let mapper = NRom::new(prg, chr, mirror_mode);
        cartridge.mapper = Some(Box::new(mapper));
        Ok(cartridge)
    }

    // load cartridge from nes file
    pub fn load(file: &str) -> Result<Cartridge, Box<dyn Error>> {
        let mut file = File::open(file)?;
        let cartridge = Cartridge::read(&mut file)?;
        Ok(cartridge)
    }

    pub fn to_mapper(self) -> Rc<RefCell<Box<dyn Mapper>>> {
        Rc::new(RefCell::new(self.mapper.unwrap()))
    }
}
