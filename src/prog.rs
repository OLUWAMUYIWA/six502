use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::Path,
};

#[derive(Debug)]
pub(crate) struct Program {
    zp: [u8; 0x100],
    stack: [u8; 0x100],
    data: Vec<u8>, // 65018
    // At the high end of memory, the last six bytes of the last page (page 255) of
    // memory are used by the hardware to contain special addresses.
    //https://people.cs.umass.edu/~verts/cmpsci201/spr_2004/Lecture_02_2004-01-30_The_6502_processor.pdf
    // IRQ, NMI, RESET. each two bytes each
    special: [u8; 0x06],
}
const MEM_SIZE: usize = 1024 * 64;
const MAX_PROG: usize = 65018;

impl Program {
    pub fn open(path: impl Into<Path>) -> Result<Self, Box<dyn Error>> {
        let b = fs::read(path)?;
        if b.len() > MAX_PROG {
            return io::Error::new(io::ErrorKind::InvalidData, "Program larger than 652 allows");
        };

        Ok(Self {
            zp: [0u8; 0x100],
            stack: [0u8; 0x100],
            data: b,
            special: [0u8; 6],
        })
    }

    pub fn dump(&self, path: impl Into<Path>) -> Result<(), Box<dyn Error>> {
        let mut f = OpenOptions::new().write(true).create(true).open(path)?;
        f.write_all(&self.zp)?;
        f.write_all(&self.stack)?;
        f.write_all(&self.data)?;
        f.write_all(&self.special)?;
        Ok(())
    }
}
