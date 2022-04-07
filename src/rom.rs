use std::io::{self, Read};

pub struct Rom {
    program: Vec<u8>,
}

impl Rom {
    pub fn load<R: Read>(rdr: R) -> Result<(), io::Error> {
        let mut n = 0;
    }
}

pub struct Hdr {
    pub magic: [u8; 4],
    pub prog_size: u8,
    pub prog_ram_size: u8,
    pub chr_rom_size: u8,
    pub flags_6: u8,
    pub flags_7: u8,
    pub flags_9: u8,
    pub flags_10: u8,
    pub zero: [u8; 5],
}
