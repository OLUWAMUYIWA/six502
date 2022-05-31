use crate::rom::{Rom, Page, Kb};

pub trait Mapper {
    fn load_prg_u8(&mut self, addr: u16) -> Result<u8, Box<dyn std::error::Error>>;
    fn store_prg_u8(&mut self, addr: u16, v: u8);
    fn load_chr_u8(&mut self, addr: u16) -> u8;
    fn store_chr_u8(&mut self, addr: u16, v: u8);
}
/// About the simplest mapper there is; 32K PRG and 8K CHR. Most beginners start with this.
pub(crate) struct NRom {
    pub(crate) data: Rom,
}

impl NRom {
    fn neww(data: Rom) -> Self {
        Self {
            data,
        }
    }
}

impl Mapper for NRom {
    fn load_prg_u8(&mut self, addr: u16) -> Result<u8, Box<dyn std::error::Error>> {
        match addr {
            0x6000..=0x7fff => self.data.prg_rom.load_u8(addr - 0x6000, Page::Zero{size: Kb::Eight}),
            0x8000..=0xbfff => self.data.prg_rom
                .load_u8(addr - 0x8000, Page::Zero{size: Kb::Sixteen}),
            0xc000..=0xffff => self.data.prg_rom.load_u8(addr - 0xc000, Page::Last{size: Kb::Sixteen} ),
            a => panic!("bad address: {:04X}", a),
        }
    }

    fn store_prg_u8(&mut self, addr: u16, v: u8) {
        todo!()
    }

    fn load_chr_u8(&mut self, addr: u16) -> u8 {
        todo!()
    }

    fn store_chr_u8(&mut self, addr: u16, v: u8) {
        todo!()
    }
}