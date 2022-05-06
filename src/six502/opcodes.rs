use super::{addressing::AddressingMode, flags, Six502};
use super::{BRK_VECTOR, IRQ_VECTOR, NMI_VECTOR, RESET_VECTOR, STACK_OFFSET};
use std::ops::{Shl, Shr};

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

// load/store ops
impl Six502 {
    pub(super) fn lda(&mut self, mode: AddressingMode) -> bool {
        let (carry, v) = mode.load(self);
        self.a = v;
        self.update_zero_neg_flags(v);
        carry
    }

    pub(super) fn ldx(&mut self, mode: AddressingMode) -> bool {
        let (carry, v) = mode.load(self);
        self.x = v;
        self.update_zero_neg_flags(v);
        carry
    }

    pub(super) fn ldy(&mut self, mode: AddressingMode) -> bool {
        let (carry, v) = mode.load(self);
        self.y = v;
        self.update_zero_neg_flags(v);
        carry
    }

    pub(super) fn sta(&mut self, mode: AddressingMode) -> bool {
        mode.store(self, self.a)
    }

    pub(super) fn stx(&mut self, mode: AddressingMode) -> bool {
        mode.store(self, self.x)
    }

    pub(super) fn sty(&mut self, mode: AddressingMode) -> bool {
        mode.store(self, self.y)
    }
}

pub(super) fn check_overflow(a: u8, b: u8, res: u8) -> bool {
    ((a ^ res) & 0x80 != 0) && ((a ^ b) & 0x80 == 0x80)
}

// comparisons
impl Six502 {
    pub(super) fn cmp(&mut self, mode: AddressingMode) -> bool {
        let (v, carry) = mode.load(self);
        let b = v as u16;
        let a = self.a as u16;
        let res = a - b;
        if res & 0x100 == 0 {
            self.flag_on(flags::CARRY);
        }
        self.update_zero_neg_flags(res as u8);
        carry
    }

    pub(super) fn cpx(&mut self, mode: AddressingMode) -> bool {
        let (v, carry) = mode.load(self);
        let b = v as u16;
        let a = self.x as u16;
        let res = a - b;
        if res & 0x100 == 0 {
            self.flag_on(flags::CARRY);
        }
        self.update_zero_neg_flags(res as u8);
        carry
    }

    pub(super) fn cpy(&mut self, mode: AddressingMode) -> bool {
        let (v, carry) = mode.load(self);
        let b = v as u16;
        let a = self.y as u16;
        let res = a - b;
        if res & 0x100 == 0 {
            self.flag_on(flags::CARRY);
        }
        self.update_zero_neg_flags(res as u8);
        carry
    }

    pub(super) fn bit(&mut self, mode: AddressingMode) -> bool {
        let a = self.a;
        let (b, carry) = mode.load(self);
        if a & b == 0 {
            self.flag_on(flags::ZERO);
        }
        if b & 0x80 != 0 {
            self.flag_on(flags::NEGATIVE);
        }
        if b & 0x40 != 0 {
            self.flag_on(flags::OVERFLOW);
        }
        carry
    }
}

// register transfers
impl Six502 {
    pub(super) fn tax(&mut self) -> bool {
        self.x = self.a;
        self.update_zero_neg_flags(self.a);
        false
    }

    pub(super) fn txa(&mut self) -> bool {
        self.a = self.x;
        self.update_zero_neg_flags(self.x);
        false
    }

    pub(super) fn tay(&mut self) -> bool {
        self.y = self.a;
        self.update_zero_neg_flags(self.a);
        false
    }

    pub(super) fn tya(&mut self) -> bool {
        self.a = self.y;
        self.update_zero_neg_flags(self.y);
        false
    }

    pub(super) fn tsx(&mut self) -> bool {
        self.x = self.s;
        self.update_zero_neg_flags(self.s);
        false
    }

    pub(super) fn txs(&mut self) -> bool {
        self.s = self.x;
        self.update_zero_neg_flags(self.x);
        false
    }
}

// stack ops
impl Six502 {
    // helpers
    pub(super) fn push_u8(&mut self, b: u8) {
        let addr = (STACK_OFFSET + self.s) as u16;
        self.store_u8(addr, b);
        self.s -= 1;
    }

    pub(super) fn pull_u8(&mut self) -> u8 {
        let addr = (STACK_OFFSET + self.s) as u16 + 1;
        let v = self.load_u8(addr);
        self.s += 1;
        v
    }

    pub(super) fn push_u16(&mut self, w: u16) {
        let addr = (STACK_OFFSET + (self.s - 1)) as u16;
        self.store_u16(addr, w);
        self.s -= 2;
    }

    pub(super) fn pull_u16(&mut self) -> u16 {
        let addr = (STACK_OFFSET + self.s) as u16 + 1;
        let v = self.load_u16(addr);
        self.s += 2;
        v
    }

    pub(super) fn pha(&mut self) -> bool {
        self.push_u8(self.a);
        false
    }

    pub(super) fn pla(&mut self) -> bool {
        let v = self.pull_u8();
        self.update_zero_neg_flags(v);
        self.a = v;
        false
    }

    pub(super) fn php(&mut self) -> bool {
        let flags = self.p;
        self.push_u8(flags | flags::BREAK);
        false
    }

    pub(super) fn plp(&mut self) -> bool {
        let val = self.pull_u8();
        self.p = (val | 0x30) - 0x10;
        false
    }
}

// logical ops
impl Six502 {
    pub(super) fn and(&mut self, mode: AddressingMode) -> bool {
        let (b, carry) = mode.load(self);
        self.a = self.a &= b;
        self.update_zero_neg_flags(self.a);
        carry
    }

    pub(super) fn ora(&mut self, mode: AddressingMode) {
        let b = mode.load(self);
        self.a = self.a |= b;
        self.update_zero_neg_flags(self.a);
    }

    pub(super) fn eor(&mut self, mode: AddressingMode) {
        let b = mode.load(self);
        self.a = self.a ^= b;
        self.update_zero_neg_flags(self.a);
    }
}

// arithmetic ops
impl Six502 {
    pub(super) fn adc(&mut self, mode: AddressingMode) {
        let a = self.a as u16;
        let (v, carry) = mode.load(self);
        let b = v as u16;

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

        if check_overflow(a as u8, b as u8, res as u8) {
            self.flag_on(flags::OVERFLOW);
        }
        self.a = res as u8;
        self.update_zero_neg_flags(res as u8);
        carry
    }

    pub(super) fn sbc(&mut self, mode: AddressingMode) {
        let a = self.a as u16;
        let (v, carry) = mode.load(self);
        let b = v as u16;
        let res = if self.is_flag_set(flags::CARRY) {
            a - (b + 1)
        } else {
            a - b
        };

        if res & 0x100 == 0 {
            self.flag_on(flags::CARRY);
        }

        if check_overflow(a as u8, b as u8, res as u8) {
            self.flag_on(flags::OVERFLOW);
        }

        self.a = res as u8;
        self.update_zero_neg_flags(res as u8);
        carry
    }
}

//incrs and decrs
impl Six502 {
    pub(super) fn inc(&mut self, mode: AddressingMode) {
        let (v, carry) = mode.load(self);
        let v = v.wrapping_add(1);
        self.update_zero_neg_flags(v);

        mode.store(self, v);
    }

    pub(super) fn dec(&mut self, mode: AddressingMode) {
        let (v, carry) = mode.load(self);
        let v = v.wrapping_sub(1);
        self.update_zero_neg_flags(v);

        mode.store(self, v);
    }

    pub(super) fn inx(&mut self) {
        let x = self.x.wrapping_add(1);
        self.update_zero_neg_flags(x);

        self.x = x;
    }

    pub(super) fn dex(&mut self) {
        let x = self.x.wrapping_sub(1);
        self.update_zero_neg_flags(x);

        self.x = x;
    }

    pub(super) fn iny(&mut self) {
        let y = self.y.wrapping_add(1);
        self.update_zero_neg_flags(y);

        self.y = y;
    }

    pub(super) fn dey(&mut self) {
        let y = self.y.wrapping_sub(1);
        self.update_zero_neg_flags(y);

        self.y = y;
    }
}

// shifts
impl Six502 {
    pub(super) fn rol(&mut self, mode: AddressingMode) {
        let b = mode.load(self);
        let mut res = b.shl(1);
        if self.is_flag_set(flags::CARRY) {
            res.bitor_assign(1);
        }
        if b & 0x80 != 0 {
            self.flag_on(flags::CARRY);
        }

        self.update_zero_neg_flags(res);
        mode.store(self, res);
    }

    pub(super) fn asl(&mut self, mode: AddressingMode) {
        let b = mode.load(self);
        let mut res = b.shl(1);
        if b & 0x80 != 0 {
            self.flag_on(flags::CARRY);
        }

        self.update_zero_neg_flags(res);

        mode.store(self, res);
    }

    pub(super) fn ror(&mut self, mode: AddressingMode) {
        let b = mode.load(self);
        let mut res = b.shr(1);
        if self.is_flag_set(flags::CARRY) {
            res.bitor_assign(0x80);
        }
        if (b & 0x1) != 0 {
            self.flag_on(flags::CARRY);
        }

        self.update_zero_neg_flags(res);

        mode.store(self, res);
    }

    pub(super) fn lsr(&mut self, mode: AddressingMode) {
        let b = mode.load(self);
        let mut res = b.shr(1);
        if (b & 0x1) != 0 {
            self.flag_on(flags::CARRY);
        }

        self.update_zero_neg_flags(res);

        mode.store(self, res);
    }
}

// jumps and calls
impl Six502 {
    const BRK_VECTOR: u16 = 0xfffe;

    pub(super) fn jmp(&mut self) {
        self.pc = self.load_u16_bump_pc();
    }

    pub(super) fn jmpi(&mut self) {
        let op = self.load_u16_bump_pc();
        let lo = cpu.load_u8(op);
        let hi = cpu.load_u8((op & 0xff00) | ((op + 1) & 0x00ff));
        self.pc = u16::from_le_bytes([lo, hi]);
    }

    pub(super) fn jsr(&mut self) {
        let pc = self.pc;
        let addr = self.load_u16_bump_pc();
        self.push_u16(pc - 1); // push curr pc-1 to the stack
        self.pc = addr;
    }

    pub(super) fn rts(&mut self) {
        let pos = self.pull_u16() + 1;
        self.pc = pos;
    }

    pub(super) fn brk(&mut self) {
        let pc = self.pc;
        self.push_u16(pc + 1);
        self.push_u8(self.p);
        self.flag_on(flags::IRQ);
        self.pc = self.load_u16(BRK_VECTOR);
    }

    pub(super) fn rti(&mut self) {
        let flags = self.pull_u8();
        self.p = (self.p | 0x30) - 0x10;
        self.pc = self.pull_u16();
    }
}

// branches
impl Six502 {
    pub(super) fn bpl(&mut self) {
        let v = self.load_u8_bump_pc();

        if !self.is_flag_set(flags::NEGATIVE) {
            self.pc = self.pc.wrapping_add(v as u16);
        }
    }

    pub(super) fn bmi(&mut self) {
        let v = self.load_u8_bump_pc();
        if self.is_flag_set(flags::NEGATIVE) {
            self.pc = self.pc.wrapping_add(v as u16);
        }
    }

    pub(super) fn bvc(&mut self) {
        let v = self.load_u8_bump_pc();
        if !self.is_flag_set(flags::OVERFLOW) {
            self.pc = self.pc.wrapping_add(v as u16);
        }
    }

    pub(super) fn bvs(&mut self) {
        let v = self.load_u8_bump_pc();
        if self.is_flag_set(flags::OVERFLOW) {
            self.pc = self.pc.wrapping_add(v as u16);
        }
    }

    pub(super) fn bcc(&mut self) {
        let v = self.load_u8_bump_pc();
        if !self.is_flag_set(flags::CARRY) {
            self.pc = self.pc.wrapping_add(v as u16);
        }
    }

    pub(super) fn bcs(&mut self) {
        let v = self.load_u8_bump_pc();
        if self.is_flag_set(flags::CARRY) {
            self.pc = self.pc.wrapping_add(v as u16);
        }
    }

    pub(super) fn bne(&mut self) {
        let v = self.load_u8_bump_pc();

        if !self.is_flag_set(flags::ZERO) {
            self.pc = self.pc.wrapping_add(v as u16);
        }
    }
    pub(super) fn beq(&mut self) {
        let v = self.load_u8_bump_pc();

        if self.is_flag_set(flags::ZERO) {
            self.pc = self.pc.wrapping_add(v as u16);
        }
    }
}

// status flag changes
impl Six502 {
    // helpers
    pub(super) fn flag_on(&mut self, flag: u8) {
        self.p |= flag;
    }

    pub(super) fn flag_off(&mut self, flag: u8) {
        self.p &= !flag
    }

    pub(super) fn is_flag_set(&mut self, flag: u8) -> bool {
        self.p & flag != 0
    }

    // cpu ops:
    pub(super) fn clc(&mut self) {
        self.flag_off(flags::CARRY);
    }
    pub(super) fn sec(&mut self) {
        self.flag_on(flags::CARRY);
    }
    pub(super) fn cli(&mut self) {
        self.flag_off(flags::IRQ);
    }
    pub(super) fn sei(&mut self) {
        self.flag_on(flags::IRQ);
    }
    pub(super) fn clv(&mut self) {
        self.flag_off(flags::OVERFLOW);
    }
    pub(super) fn cld(&mut self) {
        self.flag_off(flags::DECIMAL);
    }
    pub(super) fn sed(&mut self) {
        self.flag_on(flags::DECIMAL);
    }
}

// system functions
impl Six502 {}
