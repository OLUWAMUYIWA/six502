#![allow(unused_imports, dead_code)]
mod bus;
mod macros;
mod six502;

use bus::ByteAccess;
pub use six502::addr_mode::AddressingMode;

use six502::Op;
pub trait Cpu: ByteAccess  + Addressing  {
    
    fn new() -> Self;

    fn load_u8_bump_pc(&mut self) -> u8;

    fn load_u16_bump_pc(&mut self) -> u16;

    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    fn fetch_op(&mut self, op: &mut Op) ;

    fn exec(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    fn reset(&mut self);
}

pub trait Addressing {
    
    fn dispatch_load(&mut self) -> u8;
    fn dispatch_store(&mut self);
}


