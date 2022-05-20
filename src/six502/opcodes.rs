use super::{addressing::AddressingMode, flags, Six502};
use super::{BRK_VECTOR, IRQ_VECTOR, NMI_VECTOR, RESET_VECTOR, STACK_OFFSET};
use std::ops::{BitAnd, BitOr, Shl, Shr};

// load/store ops
impl Six502 {
    pub(super) fn lda(&mut self, mode: AddressingMode) -> bool {
        let (carry, v) = mode.load(self);
        self.a = v;
        self.update_z(v);
        self.update_n(v);
        carry
    }

    pub(super) fn ldx(&mut self, mode: AddressingMode) -> bool {
        let (carry, v) = mode.load(self);
        self.x = v;
        self.update_z(v);
        self.update_n(v);
        carry
    }

    pub(super) fn ldy(&mut self, mode: AddressingMode) -> bool {
        let (carry, v) = mode.load(self);
        self.y = v;
        self.update_z(v);
        self.update_n(v);
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
    /// cmp: compare accumulator. It sets flags as if a subtraction had been carried out between the accumulator and the operand
    pub(super) fn cmp(&mut self, mode: AddressingMode) -> bool {
        let (v, cross) = mode.load(self);
        let v = v as u16;
        let a = self.a as u16;
        if a >= v {
            self.flag_on(flags::CARRY);
        }
        self.update_z((a - v) as u8);
        self.update_n((a - v) as u8);
        cross
    }

    /// cpx: compare accumulator. It sets flags as if a subtraction had been carried out between the x register and the operand
    pub(super) fn cpx(&mut self, mode: AddressingMode) -> bool {
        let (v, cross) = mode.load(self);
        let v = v as u16;
        let x = self.x as u16;
        if x >= v {
            self.flag_on(flags::CARRY);
        }

        self.update_z((x - v) as u8);
        self.update_n((x - v) as u8);
        cross
    }

    /// cpy: compare accumulator. It sets flags as if a subtraction had been carried out between the y register and the operand

    pub(super) fn cpy(&mut self, mode: AddressingMode) -> bool {
        let (v, cross) = mode.load(self);
        let v = v as u16;
        let y = self.y as u16;
        if y >= v {
            self.flag_on(flags::CARRY);
        }
        self.update_z((y - v) as u8);
        self.update_n((y - v) as u8);
        cross
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
    /// tax transfers accumulator into x register, updating the z and n flags based on the value of a
    pub(super) fn tax(&mut self) -> bool {
        self.x = self.a;
        self.update_z(self.a);
        self.update_n(self.a);
        false
    }

    pub(super) fn txa(&mut self) -> bool {
        self.a = self.x;
        self.update_z(self.x);
        self.update_n(self.x);
        false
    }

    pub(super) fn tay(&mut self) -> bool {
        self.y = self.a;
        self.update_z(self.a);
        self.update_n(self.a);
        false
    }

    pub(super) fn tya(&mut self) -> bool {
        self.a = self.y;
        self.update_z(self.y);
        self.update_n(self.y);
        false
    }

    /// tsx: Transfer Stack ptr to X
    pub(super) fn tsx(&mut self) -> bool {
        self.x = self.s;
        self.update_z(self.s);
        self.update_n(self.s);
        false
    }

    /// txs: transfer x register to stack pointer
    pub(super) fn txs(&mut self) -> bool {
        self.s = self.x;
        self.update_z(self.x);
        self.update_n(self.x);
        false
    }
}

// stack ops
impl Six502 {
    pub(super) fn pha(&mut self) -> bool {
        self.push_u8(self.a);
        false
    }

    pub(super) fn pla(&mut self) -> bool {
        let v = self.pull_u8();
        self.update_z(v);
        self.update_n(v);
        self.a = v;
        false
    }

    // php push processor status
    pub(super) fn php(&mut self) -> bool {
        let flags = self.p;
        self.push_u8(flags | flags::BREAK);
        false
    }

    /// plp pulls processor status
    pub(super) fn plp(&mut self) -> bool {
        let val = self.pull_u8();
        self.p = (val | 0x30) - 0x10;
        false
    }
}

// logical ops
impl Six502 {
    /// and: bitwise AND with accumulator. affects the n and z flags
    pub(super) fn and(&mut self, mode: AddressingMode) -> bool {
        let (v, cross) = mode.load(self);
        self.a &= v;
        self.update_z(self.a);
        self.update_n(self.a);
        cross
    }

    pub(super) fn ora(&mut self, mode: AddressingMode) -> bool {
        let (v, cross) = mode.load(self);
        self.a |= v;
        self.update_z(self.a);
        self.update_n(self.a);
        cross
    }

    pub(super) fn eor(&mut self, mode: AddressingMode) -> bool {
        let (v, cross) = mode.load(self);
        self.a ^= v;
        self.update_z(self.a);
        self.update_n(self.a);
        cross
    }
}

// arithmetic ops
impl Six502 {
    /// adc adds a value and the carry bit to the accumulator
    pub(super) fn adc(&mut self, mode: AddressingMode) -> bool {
        let a = u16::from(self.a);
        let (v, cross) = mode.load(self);
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
        self.update_z(res as u8);
        self.update_n(res as u8);
        cross
    }

    /// sbc subtracts a value and the inverse of the carry bit from the accumulator.
    pub(super) fn sbc(&mut self, mode: AddressingMode) -> bool {
        let a = u16::form(self.a);
        let (v, cross) = mode.load(self);
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
        self.update_z(res as u8);
        self.update_n(res as u8);
        cross
    }
}

//incrs and decrs
impl Six502 {
    pub(super) fn inc(&mut self, mode: AddressingMode) -> bool {
        let (v, cross) = mode.load(self);
        let v = v.wrapping_add(1);
        self.update_z(v);
        self.update_n(v);
        mode.store(self, v);
        cross
    }

    pub(super) fn dec(&mut self, mode: AddressingMode) -> bool {
        let (v, cross) = mode.load(self);
        let v = v.wrapping_sub(1);
        self.update_z(v);
        self.update_n(v);
        mode.store(self, v);
        cross
    }

    pub(super) fn inx(&mut self) -> bool {
        let x = self.x.wrapping_add(1);
        self.update_z(x);
        self.update_n(x);
        self.x = x;
        false
    }

    pub(super) fn dex(&mut self) -> bool {
        let x = self.x.wrapping_sub(1);
        self.update_z(x);
        self.update_n(x);
        self.x = x;
        false
    }

    pub(super) fn iny(&mut self) -> bool {
        let y = self.y.wrapping_add(1);
        self.update_z(y);
        self.update_n(y);
        self.y = y;
        false
    }

    pub(super) fn dey(&mut self) -> bool {
        let y = self.y.wrapping_sub(1);
        self.update_z(y);
        self.update_n(y);
        self.y = y;
        false
    }
}

// shifts
impl Six502 {
    pub(super) fn rol(&mut self, mode: AddressingMode) -> bool {
        let (b, cross) = mode.load(self);
        let mut res = b.shl(1);
        if self.is_flag_set(flags::CARRY) {
            res.bitor_assign(1);
        }
        if b & 0x80 != 0 {
            self.flag_on(flags::CARRY);
        }

        self.update_z(res);
        self.update_n(res);
        mode.store(self, res);
        cross
    }

    pub(super) fn asl(&mut self, mode: AddressingMode) -> bool {
        let (b, cross) = mode.load(self);
        let mut res = b.shl(1);
        if b & 0x80 != 0 {
            self.flag_on(flags::CARRY);
        }

        self.update_z(res);
        self.update_n(res);
        mode.store(self, res);
        cross
    }

    pub(super) fn ror(&mut self, mode: AddressingMode) -> bool {
        let (b, cross) = mode.load(self);
        let mut res = b.shr(1);
        if self.is_flag_set(flags::CARRY) {
            res.bitor_assign(0x80);
        }
        if (b & 0x1) != 0 {
            self.flag_on(flags::CARRY);
        }
        self.update_z(res);
        self.update_n(res);
        mode.store(self, res);
        cross
    }

    pub(super) fn lsr(&mut self, mode: AddressingMode) -> bool {
        let (b, cross) = mode.load(self);
        let mut res = b.shr(1);
        if (b & 0x1) != 0 {
            self.flag_on(flags::CARRY);
        }
        self.update_z(res);
        self.update_n(res);
        mode.store(self, res);
        cross
    }
}

// jumps and calls
impl Six502 {
    const BRK_VECTOR: u16 = 0xfffe;

    pub(super) fn jmp(&mut self) {
        self.pc = self.load_u16_bump_pc();
    }

    pub(super) fn jmpi(&mut self) -> bool {
        let op = self.load_u16_bump_pc();
        let lo = cpu.load_u8(op);
        let hi = cpu.load_u8((op & 0xff00) | ((op + 1) & 0x00ff));
        self.pc = u16::from_le_bytes([lo, hi]);
        false
    }

    pub(super) fn jsr(&mut self) -> bool {
        let pc = self.pc;
        let addr = self.load_u16_bump_pc();
        self.push_u16(pc - 1); // push curr pc-1 to the stack
        self.pc = addr;
        false
    }

    pub(super) fn rts(&mut self) -> bool {
        let pos = self.pull_u16() + 1;
        self.pc = pos;
        false
    }

    pub(super) fn brk(&mut self) -> bool {
        let pc = self.pc;
        self.push_u16(pc + 1);
        self.push_u8(self.p);
        self.flag_on(flags::IRQ);
        self.pc = self.load_u16(BRK_VECTOR);
        false
    }

    pub(super) fn rti(&mut self) -> bool {
        let flags = self.pull_u8();
        self.p = (self.p | 0x30) - 0x10;
        self.pc = self.pull_u16();
        false
    }
}

// branches
// All branches are relative mode and have a length of two bytes
// branching ops do not affect any flag, but they dpend on flag states.
// Add one if the branch is taken and add one more if the branch crosses a page boundary
// comeback: change the return type of opcode functions from bool to int because of cases like the above
impl Six502 {
    pub(super) fn bpl(&mut self) -> bool {
        let v = self.load_u8_bump_pc();
        if !self.is_flag_set(flags::NEGATIVE) {
            self.pc = self.pc.wrapping_add(v as u16);
        }
        false
    }

    pub(super) fn bmi(&mut self) -> bool {
        let v = self.load_u8_bump_pc();
        if self.is_flag_set(flags::NEGATIVE) {
            self.pc = self.pc.wrapping_add(v as u16);
        }
        false
    }

    pub(super) fn bvc(&mut self) -> bool {
        let v = self.load_u8_bump_pc();
        if !self.is_flag_set(flags::OVERFLOW) {
            self.pc = self.pc.wrapping_add(v as u16);
        }
        false
    }

    pub(super) fn bvs(&mut self) -> bool {
        let v = self.load_u8_bump_pc();
        if self.is_flag_set(flags::OVERFLOW) {
            self.pc = self.pc.wrapping_add(v as u16);
        }
        false
    }

    pub(super) fn bcc(&mut self) -> bool {
        let v = self.load_u8_bump_pc();
        if !self.is_flag_set(flags::CARRY) {
            self.pc = self.pc.wrapping_add(v as u16);
        }
        false
    }

    pub(super) fn bcs(&mut self) -> bool {
        let v = self.load_u8_bump_pc();
        if self.is_flag_set(flags::CARRY) {
            self.pc = self.pc.wrapping_add(v as u16);
        }
        false
    }

    pub(super) fn bne(&mut self) -> bool {
        let v = self.load_u8_bump_pc();

        if !self.is_flag_set(flags::ZERO) {
            self.pc = self.pc.wrapping_add(v as u16);
        }
        false
    }

    pub(super) fn beq(&mut self) -> bool {
        let v = self.load_u8_bump_pc();

        if self.is_flag_set(flags::ZERO) {
            self.pc = self.pc.wrapping_add(v as u16);
        }
        false
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
    pub(super) fn clc(&mut self) -> bool {
        self.flag_off(flags::CARRY);
        false
    }
    pub(super) fn sec(&mut self) -> bool {
        self.flag_on(flags::CARRY);
        false
    }
    pub(super) fn cli(&mut self) -> bool {
        self.flag_off(flags::IRQ);
        false
    }
    pub(super) fn sei(&mut self) -> bool {
        self.flag_on(flags::IRQ);
        false
    }
    pub(super) fn clv(&mut self) -> bool {
        self.flag_off(flags::OVERFLOW);
        false
    }
    pub(super) fn cld(&mut self) -> bool {
        self.flag_off(flags::DECIMAL);
        false
    }
    pub(super) fn sed(&mut self) -> bool {
        self.flag_on(flags::DECIMAL);
        false
    }
}

// system functions
impl Six502 {}
