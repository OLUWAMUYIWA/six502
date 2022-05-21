use super::Six502;
use crate::bus::{ByteAccess, WordAccess};
use core::panic;
use std::ops::{Deref, DerefMut, Index};

impl ByteAccess for Six502 {
    fn load_u8(&self, addr: u16) -> u8 {
        self.bus.load_u8(addr)
    }

    fn store_u8(&mut self, addr: u16, v: u8) {
        self.bus.store_u8(addr, v);
    }
}

impl Six502 {
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
    pub(crate) fn new() -> Self {
        Self {
            array: [0u8; 0x800],
        }
    }
}

impl ByteAccess for Ram {
    // first 8192 bytes are for the ram. the ram is 2048 consecutive bytes mirrored three other times, consecutively
    fn load_u8(&self, addr: u16) -> u8 {
        self[(addr & 0x7ff) as usize]
    }

    fn store_u8(&mut self, addr: u16, val: u8) {
        self[(addr & 0x7ff) as usize] = val;
    }
}
