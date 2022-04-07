use std::ops::{Deref, DerefMut, Index};

pub struct Ram {
    cont: [u8; 0x800],
}

impl Deref for Ram {
    type Target = [u8; 0x800];

    fn deref(&self) -> &Self::Target {
        &self.cont
    }
}

impl DerefMut for Ram {
    type Target = [u8; 0x800];

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cont
    }
}

impl Ram {
    fn load_byte(&self, addr: u16) -> u8 {
        *self.index(addr as usize)
    }

    fn load_word(&self, addr: u16) -> u16 {
        self.load_byte(addr) as u16 | ((self.load_byte(addr + 1) as u16) << 8)
    }

    fn save_byte(&mut self, addr: u16, val: u8) {
        self[addr as usize] = val;
    }

    fn save_word(&mut self, addr: u16, val: u16) {
        self.save_byte(addr, (val & 0xff))
    }
}
