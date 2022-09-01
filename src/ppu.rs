
use crate::board::Ram;

// ppu
#[derive(Default)]
pub struct PPU {

    // vram space 16K

    // $0000-$0FFF  $1000   Pattern table 0
    // $1000-$1FFF  $1000   Pattern table 1
    // $2000-$23FF  $0400   Nametable 0
    // $2400-$27FF  $0400   Nametable 1
    // $2800-$2BFF  $0400   Nametable 2
    // $2C00-$2FFF  $0400   Nametable 3
    // $3000-$3EFF  $0F00   Mirrors of $2000-$2EFF
    // $3F00-$3F1F  $0020   Palette RAM indexes
    // $3F20-$3FFF  $00E0   Mirrors of $3F00-$3F1F
    vram: Ram,

    // ppu contrl flags

    // nmi enabled 0: off 1: on
    nmi_enabled: bool,  
    // nmi occurred status
    nmi_occurred: bool,
    // vram address increment 0: add 1  1: add 32
    vram_increment: u16,
    // sprite szie 0: 8x8 1: 8x16
    sprite_size: u8,
    // sprite pattern selection 0: 0x0000 1: 0x1000
    sprite_table: u8,
    // backgroun pattern selection 0: 0x0000 1: 0x1000
    background_table: u8,
    // master/slave  0: read backdrop from EXT pins; 1: output color on EXT pins
    master_mode: bool, 

    // rendering mask
    greyscale: bool,
    show_left_background: bool,
    show_left_sprite: bool,
    show_background: bool,
    show_sprite: bool,
    // Emphasize mode not implemented

    // status
    sprite_overflow: bool,
    sprite_0_hit: bool,


    // frame number
    frame_number: u32,
    // oam addr
    oam_addr: u8,
    // oam data
    oam_data: u8,
    // vram data read buffer
    vram_read_buffer: u8,
    x: u16,
    y: u16,
    // toggle for 2x write
    write_toggle: bool,


}


impl PPU {

    pub fn new() -> Self {
        Self{
            vram: Ram::new(16384),
            ..PPU::default()
        }
    }

    pub fn read_u8(&mut self, addr: u16) -> u8 {
        0
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) -> u32 {
        0
    }
}