#![allow(dead_code, unused_variables, unused_imports)]

use std::ops::{AddAssign, Index};

pub struct Six502 {
    reg_a: u8,
    reg_x: u8,
    pc: u16,
    flags: u8,
    memory: [u8; 0xFF],
}

impl Six502 {
    pub(crate) fn new() -> Self {
        Self {
            reg_a: 0,
            reg_x: 0,
            pc: 0,
            flags: 0,
            memory: [0u8; 0xFF],
        }
    }
    pub fn run(&mut self) {}

    fn read_mem(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn write_mem(&mut self, addr: u16, v: u8) {
        self.memory[addr as usize] = v;
    }
    fn load(&mut self, prog: &[u8]) {
        //comeback
        //assert that the program is not longer than accepted memory space
        assert!(0x8000 + prog.len() < 0xff);

        self.pc = 0x8000;
        self.memory[0x8000..(0x8000 + prog.len())].copy_from_slice(prog);
    }

    fn update_flags(&mut self, v: u8) {
        if self.reg_x == 0 {
            self.flags.add_assign(0b0000_0010);
        } else {
            self.flags.add_assign(0b1111_1101);
        };

        if self.reg_a & 0b1000_0000 != 0 {
            // MSB is set
            self.flags.add_assign(0b1000_0000);
        } else {
            self.flags.add_assign(0b0111_1111);
        };
    }

    fn interpret(&mut self, prog: Vec<u8>) {
        loop {
            //the opcode comes before the args
            let opcode = prog.index(self.pc as usize).to_owned();
            self.pc += 1;

            match opcode {
                0xA9 => {
                    let to_load = prog.index(self.pc as usize).to_owned();
                    self.pc += 1;
                    self.reg_a = to_load;

                    //change the flags bits
                    self.update_flags(to_load);
                }

                0xAA => {
                    self.reg_x = self.reg_a;
                    self.update_flags(self.reg_x);
                }

                0x00 => {
                    return;
                }

                _ => todo!(),
            }
        }
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_0xa9() {
        let mut cpu = Six502::new();
        cpu.interpret(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.reg_a, 0xA9);
        assert_eq!(cpu.flags & 0b0000_0010, 0);
        assert_eq!(cpu.flags & 0b1000_0000, 0);

        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0x00]);
        assert_eq!(cpu.flags & 0b0000_0010, 0b0000_0010);
    }
    #[test]
    fn test_0xaa() {
        let mut cpu = Six502::new();
        cpu.interpret(vec![0xaa, 0x00]);
        assert_eq!(cpu.reg_x, cpu.reg_a);
    }

    #[test]
    fn test_0xa9_0xaa() {
        let mut cpu = Six502::new();
        cpu.interpret(vec![0xa9, 0x05, 0xAA, ox00]);
        assert_eq!(cpu.reg_x, 0x05);
    }

    fn test_ops() {
        let mut cpu = Six502::new();
        let prog = vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00];
        assert_eq!(cpu.reg_x, 0xc1);
    }
}
