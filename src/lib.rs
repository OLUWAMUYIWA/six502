#![allow(unused_imports, dead_code)]
mod bus;
mod macros;
mod six502;

use bus::ByteAccess;
pub use six502::addr_mode::AddressingMode;

use six502::Op;
pub trait Cpu: ByteAccess  {
    fn new() -> Self;

    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    fn fetch_op(&mut self, op: &mut Op) ;

    fn exec(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    fn reset(&mut self);

}


