use crate::{apu::Apu, ctrl::Joypad, ppu::Ppu, rom::Rom};

use super::six502::interrupt::Interrupt;

pub struct Bus {
    pub ram: [u8; 0x800],
    pub rom: Rom,
    pub apu: Apu,
    pub ppu: Ppu,
    pub joypad_1: Joypad,
    pub joypad_2: Joypad,
    pub rom: Rom,
    pub interrupt: Interrupt,
    pub cycles: u64,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub(crate) fn load_u8(&self, addr: u16) -> u8 {
        match addr {
            0...0x1FFF => self.index((addr & 0x7ff) as usize), // first 8192 bytes are the ram. the ram is 2048 consecutive bytes mirrored three other times, consecutively
            0x2000...0x3FFF => self.ppu.load_u8(addr),
            0x4015 => self.apu.load_u8(addr),
            0x4016 => self.joypad_1.load_u8(addr),
            0x4017 => self.joypad_2.load_u8(addr),
            0x4018...0xFFFF => self.rom.load_u8(addr),
            addr => panic!(format!("Address {} not addressable", addr)),
        }
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self {
            ram: [0; 256],
            rom: Rom::new(),
            apu: Apu::new(),
            ppu: Ppu::new(),
            joypad_1: Joypad::new(),
            joypad_2: Joypad::new(),
            interrupt: Interrupt::new(),
            cycles: todo!(),
        }
    }
}
