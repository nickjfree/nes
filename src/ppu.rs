use std::ops::{Deref, DerefMut};
use std::cell::RefCell;
use std::rc::Rc;
use crate::board::{ Memory, Signal };
use crate::mapper::Mapper;

const PPUCTRL: u16    = 0x2000;
const PPUMASK: u16    = 0x2001;
const PPUSTATUS: u16  = 0x2002;
const OAMADDR: u16    = 0x2003;
const OAMDATA: u16    = 0x2004;
const PPUSCROLL: u16  = 0x2005;
const PPUADDR: u16    = 0x2006;
const PPUDATA: u16    = 0x2007;

// sprite in oam
trait Sprite {
    fn x(&self) -> u8;
    fn y(&self) -> u8;
    fn pattern_index(&self) -> u8;
    fn pallette_index(&self) -> u8;
    fn bank_addr(&self) -> u16;
    fn front(&self) -> bool;
    fn flip_h(&self) -> bool;
    fn flip_v(&self) -> bool;
}


impl Sprite for [u8; 4] {

    fn y(&self) -> u8 {
        self[0]
    }

    fn x(&self) -> u8 {
        self[3]
    }

    fn pattern_index(&self) -> u8 {
        self[1]
    }

    fn bank_addr(&self) -> u16 {
        match self[1] & 0x01 {
            0 => 0x0000,
            _ => 0x1000,
        }
    }

    fn pallette_index(&self) -> u8 {
        (self[2] & 0x03) << 2
    }

    fn front(&self) -> bool {
        match self[2] & 0x20 {
            0 => true,
            _ => false,
        }
    }

    fn flip_h(&self) -> bool {
        match self[2] & 0x40 {
            0 => false,
            _ => true,
        }
    }

    fn flip_v(&self) -> bool {
        match self[2] & 0x80 {
            0 => false,
            _ => true,
        }
    }
}

// ppu oam
struct OAM {
    sprites: Vec<[u8; 4]>,
}

impl OAM {
    fn new(size: usize) -> Self {
        Self{
            sprites: vec![[0, 0, 0, 0]; size],
        }
    }
}

impl Default for OAM {
    fn default() -> Self {
        Self{
            sprites: vec![[0, 0, 0, 0]; 64],
        }
    }
}

impl Deref for OAM {

    type Target = Vec<[u8; 4]>;

    fn deref(&self) -> &Self::Target {
        &self.sprites
    }
}

impl DerefMut for OAM {

    fn deref_mut(&mut self) -> &mut Vec<[u8; 4]> {
        &mut self.sprites
    }
}

#[derive(Default, Copy, Clone)]
struct FetchedSprite {
    data: u32,
    sprite: [u8; 4],
    row: u8,
    sprite_index: u8,
    dummy: bool,
}

impl FetchedSprite {

    fn default() -> Self {
        Self {
            data: 0,
            sprite: [0, 255, 0, 0],
            row: 0,
            sprite_index: 255,
            dummy: true,
        }
    }

    fn fetch(&self, cycle: u16) -> u8 {
        let d = cycle.wrapping_sub(self.x() as u16);
        if d < 8 {
            ((self.data >> ((7 - d) * 4)) & 0x0f) as u8
        } else {
            0
        }
    }
}

impl Deref for FetchedSprite {

    type Target = [u8; 4];

    fn deref(&self) -> &Self::Target {
        &self.sprite
    }
}

// ppu bus
//
//
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
struct PPUBus {
    // mapper as chr rom and name_table
    mapper: Rc<RefCell<Box<dyn Mapper>>>,
    pallette: Memory,
}


impl PPUBus {

    pub fn new(mapper: Rc<RefCell<Box<dyn Mapper>>>) -> Self {
        Self {
            mapper: mapper,
            pallette: Memory::new(32),
        }
    }

    pub fn read_u8(&mut self, addr: u16) -> u8 {
        let addr = addr & 0x3fff;
        match addr {
            0x0000..=0x3eff => {
                self.mapper.borrow_mut().read_u8(addr)
            },
            0x3f00..=0x3fff => {
                // mirror: 3f10 3f14 3f18 3f1c mirror to 3f00 3f04 3f08 3f0c
                // TODO: background palette hack
                let addr = match addr & 0x03 {
                    0 => addr & 0x0f,
                    _ => addr & 0x1f,
                };
                self.pallette.read_u8(addr)
            },
            _ => panic!("read vram address {:#02x}", addr),
        }
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        let addr = addr & 0x3fff;
        match addr {
            0x0000..=0x3eff => {
                self.mapper.borrow_mut().write_u8(addr, val)
            },
            0x3f00..=0x3fff => {
                let addr = match addr & 0x03 {
                    0 => addr & 0x0f,
                    _ => addr & 0x1f,
                };
                self.pallette.write_u8(addr & 0x1f, val);
            },
            _ => panic!("write vram address {:#02x}", addr),
        }
    }
}

// ppu internal registers
#[derive(Default, Debug)]
struct PPURegisters {
    // oam addr
    oam_addr: u8,
    // vram data read buffer
    vram_read_buffer: u8,
    // current vram address
    v: u16,
    // tempoeray vram address
    t: u16,
    // fine x scroll
    x: u8,
    // write toggle
    w: u8,
    // bus data
    bus_data: u8,
}


impl PPURegisters {
    
    fn inc_hori_v(&mut self) {
        let coarse_x = self.v & 0x001f;
        match coarse_x {
            31 => {
                self.v &= !0x001f;
                self.v ^= 0x0400;
            },
            _ => {
                self.v = self.v.wrapping_add(1);
            }
        }
    }

    fn inc_vert_v(&mut self) {
        let fine_y = self.v & 0x7000;
        match fine_y {
            0x7000 => {
                // finey = 0
                self.v &= !0x7000; 
                let y = (self.v & 0x03e0)  >> 5;
                let coarse_y = match y {
                    29 => {self.v ^= 0x0800; 0},
                    31 => 0,
                    _ => y.wrapping_add(1),
                };
                self.v = (self.v & !0x03e0) | (coarse_y << 5);
            },
            _ => { 
                // fine y < 7
                self.v = self.v.wrapping_add(0x1000);
            },
        }
    }

    fn copy_hori_t(&mut self) {
        // hori(v) = hori(t)
        // v: .....F.. ...EDCBA = t: .....F.. ...EDCBA
        self.v = (self.v & 0xFBE0) | (self.t & 0x041F);
    }

    fn copy_vert_t(&mut self) {
        // vert(v) = vert(t)
        // v: .IHGF.ED CBA..... = t: .IHGF.ED CBA.....
        self.v = (self.v & 0x841f) | (self.t & 0x7be0);
    }
}



// ppu rendering status
#[derive(Default, Debug)]
struct RenderStatus {

    // rendering mask
    greyscale: bool,
    show_left_background: bool,
    show_left_sprite: bool,
    show_background: bool,
    show_sprite: bool,
    // TODO: color emphasize mode not implemented

    // current scanline
    scanline: u16,
    // current cycle
    cycle: u16,
    // sprite overflow flag
    sprite_overflow: bool,
    // sprite 0 hit flag
    sprite_0_hit: bool,
    // frame number
    frame_number: u32,
    // nt tile byte
    tile_index: u8,
    // at data
    at_data: u8,
    // tile low
    tile_low: u8,
    // tile high
    tile_high: u8,
    // shift register for 2 tiles, with pallette index
    tile_data: u64,

    // nmi suppress timer
    nmi_suppress_timer: u32,
    // nmi_occurred_timer  
    nmi_occurred_timer: u32,
    // debug
    nmi_frame: u32,
}


impl RenderStatus {

    // render enabled
    fn is_render_enabled(&self) -> bool {
        self.show_background | self.show_sprite
    }

    // is in odd frame
    fn is_odd_frame(&self) -> bool {
        (self.frame_number % 2) != 0
    }

    // advance cycle
    fn inc_cycle(&mut self) -> bool {
        // if self.scanline == 62 {
        //     println!("line {} {}", self.scanline, self.cycle);
        // }
        self.cycle += 1;
        if self.cycle > 340 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline > 261 {
                self.scanline = 0;
                self.frame_number = self.frame_number.wrapping_add(1);
                return true
            }
        }
        // dec nmi suppress timer
        if self.nmi_suppress_timer > 0 {
            self.nmi_suppress_timer -= 1;
        }
        // dec nnmi_occurred_timer
        if self.nmi_occurred_timer > 0 {
            self.nmi_occurred_timer -= 1;
        }
        false
    }
}




// ppu
pub struct PPU {

       
    // ppu bus
    ppu_bus: PPUBus,

    // ppu registers
    regs: PPURegisters,

    // OAM
    oam: OAM,
    // fetched sprite
    sprite_cache: Vec<FetchedSprite>,

    // render status
    rs: RenderStatus,

    // palette color
    palette_color: Vec<u8>,

    // output buffer
    output: Vec<u8>,
    // ppu contrl flags
    // nmi enabled 0: off 1: on
    nmi_enabled: bool,  
    // nmi occurred status
    nmi_occurred: bool,
    // previous nmi
    nmi_prev: bool,
    // nmi signal line
    nmi: Signal,
    // vram address increment 0: add 1  1: add 32
    vram_increment: u16,
    // sprite szie 0: 8x8 1: 8x16
    sprite_size: u8,
    // sprite pattern selection 0: 0x0000 1: 0x1000
    sprite_table: u16,
    // backgroun pattern selection 0: 0x0000 1: 0x1000
    background_table: u16,
    // master/slave  0: read backdrop from EXT pins; 1: output color on EXT pins
    // master_mode: bool, 
}


impl PPU {

    pub fn new(mapper: Rc<RefCell<Box<dyn Mapper>>>, nmi: Signal) -> Self {
        Self{
            background_table: 0,
            sprite_table: 0,
            nmi_enabled: false,
            nmi_occurred: false,
            nmi_prev: false,
            vram_increment: 1,
            sprite_size: 8,
            rs: RenderStatus::default(),
            regs: PPURegisters::default(),
            oam: OAM::new(64),
            sprite_cache: vec![FetchedSprite::default(); 8],
            ppu_bus: PPUBus::new(mapper),
            output: vec![0; 256*240*3],
            palette_color: vec![
                 84,  84,  84,  0,     0,  30, 116, 0,    8,  16, 144, 0,     48,   0, 136, 0,    68,   0, 100, 0,    92,   0,  48, 0,    84,   4,   0, 0,    60,  24,   0, 0,     32,  42,   0, 0,      8,  58,   0, 0,     0,  64,   0,  0,    0,  60,   0, 0,     0,  50,  60, 0,    0,   0,   0, 0,   0, 0, 0, 0,  0, 0, 0, 0,  
                152, 150, 152,  0,     8,  76, 196, 0,   48,  50, 236, 0,     92,  30, 228, 0,   136,  20, 176, 0,   160,  20, 100, 0,   152,  34,  32, 0,   120,  60,   0, 0,     84,  90,   0, 0,     40, 114,   0, 0,     8, 124,   0,  0,    0, 118,  40, 0,     0, 102, 120, 0,    0,   0,   0, 0,   0, 0, 0, 0,  0, 0, 0, 0,  
                236, 238, 236,  0,    76, 154, 236, 0,  120, 124, 236, 0,    176,  98, 236, 0,   228,  84, 236, 0,   236,  88, 180, 0,   236, 106, 100, 0,   212, 136,  32, 0,    160, 170,   0, 0,    116, 196,   0, 0,    76, 208,  32,  0,   56, 204, 108, 0,    56, 180, 204, 0,   60,  60,  60, 0,   0, 0, 0, 0,  0, 0, 0, 0,  
                236, 238, 236,  0,   168, 204, 236, 0,  188, 188, 236, 0,    212, 178, 236, 0,   236, 174, 236, 0,   236, 174, 212, 0,   236, 180, 176, 0,   228, 196, 144, 0,    204, 210, 120, 0,    180, 222, 120, 0,   168, 226, 144,  0,  152, 226, 180, 0,   160, 214, 228, 0,  160, 162, 160, 0,   0, 0, 0, 0,  0, 0, 0, 0,  
            ],
            nmi: nmi,
        }
    }

    // read ppu registers
    pub fn read_u8(&mut self, addr: u16) -> u8 {
        // println!("PPU: read {:#04x}", addr);
        match addr {
            // read ppu status
            //             7  bit  0
            // ---- ----
            // VSO. ....
            // |||| ||||
            // |||+-++++- PPU open bus. Returns stale PPU bus contents.
            // ||+------- Sprite overflow. The intent was for this flag to be set
            // ||         whenever more than eight sprites appear on a scanline, but a
            // ||         hardware bug causes the actual behavior to be more complicated
            // ||         and generate false positives as well as false negatives; see
            // ||         PPU sprite evaluation. This flag is set during sprite
            // ||         evaluation and cleared at dot 1 (the second dot) of the
            // ||         pre-render line.
            // |+-------- Sprite 0 Hit.  Set when a nonzero pixel of sprite 0 overlaps
            // |          a nonzero background pixel; cleared at dot 1 of the pre-render
            // |          line.  Used for raster timing.
            // +--------- Vertical blank has started (0: not in vblank; 1: in vblank).
            //            Set at dot 1 of line 241 (the line *after* the post-render
            //            line); cleared after reading $2002 and at dot 1 of the
            //            pre-render line.
            PPUSTATUS => {
                // low 5bit is stale bus content
                let mut ret = self.regs.bus_data & 0x1f;
                if self.rs.sprite_overflow {
                    ret |= 0x20;
                }
                if self.rs.sprite_0_hit {
                    ret |= 0x40;
                }
                if self.nmi_occurred {
                    ret |= 0x80;
                }
                self.regs.w = 0;
                // read this will clear nmi
                self.nmi_occurred = false;
                self.rs.nmi_suppress_timer = 2;
                ret
            },
            // read oam data
            OAMDATA => {
                let n: usize = usize::from((self.regs.oam_addr >> 2) & 0x3f);
                let m: usize = usize::from(self.regs.oam_addr & 0x03);
                self.oam[n][m]
            },
            // read ppu vram data
            PPUDATA => {
                let ret;
                // When reading while the VRAM address is in the range 0-$3EFF (i.e., before the palettes), the read will return the contents of an internal read buffer. 
                // This internal buffer is updated only when reading PPUDATA, and so is preserved across frames. 
                // After the CPU reads and gets the contents of the internal buffer, the PPU will immediately update the internal buffer with the byte at the current VRAM address. 
                // Thus, after setting the VRAM address, one should first read this register to prime the pipeline and discard the result.
                // Reading palette data from $3F00-$3FFF works differently. The palette data is placed immediately on the data bus, and hence no priming read is required. 
                //Reading the palettes still updates the internal buffer though, but the data placed in it is the mirrored nametable data that would appear "underneath" the palette. (Checking the PPU memory map should make this clearer.)
                match self.regs.v & 0x3fff {
                    // buffered
                    0x0000..=0x3eff => {
                        ret = self.regs.vram_read_buffer;
                        self.regs.vram_read_buffer = self.ppu_bus.read_u8(self.regs.v);
                    },
                    // not buffered
                    0x3f00..=0x3fff => {
                        ret = self.ppu_bus.read_u8(self.regs.v);
                        // the buffered data is nametable mirror
                        self.regs.vram_read_buffer = self.ppu_bus.read_u8((self.regs.v - 0x2000) % 4096 + 0x2000);
                    },
                    _ => panic!("read vram address {:#02x}", self.regs.v),
                }
                // println!("before ppu_data {:?}, ret: {:#04x}, inc {:?}", self.regs, ret, self.vram_increment);
                self.regs.v = self.regs.v.wrapping_add(self.vram_increment);
                // println!("ppu_data {:?}, ret: {:#04x}, inc {:?}", self.regs, ret, self.vram_increment);
                ret
            },
            _ => 0,
        }
    }

    // write ppu registers
    pub fn write_u8(&mut self, addr: u16, val: u8) {
        // println!("PPU: write {:#02x} {:#02x} {:?}", addr, val, self.regs);
        self.regs.bus_data = val;
        match addr {
            // write ppu ctrl
            // 
            //             7  bit  0
            // ---- ----
            // VPHB SINN
            // |||| ||||
            // |||| ||++- Base nametable address
            // |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
            // |||| |+--- VRAM address increment per CPU read/write of PPUDATA
            // |||| |     (0: add 1, going across; 1: add 32, going down)
            // |||| +---- Sprite pattern table address for 8x8 sprites
            // ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
            // |||+------ Background pattern table address (0: $0000; 1: $1000)
            // ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels – see PPU OAM#Byte 1)
            // |+-------- PPU master/slave select
            // |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
            // +--------- Generate an NMI at the start of the
            //            vertical blanking interval (0: off; 1: on)
            PPUCTRL =>  {
                let val = val as u16;
                // t: ...GH.. ........ <- d: ......GH
                // <used elsewhere> <- d: ABCDEF..
                self.regs.t = (self.regs.t & 0xf3ff) | ((val & 0x03) << 10);
                self.vram_increment = if val & 0x04 == 0 { 1 } else { 32 };
                self.background_table = if val & 0x10 == 0 { 0x0000 } else { 0x1000 };
                self.sprite_table = if val & 0x08 == 0 { 0x0000 } else { 0x1000 };
                self.sprite_size = if val & 0x20 == 0 { 8 } else { 16 };
                self.nmi_enabled = if val & 0x80 == 0 { false } else { true };
            },
            // write ppu mask
            //             7  bit  0
            // ---- ----
            // BGRs bMmG
            // |||| ||||
            // |||| |||+- Greyscale (0: normal color, 1: produce a greyscale display)
            // |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
            // |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
            // |||| +---- 1: Show background
            // |||+------ 1: Show sprites
            // ||+------- Emphasize red (green on PAL/Dendy)
            // |+-------- Emphasize green (red on PAL/Dendy)
            // +--------- Emphasize blue
            PPUMASK => {
                self.rs.greyscale = if val & 0x01 != 0 { true } else { false };
                self.rs.show_left_background = if val & 0x02 != 0 { true } else { false };
                self.rs.show_left_sprite = if val & 0x04 != 0 { true } else { false };
                self.rs.show_background = if val & 0x08 != 0 { true } else { false };
                self.rs.show_sprite = if val & 0x10 != 0 { true } else { false };
            },
            // write ppu oam addr
            OAMADDR => {
                self.regs.oam_addr = val;
            },
            // write ppu oam data
            OAMDATA => {
                let n: usize = usize::from((self.regs.oam_addr >> 2) & 0x3f);
                let m: usize = usize::from(self.regs.oam_addr & 0x03);
                self.oam[n][m] = val;
                self.regs.oam_addr = self.regs.oam_addr.wrapping_add(1);
            },
            // write ppu scroll
            PPUSCROLL => {   
                // println!("PPU: write {:#02x} {:#02x} {:?}", addr, val, self.rs.scanline);
                let val = val as u16;
                match self.regs.w {
                    0 => {
                        // t: ....... ...ABCDE <- d: ABCDE...
                        // x:              FGH <- d: .....FGH
                        // w:                  <- 1
                        self.regs.t = (self.regs.t & 0xffe0) | ((val >> 3) & 0x1f);
                        self.regs.x = (val & 0x07) as u8;
                        self.regs.w = 1;
                    },
                    1 => {
                        //    10001100 00011111
                        //  t:.FGH..AB CDE..... <- d: ABCDEFGH
                        //  w:                  <- 0
                        self.regs.t = (self.regs.t & 0x8c1f) | ((val & 0xf8) << 2) | ((val & 0x07) << 12);
                        self.regs.w = 0;
                    },
                    _ => (),
                }
            },
            // write vram addr
            PPUADDR => {
                //println!("PPU: write {:#06x} {:#06x} {:?}", addr, val, self.regs);
                let val = val as u16;
                match self.regs.w {
                   
                    0 => {
                        // t: .CDEFGH ........ <- d: ..CDEFGH
                        //        <unused>     <- d: AB......
                        // t: Z...... ........ <- 0 (bit Z is cleared)
                        // w:                  <- 1     
                        self.regs.t = (self.regs.t & 0xc0ff) | ((val & 0x003f) << 8);
                        self.regs.w = 1;
                    },
                    1 => {
                        // t: ....... ABCDEFGH <- d: ABCDEFGH
                        // v: <...all bits...> <- t: <...all bits...>
                        // w:                  <- 0
                        self.regs.t = (self.regs.t & 0xff00) | (val & 0x00ff);
                        self.regs.v = self.regs.t;
                        self.regs.w = 0;

                    },
                    _ => (),
                }
            },
            // write vram data
            PPUDATA => {
                self.ppu_bus.write_u8(self.regs.v, val);
                self.regs.v = self.regs.v.wrapping_add(self.vram_increment);
            },
            _ => (),
        }
    }

    pub fn oam_dma(&mut self, data: &[u8]) {
        // println!("oam dma");
        let mut oam_addr: u8 = self.regs.oam_addr;
        data.iter().for_each(|x| {
            let n: usize = usize::from((oam_addr >> 2) & 0x3f);
            let m: usize = usize::from(oam_addr & 0x03);
            self.oam[n][m] = *x;
            oam_addr = oam_addr.wrapping_add(1);
        });
    }

    pub fn reset(&mut self) {
        self.rs.cycle = 340;
        self.rs.scanline = 240;
        self.rs.frame_number = 0;
        self.write_u8(PPUCTRL, 0);
        self.write_u8(PPUMASK, 0);
        self.write_u8(OAMADDR, 0);
    }

      
    fn fetch_nt(&mut self) {
        // tile address      = 0x2000 | (v & 0x0FFF)
        let addr = 0x2000 | (self.regs.v & 0x0fff);
        self.rs.tile_index = self.ppu_bus.read_u8(addr);
    }

    fn fetch_at(&mut self) {
        // attribute address = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07)
        let addr = 0x23C0 | (self.regs.v & 0x0C00) | ((self.regs.v >> 4) & 0x38) | ((self.regs.v >> 2) & 0x07);
        let at = self.ppu_bus.read_u8(addr);
        let shift = ((self.regs.v >> 4) & 0x4) | (self.regs.v & 0x02);
        self.rs.at_data = (at >> shift & 0x03) << 2;
    }

    fn fetch_bg_tile_low(&mut self) {
        let fine_y = (self.regs.v >> 12) & 0x07;
        let index = self.rs.tile_index as u16;
        let addr = self.background_table.wrapping_add((index << 4) | fine_y);
        self.rs.tile_low = self.ppu_bus.read_u8(addr);
    }

    fn fetch_bg_tile_high(&mut self) {
        let fine_y = (self.regs.v >> 12) & 0x07;
        let index = self.rs.tile_index as u16;
        let addr = self.background_table.wrapping_add((index << 4) | 0x08 | fine_y );
        self.rs.tile_high = self.ppu_bus.read_u8(addr);
        
    }

    fn store_shift_register(&mut self) {
        let mut data: u64 = 0;
        // store data to shift registers
        for i in 0..8 {
            let t1 = (self.rs.tile_low >> (7 - i)) & 0x01;
            let t2 = (self.rs.tile_high >> (7 - i)) & 0x01;
            data <<= 4;
            data |= (self.rs.at_data | t1 | (t2 << 1)) as u64;
        }
        self.rs.tile_data |= data;
    }

    // fetch sprite low byte
    fn fetch_sp_tile_low(&mut self, cache_index: usize) {
        let sprite = self.sprite_cache[cache_index];
        let h = self.sprite_size as u16;
        let mut tile_index: u16;
        let mut row = sprite.row as u16;
        if sprite.flip_v() {
            row = h - row - 1;
        }
        let addr: u16;
        if h == 8 {
            tile_index = sprite.pattern_index() as u16;
            addr = self.sprite_table.wrapping_add((tile_index << 4) | row);
        } else {
            tile_index = (sprite.pattern_index() & 0xfe) as u16;
            let table = sprite.bank_addr();
            if row > 7 {
                tile_index += 1;
                row -= 8;
            }
            addr = table.wrapping_add((tile_index << 4) | row);
        }
        self.rs.tile_low = self.ppu_bus.read_u8(addr);
    }

    // fetch sprite high byte
    fn fetch_sp_tile_high(&mut self, cache_index: usize) {
        let sprite = self.sprite_cache[cache_index];
        let h = self.sprite_size as u16;
        let mut tile_index: u16;
        let mut row = sprite.row as u16;
        if sprite.flip_v() {
            row = h - row - 1;
        }
        let addr: u16;
        if h == 8 {
            tile_index = sprite.pattern_index() as u16;
            addr = self.sprite_table.wrapping_add((tile_index << 4) |  0x08 | row);
        } else {
            tile_index = (sprite.pattern_index() & 0xfe) as u16;
            let table = sprite.bank_addr();
            if row > 7 {
                tile_index += 1;
                row -= 8;
            }
            addr = table.wrapping_add((tile_index << 4) |  0x08 | row);
        }
        self.rs.tile_high = self.ppu_bus.read_u8(addr);
    }

    // store fetch sprite data
    fn store_sprite_data(&mut self, cache_index: usize) {
        let mut sprite = &mut self.sprite_cache[cache_index];
        // store 0 for left over sprites
        if sprite.dummy {
            return;
        }
        let mut data: u32 = 0;
        for i in 0..8 {
            let shift: u8 = match sprite.flip_h() {
                true => i,
                false => 7-i,
            };
            data <<= 4;
            data |= (((self.rs.tile_high >> shift) << 1) & 0x02 | (self.rs.tile_low >> shift) & 0x01 | sprite.pallette_index()) as u32;
        }
        sprite.data = data;
    }

    fn sprite_evaluation(&mut self) {
        // clear secondary_oam oam
        for sp in self.sprite_cache.iter_mut() {
            *sp = FetchedSprite::default();
        }
        // fill it with next line sprite
        let scanline = self.rs.scanline;
        let h = self.sprite_size as u16;
        let mut count: usize = 0;
        for n in 0..64 {
            let sprite = self.oam[n];
            let row = scanline.wrapping_sub(sprite.y() as u16);
            if row >= h {
                continue
            }
            if count < 8 {
                self.sprite_cache[count] = FetchedSprite{
                    data: 0,
                    sprite_index: n as u8,
                    row: row as u8,
                    sprite: sprite,
                    dummy: false,
                }
            }
            count += 1;
            if count > 8 {
                if self.rs.sprite_overflow == false {
                    // println!("overflow at {:?}", self.rs);
                }
               self.rs.sprite_overflow = true;
               break
            }
        }
    }

    fn get_sprite_color(&self) -> (u8, bool, u8) {
        let cycle = self.rs.cycle - 1;
        if self.rs.scanline == 0 {
           return (0, false, 0);
        } 
        for i in 0..8 {
            let color = self.sprite_cache[i].fetch(cycle);
            if color & 0x03 != 0 {
                return (color, self.sprite_cache[i].front(), self.sprite_cache[i].sprite_index);
            }
        }
        (0, false, 0)
    }

    fn get_background_color(&self) -> u8 {
        let bg = ((self.rs.tile_data >> (60 - self.regs.x * 4)) & 0x0f) as u8;
        if bg & 0x03 == 0 {
            0
        } else {
            bg
        }
    }

    // color index to color
    fn get_color(&self, color_index: u8) -> (u8, u8, u8) {
        let offset = (color_index * 4) as usize;
        (self.palette_color[offset], self.palette_color[offset+1], self.palette_color[offset+2])
    }

    // visible and pre-render line logic
    fn fetch_cycle_update(&mut self) {
        let cycle = self.rs.cycle;
        let scanline = self.rs.scanline;
        if self.rs.is_render_enabled() {
            // draw logic
            match (scanline, cycle) {
                (0..=239, 1..=256) => {
                    let bg_palette_index = self.get_background_color();
                    let (sp_palette_index, front_sprite, index) = self.get_sprite_color();
                    let maybe_zero_hit = self.rs.show_background && self.rs.show_sprite && index == 0 && cycle != 256;
                    let color_index = match (bg_palette_index & 0x03, sp_palette_index & 0x03, front_sprite) {
                        (0, 0, _) | (1..=3, 0, _) => {
                            self.ppu_bus.read_u8(0x3f00 + bg_palette_index as u16)
                        },
                        (1..=3, 1..=3, false) => {
                            self.rs.sprite_0_hit = self.rs.sprite_0_hit || maybe_zero_hit;
                            self.ppu_bus.read_u8(0x3f00 + bg_palette_index as u16)
                        }
                        (0, 1..=3, _) => {
                            self.ppu_bus.read_u8(0x3f10 + sp_palette_index as u16)
                        },
                        (1..=3, 1..=3, true) => {
                            self.rs.sprite_0_hit = self.rs.sprite_0_hit || maybe_zero_hit;
                            self.ppu_bus.read_u8(0x3f10 + sp_palette_index as u16)
                        }
                        _ => 0,
                    };
                    let color = self.get_color(color_index);
                    // set output
                    let (x, y) = (cycle as u32 - 1, scanline as u32);
                    let offset: usize = (y * 256 * 3 + x * 3) as usize;
                    self.output[offset + 0] = color.0;
                    self.output[offset + 1] = color.1;
                    self.output[offset + 2] = color.2;
                },
                _ => (),
            }
            // fetch logic
            match (scanline, cycle) {
                (_, 0) => (),
                (0..=239 | 261,  1..=256 | 321..=336) => {
                    // shift to next pixel
                    self.rs.tile_data <<= 4;
                    match cycle % 8 {
                        2 => self.fetch_nt(),
                        4 => { 
                            self.fetch_at();
                            // the actual read finished at cycle 6, but address bus is set at cycle 4
                            // we read here, becuase some maapers depend on the address bus behavior. eg. MMC3 A12
                            self.fetch_bg_tile_low(); 
                        },
                        6 => self.fetch_bg_tile_high(),
                        0 => { 
                            self.store_shift_register(); 
                            self.regs.inc_hori_v();
                        },
                        _ => (),
                    }
                },
                _ => (),
            }
            // sprite
            match (scanline, cycle) {
                (0..=239, 256) => {
                    // sprite_evaluation for the next line
                    self.sprite_evaluation();
                },
                (0..=239 | 261, 257..=320) => {
                    // set oam to 0
                    self.regs.oam_addr = 0;
                    let index = (((cycle - 1) >> 3) & 0x07) as usize;
                    match cycle % 8 {
                        4 => { 
                            self.fetch_at();
                            // the actual read finished at cycle 6, but address bus is set at cycle 4
                            // we read here, becuase some maapers depend on the address bus behavior. eg. MMC3 A12
                            self.fetch_sp_tile_low(index); 
                        },
                        6 => self.fetch_sp_tile_high(index),
                        0 => self.store_sprite_data(index),
                        _ => (),
                    }
                },
                _ => (),
            }
            // pre-line fetch logic
            match (scanline, cycle) {
                (0..=239 | 261, 257) => self.regs.copy_hori_t(),
                (0..=239 | 261, 256) => self.regs.inc_vert_v(),
                (261, 280..=304) => self.regs.copy_vert_t(),
                (261, 339) => {
                    if self.rs.is_odd_frame() && self.rs.show_background {
                        // skip cycle on odd frams
                        self.rs.inc_cycle();
                    }
                },
                _ => (),
            }
        }
    }

    fn vblank_cycle_update(&mut self) {
        match (self.rs.scanline, self.rs.cycle) {
            (241, 1) => {
                self.nmi_occurred = self.rs.nmi_suppress_timer == 0;
                self.rs.nmi_occurred_timer = 2;
            },
            (261, 1) => {
                // println!("pre {:?}", self.rs);
                self.nmi_occurred = false;
                self.rs.sprite_overflow = false;
                self.rs.sprite_0_hit = false;
            },
            _ => (),
        }
    }

    // update nmi status
    fn update_nmi(&mut self) {
        let nmi_current = self.nmi_occurred && self.nmi_enabled && self.rs.nmi_occurred_timer == 0;
        if nmi_current && !self.nmi_prev {
            self.rs.nmi_frame = self.rs.frame_number;
            *self.nmi.borrow_mut() = 1;
        }
        self.nmi_prev = nmi_current;
    }

    // step simulation
    pub fn tick(&mut self) -> u8 {
        self.vblank_cycle_update();
        self.fetch_cycle_update();
        self.update_nmi();
        match self.rs.inc_cycle() {
            true => {
                1
            },
            _ => 0,
        }
    }

    pub fn get_output(&self) -> &[u8] {
        &self.output
    }

}