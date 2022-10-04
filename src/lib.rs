#![allow(unused_imports, dead_code)]
mod bus;
mod macros;
mod six502;

pub use six502::addressing::AddressingMode;

use six502::Op;
pub trait Cpu: ByteAccess {
    fn new() -> Self;

    fn load_u8_bump_pc(&mut self) -> u8;

    fn load_u16_bump_pc(&mut self) -> u16;

    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    fn fetch_op(&mut self);

    fn decode_op(&mut self, op: &mut Op);

    fn exec(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    fn reset(&mut self);
}
/// ByteAccess handles the loading and storage of u8 values. An implementor is an addressable member of the NES
/// The memory address can be regarded as 256 pages (each page defined by the high order byte) of 256 memory locations (bytes) per page.
pub trait ByteAccess {
    fn load_u8(&mut self) -> u8;
    fn store_u8(&mut self, v: u8);
    // this allows easy impl of WordAccess since were going to need to bump the address by one 
    fn bump(&mut self);
}

pub trait Addressing {
    fn dispatch_load(&mut self, mode: AddressingMode) -> u8;
    fn dispatch_store(&mut self, v: u8, mode: AddressingMode);
}



#[cfg(test)]
#[macro_use]
extern crate parameterized;