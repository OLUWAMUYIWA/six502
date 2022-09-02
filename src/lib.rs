#![allow(unused_imports, dead_code)]
mod bus;
mod macros;
mod six502;

use bus::ByteAccess;
pub use six502::addr_mode::AddressingMode;
pub use six502::Six502;


pub trait Cpu : ByteAccess{

}


impl Cpu for Six502{}