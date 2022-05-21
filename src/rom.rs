use bitflags::bitflags;
use std::{
    fs::ReadDir,
    io::{self, Read},
};

use nom::{
    bytes::complete::{tag, take},
    combinator::cond,
    error::{make_error, ErrorKind},
    number::complete::be_u8,
    Err, IResult,
};

// https://www.nesdev.org/wiki/INES
pub struct Rom {
    hdr: Hdr,
    trainer: Option<Vec<u8>>,
    prg_rom: Vec<u8>, // code. (16384 * x bytes)
    chr_rom: Vec<u8>, // (8192 * y bytes) character rom. used by the ppu
}

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
        const FOUR_SCREEN        = 0b00001000;
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

impl Rom {
    pub(crate) fn new() -> Self {
        todo!()
    }

    fn load_hdr(input: &[u8]) -> IResult<&[u8], Rom> {
        let (input, _) = tag(b"NES\x1a".into())(input)?;
        let (input, prog_len) = be_u8(input)?;
        let (input, chr_len) = be_u8(input)?;
        let (input, flag_6) = be_u8(input)?;
        let flags_6 = Flags6::from_bits(0b000001111 & flag_6)
            .ok_or(format!("Could not get flags from flag_6"))?;

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

        let (input, trail) = take(6)(input)?;
        if b"\x00\x00\x00\x00\x00" != trail {
            return Err(Err::Failure((input, ErrorKind::Fail)));
        }

        Ok(Hdr {
            prg_rom_size: 16384 * prog_len as usize,
            chr_rom_size: 8192 * chr_len as usize,
            flags_6,
            prg_ram_size: 8192 * len_ram_banks as usize,
            tv_format,
            mapper,
        })
    }

    fn load_body<'a>(hdr: &Hdr, input: &'a [u8]) -> IResult<&'a [u8], Rom> {
        let (input, trainer) =
            cond(hdr.flags_6.contains(Flags6::TRAINER_EXISTS), take(512))(input)?;
        let (input, prg_rom) = take(16384 * hdr.prog_len as usize)(input)?;
        let (input, chr_rom) = take(8192 * hdr.chr_len as usize)(input)?;
        Ok((
            input,
            Rom {
                hdr,
                trainer: trainer.map(|t| t.to_vec()),
                prg_rom: prg_rom.to_vec(),
                chr_rom: chr_rom.to_vec(),
            },
        ))
    }

    pub fn load_rom(rdr: impl Read) -> Result<Rom, Box<dyn std::error::Error>> {
        let mut h_buf = [0u8; 16];
        rdr.read_exact(&mut h_buf)?;
        if let IResult::Ok((input, hdr)) = Rom::load_hdr(&h_buf) {
            let mut b_buf = Vec::<u8>::with_capacity();
            rdr.read_to_end(&mut b_buf)?;
            match Rom::load_body(&hdr, &b_buf) {
                IResult::Ok((input, rom)) => Ok(rom),
                IResult::Err(err) => Err(Box::new(err)),
            }
        } else {
            Err(Box::<&'static str>::new(
                format!("Could not load header").into(),
            ))
        }
    }

    pub(crate) fn load_u8(&self, addr: u16) -> u8 {
        todo!()
    }

    pub(crate) fn store_u8(&self, addr: u16) {
        todo!()
    }
}
