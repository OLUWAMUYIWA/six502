use super::Six502;
use crate::{bus::{ByteAccess, WordAccess}, macros::impl_deref_mut};
use core::panic;
use std::ops::{Deref, DerefMut, Index};

#[derive(Debug)]
pub struct Ram {
    array: [u8; 0x800],
}

impl_deref_mut!(Ram {array, [u8]});


impl Default for Ram {

    fn default() -> Self {
        todo!()
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
    fn load_u8(&mut self, addr: u16) -> u8 {
        self[(addr & 0x7ff) as usize]
    }

    fn store_u8(&mut self, addr: u16, val: u8) {
        self[(addr & 0x7ff) as usize] = val;
    }
}
