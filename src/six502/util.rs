use super::{addr_mode, flags, vectors, Six502};
use crate::{
    bus::{ByteAccess, WordAccess},
    AddressingMode,
};
use std::ops::{Add, AddAssign};

const STACK_OFFSET: u16 = 0x0100;

impl<T: FnMut(&mut Self, AddressingMode) -> u8> Six502<T> {
    /// Tthe concept of interrupt is used to signal the microprocessor that an external event has occurred and the
    /// microprocessor should devote attention to it immediately.  
    /// This technique accomplishes processing in which the microprocessor's program is interrupted and the event that caused the interrupt is serviced.

    pub(super) fn interrupt(&mut self) {}

    // gives the user the ability to interrupt an interrupt
    // used when a high priority device which cannot afford to Wait during the time interrupts are disabled (using the IRQ).
    // when this line goes from high to low, the microprocessor sets an internal flag  such that at the beginning of
    // the next instruction, no matter what the status of the interrupt disable, the microprocessor performs the interrupt sequence
    fn nmi(&mut self) {
        self.push_u16(self.pc);
        self.push_u8(self.p);
        // set  the interrrupt disable flag
        self.p |= flags::IRQ;
        self.pc = self.load_u16(vectors::NMI);
        self.cy += 7;
    }

    fn irq(&mut self) {
        self.push_u16(self.pc);
        self.push_u8(self.p);
        // set  the interrrupt disable flag
        self.p |= flags::IRQ;
        self.pc = self.load_u16(vectors::IRQ);
        self.cy += 7;
    }
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

    // STACK
    // The stack in the MCS650X family is a push-down stack implemented
    //  by a processor register called the stack pointer which the programmer ini-
    //  tializes by means of a Load X immediately followed by a TXS instruction and
    //  thereafter Is controlled by the microprocessor which loads data into mem-
    //  ory based on an address constructed by adding the contents of the stack
    //  pointer to a fixed address, Hex address 0100.  Every time the microproces-
    //  sor loads data into memory using the stack pointer, it automatically decre-
    //  ments the stack pointer, thereby leaving the stack pointer pointing at the
    //  next open memory byte.  Every time the microprocessor accesses data from
    //  the stack, it adds 1 to the current value of the stack pointer and reads
    //  the memory location by putting out the address 0100 plus the stack pointer.
    //  The Status register is automatically pointing at the next memory location
    //  to which data can now be written.  The stack makes an interesting place to
    //  store interim data without the programmer having to worry about the actual
    //  memory location in which data will be directly stored.
    // operations which put data on the stack cause the pointer to be decremented automatically
    pub(super) fn push_u8(&mut self, b: u8) {
        let addr = STACK_OFFSET + self.s as u16;
        self.store_u8(addr, b);
        self.s = self.s.wrapping_sub(1);
    }

    // operations which pull data from the stack cause the pointer to be incremented automatically
    // adds 1 to the current value of the stack pointer and uses it to address the stack
    pub(super) fn pull_u8(&mut self) -> u8 {
        let addr = STACK_OFFSET + self.s as u16 + 1;
        let v = self.load_u8(addr);
        self.s = self.s.wrapping_add(1);
        v
    }

    pub(super) fn push_u16(&mut self, w: u16) {
        let addr = STACK_OFFSET + (self.s - 1) as u16;
        self.store_u16(addr, w);
        self.s = self.s.wrapping_sub(2);
    }

    pub(super) fn pull_u16(&mut self) -> u16 {
        let addr = STACK_OFFSET + self.s as u16 + 1;
        let v = self.load_u16(addr);
        self.s = self.s.wrapping_add(2);
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
    pub(super) fn nop(&mut self, _mode: AddressingMode) -> u8 {
        0
    }

    // atom does any number of ops and ticks once
    pub(super) fn atom<F: FnMut(&mut Six502<T>)>(&mut self, mut f: F) {
        f(self);
        self.cy += 1;
    }

    pub(super) fn tick(&mut self) {
        self.cy += 1;
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

pub(super) fn num_cy(b: bool) -> u8 {
    if b {
        1
    } else {
        0
    }
}
