use crate::{macros::impl_deref_mut};

use super::six502::ram::Ram;
use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    ops::{Deref, DerefMut},
    path::Path,
};

pub trait BusAccess {
    fn load_u8(&mut self, addr: u16) -> u8 ;
    fn store_u8(&mut self, addr: u16, v: u8);
}


#[derive(Debug)]
pub(crate) struct Mem {
    zp: [u8; 0x100],
    stack: [u8; 0x100],
    x: Vec<u8>, // 65018 max. unreserved. contaains program and unused
    // At the high end of memory, the last six bytes of the last page (page 255) of
    // memory are used by the hardware to contain special addresses.
    //https://people.cs.umass.edu/~verts/cmpsci201/spr_2004/Lecture_02_2004-01-30_The_6502_processor.pdf
    // IRQ, NMI, RESET. each two bytes each
    special: [u8; 0x06],
}
const MEM_SIZE: usize = 1024 * 64;
const MAX_PROG: usize = 65018;

impl Default for Mem {
    fn default() -> Self {
        Self {
            zp: [0u8; 256],
            stack: [0u8; 256],
            x: Default::default(),
            special: Default::default(),
        }
    }
}

impl Mem {
    pub fn open<T: AsRef<Path>>(path: T) -> Result<Self, Box<dyn Error>> {
        let b = fs::read(path)?;
        if b.len() > MAX_PROG {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                "Program larger than 652 allows",
            )));
        };

        Ok(Self {
            zp: [0u8; 0x100],
            stack: [0u8; 0x100],
            x: b,
            special: [0u8; 6],
        })
    }

    pub(super) fn load_zp(&self, addr: u16) -> u8 {
        self.zp[addr as usize]
    }

    pub(super) fn load_stack(&self, addr: u16) -> u8 {
        self.stack[addr as usize]
    }

    pub(super) fn store_zp(&mut self, addr: u16, v: u8) {
        self.zp[addr as usize] = v;
    }

    pub(super) fn store_stack(&mut self, addr: u16, v: u8) {
        self.zp[addr as usize] = v;
    }

    pub(crate) fn store_x(&mut self, addr: u16, v: u8) {
        self.x[((addr - 0xFFFA) as usize)] = v; // offset into the 6-bye array
    }

    pub(crate) fn load_x(&mut self, addr: u16) -> u8 {
        self.x[((addr - 0xFFFA) as usize)]
    }

    pub fn dump<T: AsRef<Path>>(&self, path: T) -> Result<(), Box<dyn Error>> {
        let mut f = OpenOptions::new().write(true).create(true).open(path)?;
        f.write_all(&self.zp)?;
        f.write_all(&self.stack)?;
        f.write_all(&self.x)?;
        f.write_all(&self.special)?;
        Ok(())
    }
}

/// The DataBus
/// data has to transfer between the accumulator and the internal registers of the microprocessor and outside sources by means of passing through
///  the microprocessor to 8 lines called the data bus. The outside sources include (in our case) the program
/// which controls the microprocessor, and the actual communications to the world through input/output ports.
///! The duty of the data bus is to facilitate exchange of data between memory and the processor's internal registers.
/// I/o operationS on this type of microprocessor are accomplished by reading and writing registers which
/// actually represent connections to physical devices or to physical pins  which connect to physical devices.
#[derive(Debug, Default)]
#[repr(transparent)]
pub(crate) struct DataBus {
    pub(crate) mem: Mem,
}

impl_deref_mut!(DataBus { mem, Mem });

impl DataBus {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    // comeback
    pub fn set(&mut self, v: u8) {
        todo!()
    }
}

impl BusAccess for DataBus {
    fn load_u8(&mut self, addr: u16) -> u8 {
        match addr {
            a @ 0x0000..=0x00FF => self.load_zp(a),
            0x0100..=0x01ff => self.load_stack(addr),
            // 0x0000..=0x1FFF => self.ram.load_u8(addr),
            addr => panic!("Address {} not addressable", addr),
        }
    }

    fn store_u8(&mut self, addr: u16, v: u8) {
        match addr {
            a @ 0x0000..=0x00ff => self.store_zp(a, v),
            a @ 0x0100..=0x01ff => self.store_stack(a, v),
            // 0x0000..=0x1FFF => self.ram.store_u8(addr, v),
            addr => panic!("Address {} not addressable", addr),
        }
    }
}

