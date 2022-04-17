mod addressing;
mod memory;
mod opcodes;
mod util;

const STACK_OFFSET: usize = 0x100;
const NMI_VECTOR: usize = 0xfffa;
const RESET_VECTOR: usize = 0xfffc;
const IRQ_VECTOR: usize = 0xfffe;

bitflags::bitflags! {
    pub struct Flags: u8 {
        const CARRY = 0b00000001;
        const ZERO = 0b00000010; //set to 1 on equality
        const IRQ = 0b00000100;
        const DECIMAL= 0b00001000;
        const BREAK =    0b00010000;
        //unused bit pos
        const OVERFLOW = 0b01000000;
        const NEGATIVE= 0b10000000;
    }
}
mod flags {
    const CARRY: U8 = 1 << 0;
    const ZERO: u8 = 1 << 1; //set to 1 on equality
    const IRQ: u8 = 1 << 2;
    const DECIMAL: u8 = 1 << 3;
    const BREAK: u8 = 1 << 4;
    //unused bit pos
    const OVERFLOW: u8 = 1 << 6;
    const NEGATIVE: u8 = 1 << 7;
}

//comeback: where is jmpi

pub struct Six502 {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    s: u8,
    flags: u8,
    pub ram: Ram,
}

impl Six502 {
    pub(crate) fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0xc000,
            s: 0xfd,
            flags: 0x24,
            ram: Ram::new(),
        }
    }
    pub fn step(&mut self) -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }
}

lazy_static::lazy_static! {
    static ref CYCLES: [U8; 256] = [
        //       0, 1, 2, 3, 4, 5, 6, 7, 8, 9, A, B, C, D, E, F
        /*0x00*/ 7, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6,
        /*0x10*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
        /*0x20*/ 6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6,
        /*0x30*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
        /*0x40*/ 6, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6,
        /*0x50*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
        /*0x60*/ 6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6,
        /*0x70*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
        /*0x80*/ 2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
        /*0x90*/ 2, 6, 2, 6, 4, 4, 4, 4, 2, 5, 2, 5, 5, 5, 5, 5,
        /*0xA0*/ 2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
        /*0xB0*/ 2, 5, 2, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4,
        /*0xC0*/ 2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
        /*0xD0*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
        /*0xE0*/ 2, 6, 3, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
        /*0xF0*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    ];
}
