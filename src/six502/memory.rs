use core::panic;
use std::ops::{Deref, DerefMut, Index};

pub struct Ram {
    array: [u8; 0x800],
}

impl Deref for Ram {
    type Target = [u8; 0x800];

    fn deref(&self) -> &Self::Target {
        &self.array
    }
}

impl DerefMut for Ram {
    type Target = [u8; 0x800];

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.array
    }
}

impl Ram {
    pub(crate) fn new() -> Self {
        Self {
            array: [0u8; 0x800],
        }
    }

    pub(crate) fn load_u8(&self, addr: u16) -> u8 {
        *self.index((addr & 0x7ff) as usize)
    }

    pub(crate) fn load_u16(&self, addr: u16) -> u16 {
        self.load_byte(addr & 0x7ff) as u16 | ((self.load_byte((addr + 1) & 0x7ff) as u16) << 8)
    }

    pub(crate) fn store_u8(&mut self, addr: u16, val: u8) {
        self[(addr & 0x7ff) as usize] = val;
    }

    pub(crate) fn store_u16(&mut self, addr: u16, val: u16) {
        self.store_u8(addr, val as u8); //comeback. does `as` truncate the MSB as expected?
        self.store_u8(addr + 1, (val >> 8) as u8);
    }
}

use crate::six502::Six502;

impl Six502 {
    pub fn load_u8(&mut self, addr: u16) -> u8 {
        match addr {
            0x000..=0x1fff => self.ram.load_u8(addr),
            0x2000..=0x3fff => todo!("ppu"),
            0x4015 => todo!("apu"),
            0x4016 => todo!("controller"),
            0x4018 => todo!("apu"),
            0x4020..=0xffff => todo!("mapper"),
            _ => panic!("inalid load from: {:02x}", addr),
        }
    }

    pub fn store_u8(&mut self, address: u16, val: u8) {
        match addr {
            0x0000..=0x1fff => self.ram.store_u8(addr, val),
            0x2000..=0x3fff => todo!(ppu),
            0x4016 => todo!("controller"),
            0x4000..=0x4017 => todo!("apu"),
            0x4020..=0xFFFF => todo!("mapper"),
            _ => panic!("invalid store to {:02x}", addr),
        }
    }
}
