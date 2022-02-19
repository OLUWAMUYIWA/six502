#![allow(dead_code, unused_variables, unused_imports)]

use std::ops::{AddAssign, Index, RangeBounds};

//comeback: where is jmpi


pub struct Six502 {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    flags: u8,
    mem: [u8; 0xFF],
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Absolute,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Immediate,
    Relative,
    Implicit,
    Indirect,
    IndexedIndirect,
    IndirectINdexed,
}


// Source: https://web.archive.org/web/20210428044647/http://www.obelisk.me.uk/6502/reference.html
pub enum OpCode {
    ADC, // add with carry
    AND, // logical and
    ASL, // Arithmetic shift left
    BCC = 0x90, // bramch if carry c;ear
    BCS = 0xb0, // branch if carry set
    BEQ = 0xf0, // branch if equla
    BIT, // bit test
    BMI = 0x30, // branch if minus
    BNE = 0xd0, // branch if not equal
    BPL = 0x10, // branch if positive
    BRK = 0x00, // force interrupt
    BVC = 0x50, // branch if overflow clear
    BVS = 0x70, // branch if overflow set
    CLC = 0x18, // clear carry flag
    CLD = 0xd8, // clear decimal node
    CLI = 0x58, // clear interrupt disable
    CLV = 0xb8, // clear overflow flag
    CMP, // compare
    CPX, // compare x register
    CPY, // cmpare y register
    DEC, // decrement memory
    DEX = 0xca, // decrement x register
    DEY = 0x88, // decrement y register
    EOR, // exclusive or
    INC, // increment memory
    INX = 0xe8, // increment x register
    INY = 0xc8, // increment y register
    JMP = 0x4c, // jump
    JSR = 0x20, // jump to subroutine
    LDA, // load accumulator
    LDX, // load x register
    LDY, // load y register
    LSR, // logical shift right
    NOP = 0xEA, // no-op
    ORA, // logical inclusive or
    PHA = 0x48, // push accumulator
    PHP = 0x08, // push processor status
    PLA = 0x68, // pull accumulator
    PLP = 0x28, // pull processor status
    ROL, // rotate left
    ROR, // rotate right
    RTI = 0x40, // return from interrupt
    RTS = 0x60, // return from subroutine
    SBC, // subtract with carry
    SEC = 0x38, // set carry flag
    SED = 0xf8, // set decimal flag
    SEI = 0x78, // set interrupt disable
    STA, // store accumulator
    STX, // store x register
    STY, // store y register
    TAX = 0xaa, // transfer accumulator to x
    TAY = 0xa8, // transfer accumulator to y
    TSX = 0xba, // transfer stack pointer to x
    TXA = 0x8a, // transfer x to accumulator
    TXS = 0x9a, // transfer x to stack pointer
    TYA = 0x98, // transfer y to accumulator
}

impl Six502 {
    pub(crate) fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            flags: 0,
            mem: [0u8; 0xFF],
        }
    }

    fn read_u8(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn read_u16(&self, addr: u16) -> u16 {
        u16::from_be_bytes(
            self.mem[(addr as usize)..=(addr + 1) as usize]
                .try_into()
                .expect("It is certainly 2"),
        )
    }

    fn write_u16(&mut self, addr: u16, v: u16) {
        self.write_u8(addr, (v >> 8) as u8);
        self.write_u8(addr + 1, (v & 0x00FF) as u8);
    }

    pub fn run(&mut self) {}

    fn write_u8(&mut self, addr: u16, v: u8) {
        self.mem[addr as usize] = v;
    }

    fn load(&mut self, prog: &[u8]) {
        //comeback
        //assert that the program is not longer than accepted mem space
        assert!(0x8000 + prog.len() < 0xff);

        self.mem[0x8000..(0x8000 + prog.len())].copy_from_slice(prog);

        //save the reference to the code into 0xFFFC
        self.write_u16(0xFFFC, 0x8000);

        self.pc = 0x8000;
    }

    //sets the zero and negative flags as is appropriate
    fn update_flags_lda(&mut self, v: u8) {
        if self.x == 0 {
            self.flags.add_assign(0b0000_0010);
        } else {
            self.flags.add_assign(0b1111_1101);
        };

        if self.a & 0b1000_0000 != 0 {
            // MSB is set
            self.flags.add_assign(0b1000_0000);
        } else {
            self.flags.add_assign(0b0111_1111);
        };
    }

    fn op_addr(&self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Absolute => self.read_u16(self.pc),
            AddressingMode::ZeroPage => self.read_u8(self.pc) as u16,
            AddressingMode::ZeroPage_X => self.read_u8(self.pc).saturating_add(self.x) as u16,
            AddressingMode::ZeroPage_Y => self.read_u8(self.pc).saturating_add(self.y) as u16,
            AddressingMode::Immediate => self.pc,
            AddressingMode::Relative => todo!(),
            AddressingMode::Implicit => todo!(),
            AddressingMode::Indirect => todo!(),
            AddressingMode::IndexedIndirect => todo!(),
            AddressingMode::IndirectINdexed => todo!(),
        }
    }

    fn interpret(&mut self, prog: Vec<u8>) {
        loop {
            //the opcode comes before the args
            let opcode = prog.index(self.pc as usize).to_owned();
            self.pc += 1;

            match opcode {
                0xA9 => {
                    self.lda(AddressingMode::Immediate);
                    self.pc += 1;
                }

                0xAA => {
                    self.x = self.a;
                    self.update_flags_lda(self.x);
                }

                0xA5 => {
                    self.lda(AddressingMode::ZeroPage);
                    self.pc += 1;
                }

                0xAD => {
                    self.lda(AddressingMode::Absolute);
                    self.pc += 1;
                }

                0x85 => {
                    self.sta(AddressingMode::ZeroPage);
                    self.pc += 1;
                }

                0x95 => {
                    self.sta(AddressingMode::ZeroPage_X);
                    self.pc += 1;
                }

                0x00 => {
                    return;
                }

                _ => todo!(),
            }
        }
    }
}


// load/store ops
impl Six502 {
    fn lda(&mut self, mode: AddressingMode) {
        let addr = self.op_addr(mode);
        let v = self.read_u8(addr);
        self.a = v;
        self.update_flags_lda(v);
    }

    fn sta(&mut self, mode: AddressingMode) {
        let addr = self.op_addr(mode);
        self.write_u8(addr, self.a);
    }
}

// register transfers
impl Six502 {

}

// stack ops
impl Six502{

}

// logical ops
impl Six502{

}

// arithmetic ops
impl Six502{

}

//incrs and decrs
impl Six502{

}

// shifts
impl Six502{

}

// jumps and calls
impl Six502{

}

// branches
impl Six502{

}

// status flag changes 
impl Six502{

}

// system functions
impl Six502{

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_0xa9() {
        let mut cpu = Six502::new();
        cpu.interpret(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.a, 0xA9);
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
        assert_eq!(cpu.x, cpu.a);
    }

    #[test]
    fn test_0xa9_0xaa() {
        let mut cpu = Six502::new();
        cpu.interpret(vec![0xa9, 0x05, 0xAA, ox00]);
        assert_eq!(cpu.x, 0x05);
    }

    fn test_ops() {
        let mut cpu = Six502::new();
        let prog = vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00];
        assert_eq!(cpu.x, 0xc1);
    }
}
