use super::{addressing::AddressingMode, flags, Six502};
use super::{IRQ_VECTOR, NMI_VECTOR, RESET_VECTOR};
use crate::bus::{ByteAccess, WordAccess};
use std::ops::{BitAnd, BitOr, BitOrAssign, Shl, Shr};

// load/store ops
impl Six502 {
    const BRK: u16 = 0xfffe;

    /// load accumulator with memory. data is transferred from memory into the accumulator
    /// zero flag is set if the acc is zero, otherwise resets
    //  negative flag is set if bit 7 of the accumulator is a 1, otherwise resets
    // address modes: Immediate; Absolute; Zero Page; Absolute,X; Absolute,Y; Zero Page,X; Indexed Indirect; and Indirect Indexed
    pub(super) fn lda(&mut self, mode: AddressingMode) -> bool {
        let (v, cross) = mode.load(self);
        self.a = v;
        self.update_z(v);
        self.update_n(v);
        cross
    }

    pub(super) fn ldx(&mut self, mode: AddressingMode) -> bool {
        let (v, cross) = mode.load(self);
        self.x = v;
        self.update_z(v);
        self.update_n(v);
        cross
    }

    pub(super) fn ldy(&mut self, mode: AddressingMode) -> bool {
        let (v, cross) = mode.load(self);
        self.y = v;
        self.update_z(v);
        self.update_n(v);
        cross
    }

    // transfers the contents of the accumulator to memory.
    // possible address modes: Absolute; Zero Page; Absolute,X; Absolute,Y; Zero Page,X; Indexed Indirect; and Indirect Indexed
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

// comparisons
impl Six502 {
    /// cmp: compare accumulator. It sets flags as if a subtraction had been carried out between the accumulator and the operand
    pub(super) fn cmp(&mut self, mode: AddressingMode) -> bool {
        let (v, cross) = mode.load(self);
        let v = v as u16;
        let a = self.a as u16;
        self.assert_flag(flags::CARRY, a >= v);
        self.update_z((a - v) as u8);
        self.update_n((a - v) as u8);
        cross
    }

    /// cpx: compare accumulator. It sets flags as if a subtraction had been carried out between the x register and the operand
    pub(super) fn cpx(&mut self, mode: AddressingMode) -> bool {
        let (v, cross) = mode.load(self);
        let v = v as u16;
        let x = self.x as u16;
        self.assert_flag(flags::CARRY, x >= v);

        self.update_z((x - v) as u8);
        self.update_n((x - v) as u8);
        cross
    }

    /// cpy: compare accumulator. It sets flags as if a subtraction had been carried out between the y register and the operand

    pub(super) fn cpy(&mut self, mode: AddressingMode) -> bool {
        let (v, cross) = mode.load(self);
        let v = v as u16;
        let y = self.y as u16;
        self.assert_flag(flags::CARRY, y >= v);
        self.update_z((y - v) as u8);
        self.update_n((y - v) as u8);
        cross
    }

    pub(super) fn bit(&mut self, mode: AddressingMode) -> bool {
        let a = self.a;
        let (b, carry) = mode.load(self);
        self.assert_flag(flags::ZERO, a & b == 0);
        self.assert_flag(flags::NEGATIVE, b & 0x80 != 0);
        self.assert_flag(flags::OVERFLOW, b & 0x40 != 0);
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
        self.set_flag(val);
        self.clear_flag(flags::BREAK);
        self.set_flag(flags::UNUSED);
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
// The MCS650X has an 8-bit arithmetic unit
// arithmetic ops are performed using the accumulator as temporary storage.
// parts involved are: the acc; the ALU; the processor status register (or flags), p; the memory
// In unsigned arithmetic, we need to watch the carry flag to detect errors. The overflow flag is not useful for unsigned ops
// In signed arithmetic, we need to watch the overflow flag to detect errors. The sign flag is not useful for signed ops
// the programmer makes this decision basd on what they want. the cpu knows nothing about their intents. it justs sets the flag accordingly
impl Six502 {
    /// Add Memory to Accumulator with Carry
    /// This instruction adds the value of memory and carry from the previous operation to the value of the accumulator and stores the
    /// result in the accumulator.
    ///  A + M + C -> A.
    /// addressing modes: Immediate; Absolute; Zero Page; Absolute,X; Absolute,Y; Zero Page,X; indexed Indirect; and Indirect Indexed
    /// Example of unsigned arithmetic (Here, A refers to the accumulator, and M refers to the contents of the selected memory)
    ///                  0000   1101     13 = (A)*
    ///                  1101   0011    211 = (M)*
    ///                            1      1 = CARRY
    /// Carry  = /0/     1110   0001    225 = (A)
    pub(super) fn adc(&mut self, mode: AddressingMode) -> u8 {
        // convert to u16 because we want to be able to know the 9th bit
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

        // If we add any 2 numbers which result in a sum which is greater than 255, we represent the result with a ninth bit plus the 8 bits of the excess
        // over 255.  The ninth bit is called "carry."
        // two cases where it will be on:
        //1.     1111 + 0001 = 0000 => carry flag is turned on.
        //2.     0000 - 0001 = 1111 => carry flag is turned on.
        self.assert_flag(flags::CARRY, res & 0x100 != 0);

        self.assert_flag(flags::OVERFLOW, check_overflow(a as u8, b as u8, res as u8));
        self.a = res as u8;
        let a = self.a;
        self.update_z(a);
        self.update_n(a);
        if cross {
            1
        } else {
            0
        }
    }

    /// sbc subtracts a value and the inverse of the carry bit from the accumulator.
    pub(super) fn sbc(&mut self, mode: AddressingMode) -> bool {
        let a = u16::from(self.a);
        let (v, cross) = mode.load(self);
        let b = v as u16;
        let res = if self.is_flag_set(flags::CARRY) {
            // sub has to wrap arround for the carry check to be useful.
            a.wrapping_sub(b).wrapping_sub(1)
        } else {
            a.wrapping_sub(b)
        };

        self.assert_flag(flags::CARRY, res & 0x100 == 0);

        self.assert_flag(flags::OVERFLOW, check_overflow(a as u8, b as u8, res as u8));

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
        let mut res: u8 = b.shl(1);
        if self.is_flag_set(flags::CARRY) {
            res.bitor_assign(1);
        }
        self.assert_flag(flags::CARRY, b & 0x80 != 0);

        self.update_z(res);
        self.update_n(res);
        mode.store(self, res);
        cross
    }

    pub(super) fn asl(&mut self, mode: AddressingMode) -> bool {
        let (b, cross) = mode.load(self);
        let mut res: u8 = b.shl(1);
        self.assert_flag(flags::CARRY, b & 0x80 != 0);

        self.update_z(res);
        self.update_n(res);
        mode.store(self, res);
        cross
    }

    pub(super) fn ror(&mut self, mode: AddressingMode) -> bool {
        let (b, cross) = mode.load(self);
        let mut res: u8 = b.shr(1);
        if self.is_flag_set(flags::CARRY) {
            res.bitor_assign(0x80);
        }
        self.assert_flag(flags::CARRY, (b & 0x1) != 0);
        self.update_z(res);
        self.update_n(res);
        mode.store(self, res);
        cross
    }

    pub(super) fn lsr(&mut self, mode: AddressingMode) -> bool {
        let (b, cross) = mode.load(self);
        let mut res = b.shr(1);
        self.assert_flag(flags::CARRY, (b & 0x1) != 0);
        self.update_z(res);
        self.update_n(res);
        mode.store(self, res);
        cross
    }
}

// jumps and calls
impl Six502 {
    const BRK_VECTOR: u16 = 0xfffe;

    // jump with absolute addressing
    pub(super) fn jmp(&mut self) -> bool {
        self.pc = self.load_u16_bump_pc();
        false
    }

    // the other version of jump, but with indirect addressing
    pub(super) fn jmp_indirect(&mut self) -> bool {
        let op = self.load_u16_bump_pc();
        let lo = self.load_u8(op);
        let hi = self.load_u8((op & 0xff00) | ((op + 1) & 0x00ff));
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
        self.set_flag(flags::IRQ);
        self.pc = self.load_u16(BRK);
        false
    }

    // retrieves the Processor Status Word (flags) and the Program Counter from the stack in that order
    pub(super) fn rti(&mut self) -> bool {
        let flags = self.pull_u8(); // pop the cpu flags from the stack
        self.clear_flag(flags::BREAK);
        self.set_flag(flags::UNUSED);
        // then pop the 16-bit pc from the stack
        self.pc = self.pull_u16();
        false
    }
}

// branches
// All branches are relative mode and have a length of two bytes
// branching ops do not affect any flag, but they depend on flag states.
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
    pub(super) fn clc(&mut self) -> bool {
        self.clear_flag(flags::CARRY);
        false
    }
    pub(super) fn sec(&mut self) -> bool {
        self.set_flag(flags::CARRY);
        false
    }
    pub(super) fn cli(&mut self) -> bool {
        self.clear_flag(flags::IRQ);
        false
    }
    pub(super) fn sei(&mut self) -> bool {
        self.set_flag(flags::IRQ);
        false
    }
    pub(super) fn clv(&mut self) -> bool {
        self.clear_flag(flags::OVERFLOW);
        false
    }
    pub(super) fn cld(&mut self) -> bool {
        self.clear_flag(flags::DECIMAL);
        false
    }
    pub(super) fn sed(&mut self) -> bool {
        self.set_flag(flags::DECIMAL);
        false
    }
}

// system functions
impl Six502 {}
