use std::io::{self, Read};

// https://www.nesdev.org/wiki/INES
pub struct Rom {
    hdr: Hdr,
    trainer: Option<[u8; 512]>,
    prg_rom: Vec<u8>, // code. (16384 * x bytes)
    chr_rom: Vec<u8>, // (8192 * y bytes) character rom. used by the ppu
}

pub struct Hdr {
    pub prg_rom_size: u8, //Size of PRG ROM in 16 KB units
    pub chr_rom_size: u8, //  Size of CHR ROM in 8 KB units (Value 0 means the board uses CHR RAM)
    pub flags_6: u8,      // Mapper, mirroring, battery, trainer
    pub flags_7: u8,      //  Mapper, VS/Playchoice, NES 2.0
    pub prg_ram_size: u8, // flag_8
    pub flags_9: u8,      //
    pub flags_10: u8,
}

impl Rom {
    pub fn load<R: Read>(rdr: R) -> Result<Rom, Box<dyn std::error::Error>> {
        let mut buf = [0u8; 16];
        rdr.read_exact(&mut buf)?;
        let ref constant: [u8; 4] = b"NES\x1a";
        if constant != &buf[..4] {
            return Err(format!("Expected: NES\x1a"));
        }

        let hdr = Hdr {
            prg_rom_size: buf[4],
            chr_rom_size: buf[5],
            flags_6: buf[6],
            flags_7: buf[7],
            prg_ram_size: buf[8],
            flags_9: buf[9],
            flags_10: buf[10],
        };

        if b"\x00\x00\x00\x00\x00" != &buf[11..16] {
            return Err(format!("Expected 5 null bytes"));
        }

        let prg_len = hdr.prg_rom_size as usize * 16384;
        let mut prg_rom = vec![0u8; prg_len];
        rdr.read_exact(&mut prg_rom)?;
        let chr_len = hdr.chr_rom_size as usize * 8192;
        let mut chr_rom = vec![0u8; chr_len];
        rdr.read_exact(&mut chr_rom)?;

        Ok(Rom {
            hdr,
            trainer: None,
            prg_rom,
            chr_rom,
        })
    }
}
