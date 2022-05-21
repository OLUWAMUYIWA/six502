use super::six502::{interrupt::Interrupt, memory::Ram};
use crate::{apu::Apu, ctrl::Joypad, ppu::Ppu, rom::Rom};

//https://www.nesdev.org/wiki/CPU_memory_map

pub trait ByteAccess {
    fn load_u8(&self, addr: u16) -> u8;
    fn store_u8(&mut self, addr: u16, v: u8);
}

pub trait WordAccess {
    fn load_u16(&self, addr: u16) -> u16;
    // loads the word from a u8 address. no carries to the other page
    fn load_u16_no_carry(&self, addr: u8) -> u16;
    fn store_u16(&mut self, addr: u16, v: u16);
}

// blanket implementation of Word Access for every item that implements `ByteAccess`
impl<T: ByteAccess> WordAccess for T {
    // 6502 arranges integers in little-endian order. lower bytes first
    fn load_u16(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.load_u8(addr), self.load_u8(addr + 1)])
    }

    // unlike `load_u16`, `load_u16_no_carry` is used by te `Indexed Indirect` and the `Indirect Indexed` addressing modes
    fn load_u16_no_carry(&self, addr: u8) -> u16 {
        u16::from_le_bytes([self.load_u8(addr as u16), self.load_u8(addr as u16)])
    }

    fn store_u16(&mut self, addr: u16, v: u16) {
        self.store_u8(addr, v as u8);
        self.store_u8(addr + 1, (v >> 8) as u8);
    }
}

pub struct Bus {
    pub ram: Ram,
    pub rom: Rom,
    pub apu: Apu,
    pub ppu: Ppu,
    pub joypad_1: Joypad,
    pub joypad_2: Joypad,
    pub interrupt: Interrupt,
    pub cycles: u64,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl ByteAccess for Bus {
    fn load_u8(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram.load_u8(addr),
            0x2000..=0x3FFF => self.ppu.load_u8(addr), //comeback to check bounds
            0x4015 => self.apu.load_u8(addr),
            0x4016 => self.joypad_1.load_u8(),
            0x4017 => self.joypad_2.load_u8(),
            0x4018..=0xFFFF => self.rom.load_u8(addr),
            addr => panic!("Address {} not addressable", addr),
        }
    }

    fn store_u8(&mut self, addr: u16, v: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram.store_u8(addr, v),
            0x2000..=0x3FFF => self.ppu.store_u8(addr, v),
            0x4015 => self.apu.store_u8(addr),
            0x4016 => self.joypad_1.store_u8(v),
            0x4017 => self.joypad_2.store_u8(v),
            0x4018..=0xFFFF => self.rom.store_u8(addr),
            addr => panic!("Address {} not addressable", addr),
        }
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self {
            ram: Ram::new(),
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
