use bitflags::bitflags;
use sdl2::Error;
use std::{
    fs::ReadDir,
    io::{self, Read}, ops::{Deref, DerefMut, Index},
};

use nom::{
    bytes::complete::{tag, take},
    combinator::cond,
    error::{make_error, ErrorKind},
    number::complete::be_u8,
    Err, IResult,
};

/// https://www.nesdev.org/wiki/INES
#[derive(Debug)]
pub struct Rom {
    hdr: Hdr,
    trainer: Option<Vec<u8>>,
    pub(crate) prg_rom: PagedData, // code. (16384 * x bytes)
    pub(crate) chr_rom: PagedData, // (8192 * y bytes) character rom. used by the ppu
}

#[derive(Debug)]
pub struct Hdr {
    pub prg_rom_size: usize, //Size of PRG ROM in 16 KB units, expanded
    pub chr_rom_size: usize, //  Size of CHR ROM in 8 KB units (Value 0 means the board uses CHR RAM), expanded
    pub prg_ram_size: usize,
    pub flags_6: Flags6,
    pub tv_format: TVFormat,
    pub mapper: u8,
}

pub enum Mirroring {
    VERTICAL,
    HORIZONTAL,
}

#[derive(Debug)]
pub enum TVFormat {
    PAL,
    NTSC,
}

// flag_6
// 76543210
// ||||||||
// |||||||+- Mirroring: 0: horizontal (vertical arrangement) (CIRAM A10 = PPU A11)
// |||||||              1: vertical (horizontal arrangement) (CIRAM A10 = PPU A10)
// ||||||+-- 1: Cartridge contains battery-backed PRG RAM ($6000-7FFF) or other persistent memory
// |||||+--- 1: 512-byte trainer at $7000-$71FF (stored before PRG data)
// ||||+---- 1: Ignore mirroring control or above mirroring bit; instead provide four-screen VRAM
// ++++----- Lower nybble of mapper number
bitflags! {
    pub struct Flags6: u8 {
        const V_MIRRORING           = 0b00000001;
        const BATTERY_BACKED_RAM    = 0b00000010;
        const TRAINER_EXISTS        = 0b00000100;
        const FOUR_SCREEN           = 0b00001000;
    }
}

impl Flags6 {
    pub fn mirroring(&self) -> Mirroring {
        if self.contains(Flags6::V_MIRRORING) {
            Mirroring::VERTICAL
        } else {
            Mirroring::HORIZONTAL
        }
    }
}
impl Default for Rom {

    fn default() -> Self {
        todo!()
    }
}
impl Rom {
    pub(crate) fn new() -> Self {
        todo!()
    }

    fn load_hdr(input: &[u8]) -> IResult<&[u8], Hdr> {
        let (input, _) = tag("NES\x1a")(input)?;
        let (input, prog_len) = be_u8(input)?;
        let (input, chr_len) = be_u8(input)?;
        let (input, flag_6) = be_u8(input)?;
        let flags_6 = Flags6::from_bits(0b000001111 & flag_6).ok_or_else(|| Err::Failure(nom::error::Error::new(input, ErrorKind::Fail)))?;

            // .ok_or_else(|| (input, format!("Could not get flags from flag_6")))?;

        let (input, flag_7) = be_u8(input)?;
        if flag_7 & 0x0C == 0x08 {
            return Err(Err::Failure(make_error(input, ErrorKind::Fail)));
        }

        let mapper = flag_7 & 0b11110000 | (flag_6 >> 4);

        let (input, len_ram_banks) = be_u8(input)?;

        let (input, flag_9) = be_u8(input)?;
        let pal = flag_9 & 1;
        let tv_format = if pal == 1 {
            TVFormat::PAL
        } else {
            TVFormat::NTSC
        };

        let (input, trail) = take(6usize)(input)?;
        if b"\x00\x00\x00\x00\x00" != trail {
            return Err(Err::Failure( nom::error::Error::new(input, ErrorKind::Fail)));
        }

        Ok((input, Hdr {
            prg_rom_size: 16384 * prog_len as usize,
            chr_rom_size: 8192 * chr_len as usize,
            flags_6,
            prg_ram_size: 8192 * len_ram_banks as usize,
            tv_format,
            mapper,
        }))
    }

    fn load_body<'a>(hdr: Hdr, input: &'a [u8]) -> IResult<&'a [u8], Rom> {
        let (input, trainer) =
            cond(hdr.flags_6.contains(Flags6::TRAINER_EXISTS), take(512usize))(input)?;
        let (input, prg_rom) = take(16384usize * hdr.prg_rom_size as usize)(input)?;
        let (input, chr_rom) = take(8192usize * hdr.chr_rom_size as usize)(input)?;
        Ok((
            input,
            Rom {
                hdr,
                trainer: trainer.map(|t| t.to_vec()),
                prg_rom: PagedData::new(prg_rom.to_vec()),
                chr_rom: PagedData::new(chr_rom.to_vec()),
            },
        ))
    }

    pub fn load_rom(rdr: &mut impl Read) -> Result<Rom, Box<dyn std::error::Error>> {
        let mut h_buf = [0u8; 16];
        rdr.read_exact(&mut h_buf)?;
        if let IResult::Ok((_, hdr)) = Rom::load_hdr(&h_buf) {
            let mut b_buf = Vec::<u8>::with_capacity(8 * 1024);
            rdr.read_to_end(&mut b_buf)?;
            match Rom::load_body(hdr, &b_buf) {
                IResult::Ok((input, rom)) => Ok(rom),
                IResult::Err(_) => Err("could not load body".into()),
            }
        } else {
            Err(
                format!("Could not load header").into(),
            )
        }
    }

    pub(crate) fn load_u8(&self, addr: u16) -> u8 {
        todo!()
    }

    pub(crate) fn store_u8(&self, addr: u16) {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct PagedData {
    v: Vec<u8>,
}

impl Deref for PagedData {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.v
    }
    
}

impl DerefMut for PagedData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.v
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Kb {
    One = 0x400,
    Four = 0x1000,
    Eight = 0x2000,
    Sixteen = 0x4000,
}

pub(crate) enum Page {
    Zero{size: Kb},
    Last{size: Kb},
    Nth{n: usize, size: Kb},
    RNth{n: usize, size: Kb},
}

impl PagedData {
    pub(crate) fn new(v: Vec<u8>) -> Self {
        Self {
            v,
        }
    }

    pub(crate) fn load_u8(&self, pos: u16, page: Page) -> Result<u8, Box<dyn std::error::Error>> {
        let index = self.nth(pos, page)?;
        Ok(self[index])
    }

    pub(crate) fn store_u8(&mut self, pos: u16, page: Page, v: u8) -> Result<(), Box<dyn std::error::Error>> {
        let index = self.nth(pos, page)?;
        self[index] = v;
        Ok(())
    }    
    fn nth(&self, pos: u16, page: Page) -> Result<usize, Box<dyn std::error::Error>> {
        
        match page {
            Page::Zero { size } => {
                self.nth(pos, Page::Nth{n: 0, size} )
            },
            Page::Last { size } => {
                let num_pages =  self.v.len() / (size as usize);
                self.nth(pos, Page::Nth{n: num_pages-1, size} )
            },
            Page::Nth { n, size } => {
                if self.v.len() % size as usize != 0 {
                    return Err(format!("paged data size ought to be a multiple of size but isn't. ").into());
                }
                let num_pages =  self.v.len() / (size as usize); // number of pages in paged data
                if n > (num_pages - 1) as usize {
                    return Err("page out of bounds".into());
                }
                if pos as usize > (size) as usize { // if pos exceeds the size of the page its out of bounds
                    return  Err("pos out of bounds".into());
                }
                Ok((n * size as usize ) + pos as usize)
            },
            Page::RNth { n, size } => {
                let num_pages =  self.v.len() / (size as usize);
                self.nth(pos, Page::Nth{n: num_pages-1-n, size}, )
            },
        }
    }
}

