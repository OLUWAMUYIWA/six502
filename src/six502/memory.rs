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
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.array
    }
}

impl Ram {
    pub(super) fn new() -> Self {
        Self {
            array: [0u8; 0x800],
        }
    }

    pub(super) fn load_u8(&self, addr: u16) -> u8 {
        self.index((addr & 0x7ff) as usize)
    }

    // 6502 arranges integers in little-endian order. lower bytes first
    pub(super) fn load_u16(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.load_u8(addr & 0x7ff), self.load_u8((addr + 1) & 0x7ff)])
    }

    pub fn load_u16_no_carry(&self, addr: u8) -> u16 {
        u16::from_le_bytes([self.load_u8(addr as u16), self.load_u8(addr as u16)])
    }

    pub(super) fn store_u8(&mut self, addr: u16, val: u8) {
        self[(addr & 0x7ff) as usize] = val;
    }

    pub(super) fn store_u16(&mut self, addr: u16, val: u16) {
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
            _ => panic!("invalid load from: {:02x}", addr),
        }
    }

    pub fn load_u16(&mut self, addr: u16) -> u16 {
        u16::from_le_bytes([self.load_u8(addr), self.load_u8(addr + 1)])
    }

    pub fn load_u16_no_carry(&self, addr: u8) -> u16 {
        u16::from_le_bytes([self.load_u8(addr as u16), self.load_u8(addr as u16)])
    }

    pub fn store_u8(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1fff => self.ram.store_u8(addr, val),
            0x2000..=0x3fff => todo!("ppu"),
            0x4016 => todo!("controller"),
            0x4000..=0x4017 => todo!("apu"),
            0x4020..=0xFFFF => todo!("mapper"),
            _ => panic!("invalid store to {:02x}", addr),
        }
    }

    pub fn store_u16(&mut self, addr: u16, val: u16) {
        self.store_u8(addr, val as u8);
        self.store_u8(addr + 1, (val >> 8) as u8);
    }

    pub(super) fn load_u8_bump_pc(&mut self) -> u8 {
        let addr = self.pc;
        self.pc = self.pc.wrapping_add(1);
        self.load_u8(addr)
    }

    pub(super) fn load_u16_bump_pc(&mut self) -> u16 {
        let addr = self.pc;
        self.pc = self.pc.wrapping_add(2);
        self.load_u16(addr)
    }
}

//https://www.nesdev.org/wiki/CPU_memory_map
