use super::six502::{interrupt::Interrupt, ram::Ram};
use crate::{apu::Apu, ctrl::Joypad, ppu::Ppu, rom::Rom};

//https://www.nesdev.org/wiki/CPU_memory_map

/// ByteAccess handles the loading and storage of u8 values. An implementor is an addressable member of the NES
/// The memory address can be regarded as 256 pages (each page defined by the high order byte) of 256 memory locations (bytes) per page.
pub trait ByteAccess {
    fn load_u8(&mut self, addr: u16) -> u8;
    fn store_u8(&mut self, addr: u16, v: u8);
}

pub trait WordAccess {
    fn load_u16(&mut self, addr: u16) -> u16;
    fn store_u16(&mut self, addr: u16, v: u16);
}

// blanket implementation of Word Access for every item that implements `ByteAccess`
impl<T: ByteAccess> WordAccess for T {
    // 6502 arranges integers in little-endian order. lower bytes first
    fn load_u16(&mut self, addr: u16) -> u16 {
        u16::from_le_bytes([self.load_u8(addr), self.load_u8(addr + 1)])
    }

    fn store_u16(&mut self, addr: u16, v: u16) {
        self.store_u8(addr, v as u8);
        self.store_u8(addr + 1, (v >> 8) as u8);
    }
}

/// The DataBus
/// data has to transfer between the accumulator and the internal registers of the microprocessor and outside sources by means of passing through the microprocessor to 8 lines
/// called the data bus. The outside sources include (in our case) the program which controls the microprocessor, and the actual communications to the world through input/output
/// ports.
///! The duty of the data bus is to facilitate exchange of data between memory and the processor's internal registers.
#[derive(Debug)]
pub struct DataBus {
    pub ram: Ram,
    pub rom: Rom,
    pub(crate) apu: Apu,
    pub(crate) ppu: Ppu,
    pub joypad_1: Joypad,
    pub joypad_2: Joypad,
    pub(crate) interrupt: Interrupt,
    pub cycles: u64,
}

impl DataBus {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}



// [memory map spec](https://8bitworkshop.com/blog/platforms/nes/)
// | ---------------------------------------------------------
// | Start  |  End    |  Description                         |
// | ---------------------------------------------------------
// | $0000  |  $00FF  |  RAM, zero-page                      |
// | $0100  |  $01FF  |  RAM, CPU stack                      |
// | $0200  |  $07FF  |  RAM, general-purpose                |
// | $0800  |  $1FFF  |  (mirror of $0000-$07FF)             |
// | $2000  |  $2007  |  PPU registers                       |
// | $2008  |  $3FFF  |  (mirror of $2000-$2007)             |
// | $4000  |  $400F  |  APU registers                       |
// | $4010  |  $4017  |  DMC, joystick, APU registers        |
// | $4020  |  $5FFF  |  Cartridge (maybe mapper registers)  |
// | $6000  |  $7FFF  |  Cartridge RAM (maybe battery-backed)|
// | $8000  |  $FFFF  |  PRG ROM (maybe bank switched)       |
// | $FFFA  |  $FFFB  |  NMI vector                          |
// | $FFFC  |  $FFFD  |  Reset vector                        |
// | $FFFE  |  $FFFF  |  BRK vector                          |
// | ---------------------------------------------------------

impl ByteAccess for DataBus {
    fn load_u8(&mut self, addr: u16) -> u8 {
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

impl Default for DataBus {
    fn default() -> Self {
        Self { 
            ram: Default::default(), 
            rom: Default::default(), 
            apu: Default::default(), 
            ppu: Default::default(), 
            joypad_1: Default::default(), 
            joypad_2: Default::default(), 
            interrupt: Default::default(), 
            cycles: Default::default() }
    }
}

pub struct AddressBus {}
