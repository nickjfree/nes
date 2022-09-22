
use std::error::Error;
use std::fs::File;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use byteorder::ReadBytesExt;
use crate::board::{ Memory, Signal };
use crate::mapper::{MirroMode, Mapper, NRom, UxRom, MMC3, PRG_BANK_SIZE, CHR_BANK_SIZE};


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

impl fmt::Display for CartridgeHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mirror = match self.flag1 & 0x01 {
            0 => "h",
            _ => "v",
        };
        let mapper_number = ((self.flag1 >> 4) & 0x0f) | (self.flag2 & 0xf0);
        let has_trainer = self.flag1 & 0x04 != 0;

        write!(f, "prg: {}K, chr: {}K, trainer: {}, mirror: {}, mapper: {}",
            self.num_prg * 16, self.num_chr * 8, has_trainer, mirror, mapper_number)?;
        Ok(())
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
    fn read<T: ReadBytesExt>(reader: &mut T, irq: Signal) -> Result<Self, Box<dyn Error>> {

        let mut cartridge = Cartridge::default();
        let header = CartridgeHeader::read(reader)?;

        println!("cartridge info {}", header);

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
        // get mapper id
        let mapper_number = ((header.flag1 >> 4) & 0x0f) | (header.flag2 & 0xf0);
        cartridge.header = header;

        let mapper: Box<dyn Mapper> = match mapper_number {
            0 => Box::new(NRom::new(prg, chr, mirror_mode)),
            2 => Box::new(UxRom::new(prg, chr, mirror_mode)),
            4 => Box::new(MMC3::new(prg, chr, mirror_mode, irq)),
            _ => panic!("unsupported mapper {}", mapper_number),
        };
        cartridge.mapper = Some(mapper);
        Ok(cartridge)
    }

    // load cartridge from nes file
    pub fn load(file: &str, irq: Signal) -> Result<Cartridge, Box<dyn Error>> {
        let mut file = File::open(file)?;
        let cartridge = Cartridge::read(&mut file, irq)?;
        Ok(cartridge)
    }

    pub fn to_mapper(self) -> Rc<RefCell<Box<dyn Mapper>>> {
        Rc::new(RefCell::new(self.mapper.unwrap()))
    }
}
