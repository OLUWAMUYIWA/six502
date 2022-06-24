use super::addr_mode;
use super::flags;
use super::Six502;
use crate::bus::{ByteAccess, WordAccess};
use std::ops::{Add, AddAssign};

const STACK_OFFSET: u16 = 0x0100;

impl Six502 {
    pub(super) fn load_u8_bump_pc(&mut self) -> u8 {
        let addr = self.pc;
        self.pc = self.pc.wrapping_add(1);
        self.load_u8(addr)
    }

    pub(super) fn load_u16_bump_pc(&mut self) -> u16 {
        let addr = self.pc;
        self.pc = self.pc.wrapping_add(2);
        self.load_u16(addr)
    }

    // stack helpers

    pub(super) fn push_u8(&mut self, b: u8) {
        let addr = u16::from(STACK_OFFSET + self.s as u16);
        self.store_u8(addr, b);
        self.s.wrapping_sub(1);
    }

    pub(super) fn pull_u8(&mut self) -> u8 {
        let addr = u16::from(STACK_OFFSET + self.s as u16) + 1;
        let v = self.load_u8(addr);
        self.s.wrapping_add(1);
        v
    }

    pub(super) fn push_u16(&mut self, w: u16) {
        let addr = u16::from(STACK_OFFSET + (self.s - 1) as u16);
        self.store_u16(addr, w);
        self.s.wrapping_sub(2);
    }

    pub(super) fn pull_u16(&mut self) -> u16 {
        let addr = u16::from(STACK_OFFSET + self.s as u16) + 1;
        let v = self.load_u16(addr);
        self.s.wrapping_add(2);
        v
    }

    // flag helpers
    // sets the flag provided in the argument
    pub(super) fn set_flag(&mut self, flag: u8) {
        self.p |= flag;
    }

    pub(super) fn clear_flag(&mut self, flag: u8) {
        self.p &= !flag // flip flag and bit_and_assign it to self.p
    }

    // assert_flag is different from set_flag in the sense that if the operation fails to fulfil a condition for changing the flag
    // the flag in question is reset by the processor anyways, to ensure that the flags ar eoperated by every op that affects them,
    // hence ensuring that they are in perfect, up-to-date state
    pub(super) fn assert_flag(&mut self, flag: u8, cond: bool) {
        if cond {
            self.set_flag(flag);
        } else {
            {
                self.clear_flag(flag);
            }
        }
    }

    pub(super) fn is_flag_set(&mut self, flag: u8) -> bool {
        self.p & flag != 0
    }

    /// The zero flag is set if the accumulator result is 0, otherwise the zero flag is reset
    pub(super) fn update_z(&mut self, v: u8) {
        self.assert_flag(flags::ZERO, v == 0);
    }

    /// The negative flag is set if the accumulator result has bit 7 on, otherwise the negative flag is reset.
    pub(super) fn update_n(&mut self, v: u8) {
        self.assert_flag(flags::NEGATIVE, v & 0x80 != 0);
    }

    // misc opcode impls
    pub(super) fn nop(&self) -> bool {
        false
    }
}

/// the overflow flag, used to indicate when a carry from 7 bits has occurred.
/// NB, here, we use zero indexing in the explanations
/// The generation of a carry out of the field in signed arithmetic is the same as when adding two 8-bit numbers(unsigned arith), except for the fact that the normal carry flag
/// does not correctly represent the fact that the field has been exceeded.
/// this is necessary because in the case of signed aritmetic, addition occurs in the 0-6 bits and not the bit 7
/// essentially, the 7th bit serves as the carry bit (the 8th is supposed to be, but since the operation only morally considers 0-6, as the 7th is the sign bit)
/// **So,The overflow flag is set whenever the sign bit (bit 7) is changed as a result of the operation.**
/// two cases:
/// 1.     0100 + 0100 = 1000 => overflow flag is turned on.
/// 2.     1000 + 1000 = 0000 => overflow flag is turned on.
/// Mixed-sign addition never turns on the overflow flag.
/// https://www.quora.com/What-is-the-difference-in-carry-and-overflow-flag-during-binary-multiplication
pub(super) fn check_overflow(a: u8, b: u8, res: u8) -> bool {
    // res refers to the value of the result after add or sub
    // a and b are the operands
    // two conditions anded together
    // 1: only one of the two operands is **not** negative (bit 7 set). i.e. they're either both pos or neg
    // 2. both the result and either of the operands are inverse of each other
    (a ^ res) & (b ^ res) & 0x80 != 0
}

// pub fn load_u8(&mut self, addr: u16) -> u8 {
//     match addr {
//         0x000..=0x1fff => self.ram.load_u8(addr),
//         0x2000..=0x3fff => todo!("ppu"),
//         0x4015 => todo!("apu"),
//         0x4016 => todo!("controller"),
//         0x4018 => todo!("apu"),
//         0x4020..=0xffff => todo!("mapper"),
//         _ => panic!("invalid load from: {:02x}", addr),
//     }
// }

// pub fn load_u16(&mut self, addr: u16) -> u16 {
//     u16::from_le_bytes([self.load_u8(addr), self.load_u8(addr + 1)])
// }

// pub fn load_u16_no_carry(&self, addr: u8) -> u16 {
//     u16::from_le_bytes([self.load_u8(addr as u16), self.load_u8(addr as u16)])
// }

// pub fn store_u8(&mut self, addr: u16, val: u8) {
//     match addr {
//         0x0000..=0x1fff => self.ram.store_u8(addr, val),
//         0x2000..=0x3fff => todo!("ppu"),
//         0x4016 => todo!("controller"),
//         0x4000..=0x4017 => todo!("apu"),
//         0x4020..=0xFFFF => todo!("mapper"),
//         _ => panic!("invalid store to {:02x}", addr),
//     }
// }

// pub fn store_u16(&mut self, addr: u16, val: u16) {
//     self.store_u8(addr, val as u8);
//     self.store_u8(addr + 1, (val >> 8) as u8);
// }

// Source: https://web.archive.org/web/20210428044647/http://www.obelisk.me.uk/6502/reference.html
// pub enum OpCode {
//     ADC,        // add with carry
//     AND,        // logical and
//     ASL,        // Arithmetic shift left
//     BCC = 0x90, // bramch if carry c;ear
//     BCS = 0xb0, // branch if carry set
//     BEQ = 0xf0, // branch if equla
//     BIT,        // bit test
//     BMI = 0x30, // branch if minus
//     BNE = 0xd0, // branch if not equal
//     BPL = 0x10, // branch if positive
//     BRK = 0x00, // force interrupt
//     BVC = 0x50, // branch if overflow clear
//     BVS = 0x70, // branch if overflow set
//     CLC = 0x18, // clear carry flag
//     CLD = 0xd8, // clear decimal node
//     CLI = 0x58, // clear interrupt disable
//     CLV = 0xb8, // clear overflow flag
//     CMP,        // compare
//     CPX,        // compare x register
//     CPY,        // cmpare y register
//     DEC,        // decrement memory
//     DEX = 0xca, // decrement x register
//     DEY = 0x88, // decrement y register
//     EOR,        // exclusive or
//     INC,        // increment memory
//     INX = 0xe8, // increment x register
//     INY = 0xc8, // increment y register
//     JMP = 0x4c, // jump
//     JSR = 0x20, // jump to subroutine
//     LDA,        // load accumulator
//     LDX,        // load x register
//     LDY,        // load y register
//     LSR,        // logical shift right
//     NOP = 0xEA, // no-op
//     ORA,        // logical inclusive or
//     PHA = 0x48, // push accumulator
//     PHP = 0x08, // push processor status
//     PLA = 0x68, // pull accumulator
//     PLP = 0x28, // pull processor status
//     ROL,        // rotate left
//     ROR,        // rotate right
//     RTI = 0x40, // return from interrupt
//     RTS = 0x60, // return from subroutine
//     SBC,        // subtract with carry
//     SEC = 0x38, // set carry flag
//     SED = 0xf8, // set decimal flag
//     SEI = 0x78, // set interrupt disable
//     STA,        // store accumulator
//     STX,        // store x register
//     STY,        // store y register
//     TAX = 0xaa, // transfer accumulator to x
//     TAY = 0xa8, // transfer accumulator to y
//     TSX = 0xba, // transfer stack pointer to x
//     TXA = 0x8a, // transfer x to accumulator
//     TXS = 0x9a, // transfer x to stack pointer
//     TYA = 0x98, // transfer y to accumulator
// }
