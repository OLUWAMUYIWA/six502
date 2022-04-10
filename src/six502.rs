#![allow(dead_code, unused_variables, unused_imports)]

use std::{
    ops::{AddAssign, Index, RangeBounds},
    simd::u32x16,
};

//comeback: where is jmpi

mod flags {
    const CARRY: U8 = 1 << 0;
    const ZERO: u8 = 1 << 1; //set to 1 on equality
    const IRQ: u8 = 1 << 2;
    const DECIMAL: u8 = 1 << 3;

    const BREAK: u8 = 1 << 4;
    const OVERFLOW: u8 = 1 << 6;
    const NEGATIVE: u8 = 1 << 7;
}

pub struct Six502 {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    s: u8,
    flags: u8,
    mem: [u8; 0xFF],
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    Accumulator,
    Absolute,
    ZeroPage, //This type of addressing is called “zero page” - only the first page (the first 256 bytes) of memory is accessible
    ZeroPage_X,
    ZeroPage_Y,
    Absolute_X,
    Absolute_Y,
    IndexedIndirect,
    IndirectIndexed,
}

impl AddressingMode {
    fn load(&self, cpu: &mut Six502) -> u8 {
        match self {
            AddressingMode::Immediate => cpu.load_u8_bump_pc(),
            AddressingMode::Accumulator => cpu.a,
            AddressingMode::Absolute => cpu.load_u8(cpu.load_u16_bump_pc()),
            AddressingMode::Absolute_X => cpu.load_u8(cpu.load_u16_bump_pc() + (cpu.x as u16)),
            AddressingMode::Absolute_Y => cpu.load_u8(cpu.load_u16_bump_pc() + (cpu.y as u16)),
            AddressingMode::ZeroPage => cpu.load_u8(cpu.load_u8_bump_pc()),
            AddressingMode::ZeroPage_X => cpu.load_u8((cpu.load_u8_bump_pc() + cpu.x) as u16),
            AddressingMode::ZeroPage_Y => cpu.load_u8((cpu.load_u8_bump_pc() + cpu.y) as u16),
            AddressingMode::IndexedIndirect => {
                let v = cpu.load_u8_bump_pc();
                let x = self.x;
                cpu.load_u8(cpu.load_u16(v + x as u16))
            }
            AddressingMode::IndirectIndexed => {
                let v = cpu.load_u8_bump_pc();
                let y = self.y;
                cpu.load_u8(cpu.load_u16(v + y as u16))
            }
        }
    }
    fn store(&self, cpu: &mut Six502, v: u8) {
        match self {
            AddressingMode::Immediate => {} // do nothing
            AddressingMode::Accumulator => cpu.a = v,
            AddressingMode::Absolute => cpu.store_u8(cpu.load_u16_bump_pc(), v),
            AddressingMode::Absolute_X => cpu.store_u8(cpu.load_u16_bump_pc() + (cpu.x as u16), v),
            AddressingMode::Absolute_Y => cpu.store_u8(cpu.load_u16_bump_pc() + (cpu.y as u16), v),
            AddressingMode::ZeroPage => cpu.store_u8(cpu.load_u8_bump_pc(), v),
            AddressingMode::ZeroPage_X => cpu.store_u8((cpu.load_u8_bump_pc() + cpu.x) as u16, v),
            AddressingMode::ZeroPage_Y => cpu.store_u8((cpu.load_u8_bump_pc() + cpu.y) as u16, v),
            AddressingMode::IndexedIndirect => {
                let val = cpu.load_u8_bump_pc();
                let x = self.x;
                cpu.store_u8(cpu.load_u16(val + x as u16), v)
            }
            AddressingMode::IndirectIndexed => {
                let val = cpu.load_u8_bump_pc();
                let y = self.y;
                cpu.store_u8(cpu.load_u16(val + y as u16), v)
            }
        }
    }
}

// Source: https://web.archive.org/web/20210428044647/http://www.obelisk.me.uk/6502/reference.html
pub enum OpCode {
    ADC,        // add with carry
    AND,        // logical and
    ASL,        // Arithmetic shift left
    BCC = 0x90, // bramch if carry c;ear
    BCS = 0xb0, // branch if carry set
    BEQ = 0xf0, // branch if equla
    BIT,        // bit test
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
    CMP,        // compare
    CPX,        // compare x register
    CPY,        // cmpare y register
    DEC,        // decrement memory
    DEX = 0xca, // decrement x register
    DEY = 0x88, // decrement y register
    EOR,        // exclusive or
    INC,        // increment memory
    INX = 0xe8, // increment x register
    INY = 0xc8, // increment y register
    JMP = 0x4c, // jump
    JSR = 0x20, // jump to subroutine
    LDA,        // load accumulator
    LDX,        // load x register
    LDY,        // load y register
    LSR,        // logical shift right
    NOP = 0xEA, // no-op
    ORA,        // logical inclusive or
    PHA = 0x48, // push accumulator
    PHP = 0x08, // push processor status
    PLA = 0x68, // pull accumulator
    PLP = 0x28, // pull processor status
    ROL,        // rotate left
    ROR,        // rotate right
    RTI = 0x40, // return from interrupt
    RTS = 0x60, // return from subroutine
    SBC,        // subtract with carry
    SEC = 0x38, // set carry flag
    SED = 0xf8, // set decimal flag
    SEI = 0x78, // set interrupt disable
    STA,        // store accumulator
    STX,        // store x register
    STY,        // store y register
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
            pc: 0xc000,
            s: 0xfd,
            flags: 0x24,
            mem: [0u8; 0xFF],
        }
    }

    fn load_u8(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn load_u16(&self, addr: u16) -> u16 {
        u16::from_be_bytes(
            self.mem[(addr as usize)..=(addr + 1) as usize]
                .try_into()
                .expect("It is certainly 2"),
        )
    }

    fn load_u8_bump_pc(&mut self) -> u8 {
        let addr = self.pc;
        self.pc += 1;
        self.load_u8(addr)
    }

    fn load_u16_bump_pc(&mut self) -> u16 {
        let addr = self.pc;
        self.pc += 2;
        self.load_u16(addr)
    }

    fn store_u16(&mut self, addr: u16, v: u16) {
        self.store_u8(addr, (v >> 8) as u8);
        self.store_u8(addr + 1, (v & 0x00FF) as u8);
    }

    fn store_u8(&mut self, addr: u16, v: u8) {
        self.mem[addr as usize] = v;
    }

    pub fn run(&mut self) {}

    fn load(&mut self, prog: &[u8]) {
        //comeback
        //assert that the program is not longer than accepted mem space
        assert!(0x8000 + prog.len() < 0xff);

        self.mem[0x8000..(0x8000 + prog.len())].copy_from_slice(prog);

        //save the reference to the code into 0xFFFC
        self.store_u16(0xFFFC, 0x8000);

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

//flags
impl Six502 {
    fn flag_on(&mut self, flag: u8) {
        self.flags |= flag;
    }

    fn flag_off(&mut self, flag: u8) {
        self.flags &= !flag
    }

    fn is_flag_set(&mut self, flag: u8) -> bool {
        self.flags & flag != 0
    }
}

// load/store ops
impl Six502 {
    fn lda(&mut self, mode: AddressingMode) {
        let v = mode.load(self);
        self.a = v;
        if v == 0 {
            self.flag_on(flags::ZERO);
        }
        if v & 0x80 != 0 {
            self.flag_on(flags::NEGATIVE);
        }
    }

    fn ldx(&mut self, mode: AddressingMode) {
        let v = mode.load(self);
        self.x = v;
        if v == 0 {
            self.flag_on(flags::ZERO);
        }
        if v & 0x80 != 0 {
            self.flag_on(flags::NEGATIVE);
        }
    }

    fn ldy(&mut self, mode: AddressingMode) {
        let v = mode.load(self);
        self.y = v;
        if v == 0 {
            self.flag_on(flags::ZERO);
        }
        if v & 0x80 != 0 {
            self.flag_on(flags::NEGATIVE);
        }
    }

    fn sta(&mut self, mode: AddressingMode) {
        mode.store(self, self.a)
    }

    fn stx(&mut self, mode: AddressingMode) {
        mode.store(self, self.x)
    }

    fn sty(&mut self, mode: AddressingMode) {
        mode.store(self, self.y)
    }
}

//comeback
// stack impls
impl Six502 {
    fn push_u8(&mut self, b: u8) {
        let addr = (0x100 + self.s) as u16;
        self.store_u8(addr, b);
        self.s -= 1;
    }

    fn pull_u8(&mut self) -> u8 {
        let addr = (0x100 + self.s) as u16 + 1;
        let v = self.load_u8(addr);
        self.s += 1;
        v
    }

    fn push_u16(&mut self, w: u16) {
        let addr = (0x100 + (self.s - 1)) as u16;
        self.store_u16(addr, w);
        self.s -= 2;
    }

    fn pull_u16(&mut self) -> u16 {
        let addr = (0x100 + self.s) as u16 + 1;
        let v = self.load_u16(addr);
        self.s += 2;
        v
    }
}

fn check_overflow(a: u8, b: u8, res: u8) -> bool {
    ((a ^ res) & 0x80 != 0) && ((a ^ b) & 0x80 == 0x80)
}

impl Six502 {
    fn adc(&mut self, mode: AddressingMode) {
        let b = mode.load(self) as u16;
        let a = self.a as u16;

        let res = if self.is_flag_set(flags::CARRY) {
            // CARRY flag may conatain a `1` from a previous computation that added a set of lower significant
            // bits. this carry may then be pushed over to the next (immediately higher) group of bits as a unit of 1
            // because in this higher batch of operands, it is a unit value.
            a + b + 1
        } else {
            a + b
        };

        if res & 0x100 != 0 {
            self.flag_on(flags::CARRY);
        }

        if check_overflow(a, b, res as u8) {
            self.flag_on(flags::OVERFLOW);
        }

        if res == 0 {
            self.flag_on(flags::ZERO);
        }

        if res as u8 & 0x80 != 0 {
            self.flag_on(flags::NEGATIVE);
        }
    }

    fn sbc(&mut self, mode: AddressingMode) {
        let a = self.a as u16;
        let b = mode.load(self) as u16;
        let res = if self.is_flag_set(flags::CARRY) {
            a - (b + 1)
        } else {
            a - b
        };

        if res & 0x100 == 0 {
            self.flag_on(flags::CARRY);
        }

        if check_overflow(a, b, res as u8) {
            self.flag_on(flags::OVERFLOW);
        }

        if res == 0 {
            self.flag_on(flags::ZERO);
        }

        if res as u8 & 0x80 != 0 {
            self.flag_on(flags::NEGATIVE);
        }
    }

    fn cmp(&mut self, mode: AddressingMode) {
        let b = mode.load(self) as u16;
        let a = self.a as u16;
        let res = a - b;
        if res & 0x100 == 0 {
            self.flag_on(flags::CARRY);
        }
        if res == 0 {
            self.flag_on(flags::ZERO);
        }

        if res as u8 & 0x80 != 0 {
            self.flag_on(flags::NEGATIVE);
        }
    }

    fn cpx(&mut self, mode: AddressingMode) {
        let b = mode.load(self) as u16;
        let a = self.x as u16;
        let res = a - b;
        if res & 0x100 == 0 {
            self.flag_on(flags::CARRY);
        }
        if res == 0 {
            self.flag_on(flags::ZERO);
        }

        if res as u8 & 0x80 != 0 {
            self.flag_on(flags::NEGATIVE);
        }
    }

    fn cpy(&mut self, mode: AddressingMode) {
        let b = mode.load(self) as u16;
        let a = self.y as u16;
        let res = a - b;
        if res & 0x100 == 0 {
            self.flag_on(flags::CARRY);
        }
        if res == 0 {
            self.flag_on(flags::ZERO);
        }

        if res as u8 & 0x80 != 0 {
            self.flag_on(flags::NEGATIVE);
        }
    }

    fn and(&mut self, mode: AddressingMode) {
        let b = mode.load(self);
        self.a = self.a &= b;
        if self.a == 0 {
            self.flag_on(flags::ZERO);
        }

        if self.a & 0x80 != 0 {
            self.flag_on(flags::NEGATIVE);
        }
    }

    fn ora(&mut self, mode: AddressingMode) {
        let b = mode.load(self);
        self.a = self.a |= b;
        if self.a == 0 {
            self.flag_on(flags::ZERO);
        }

        if self.a & 0x80 != 0 {
            self.flag_on(flags::NEGATIVE);
        }
    }

    fn eor(&mut self, mode: AddressingMode) {
        let b = mode.load(self);
        self.a = self.a ^= b;
        if self.a == 0 {
            self.flag_on(flags::ZERO);
        }

        if self.a & 0x80 != 0 {
            self.flag_on(flags::NEGATIVE);
        }
    }
}

// register transfers
impl Six502 {}

// stack ops
impl Six502 {}

// logical ops
impl Six502 {}

// arithmetic ops
impl Six502 {}

//incrs and decrs
impl Six502 {}

// shifts
impl Six502 {}

// jumps and calls
impl Six502 {}

// branches
impl Six502 {}

// status flag changes
impl Six502 {}

// system functions
impl Six502 {}

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
