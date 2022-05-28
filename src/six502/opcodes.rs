use super::util::check_overflow;
use super::{addressing::AddressingMode, flags, Six502};
use super::{IRQ_VECTOR, NMI_VECTOR, RESET_VECTOR};
use crate::bus::{ByteAccess, WordAccess};
use std::ops::{BitAnd, BitOr, BitOrAssign, Shl, Shr};

const BRK: u16 = 0xfffe;

// load/store ops
impl Six502 {
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
    /// CMP - Compare Memory and Accumulator.
    /// subtracts the contents of memory from the contents of the accumulator.
    /// It sets flags as if a subtraction had been carried out between the accumulator and the operand
    /// Immediate; Zero Page; Zero Page,X; Absolute; Absolute,X; Absolute,Y; (Indirect,X); (Indirect) ,Y
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

    /// BIT - Test Bits in Memory with Accumulator
    /// performs an AND between a memory location and the accumulator but does not store the result of the AND into the accumulator.
    /// affects Z, N, and O
    pub(super) fn bit(&mut self, mode: AddressingMode) -> bool {
        let a = self.a;
        let (b, carry) = mode.load(self);
        self.assert_flag(flags::ZERO, a & b == 0);
        self.assert_flag(flags::NEGATIVE, b & 0x80 != 0);
        self.assert_flag(flags::OVERFLOW, b & 0b01000000 != 0);
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
    /// The AND instruction performs a bit-by-bit AND operation and stores the result back in the accumulator
    /// Addressing modes: Immediate; Absolute; Zero Page; Absolute,X; Absolute,Y; Zero Page,X; Indexed Indirect; and Indirect Indexed.
    // affects z and n flags
    pub(super) fn and(&mut self, mode: AddressingMode) -> bool {
        let (v, cross) = mode.load(self);
        self.a &= v;
        self.update_z(self.a);
        self.update_n(self.a);
        cross
    }

    /// The OR instruction performs a bit-by-bit OR operation and stores the result back in the accumulator
    /// Addressing modes: Immediate; Absolute; Zero Page; Absolute,X; Absolute,Y; Zero Page,X; Indexed Indirect; and Indirect Indexed.
    // affects z and n flags
    pub(super) fn ora(&mut self, mode: AddressingMode) -> bool {
        let (v, cross) = mode.load(self);
        self.a |= v;
        self.update_z(self.a);
        self.update_n(self.a);
        cross
    }
    /// The XOR instruction performs a bit-by-bit XOR operation and stores the result back in the accumulator
    /// Addressing modes: Immediate; Absolute; Zero Page; Absolute,X; Absolute,Y; Zero Page,X; Indexed Indirect; and Indirect Indexed.
    // affects z and n flags
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
    /// A + M + C -> A.
    /// addressing modes: Immediate; Absolute; Zero Page; Absolute,X; Absolute,Y; Zero Page,X; indexed Indirect; and Indirect Indexed
    /// Example of unsigned arithmetic (Here, A refers to the accumulator, and M refers to the contents of the selected memory)
    /// ```              
    ///                  0000   1101     13 = (A)*
    ///                  1101   0011    211 = (M)*
    ///                            1      1 = CARRY
    /// Carry  = /0/     1110   0001    225 = (A)
    /// ```
    pub(super) fn adc(&mut self, mode: AddressingMode) -> bool {
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
        cross
    }

    /// subtracts the value of memory and borrow from the value of the accumulator, using two's complement arithmetic, and stores the result in the accumulator
    ///  Borrow is defined as the carry flag complemented
    /// A - M - C -> A.
    /// It has addressing modes Immediate; Absolute; Zero Page; Absolute,X; Absolute,Y; Zero Page,X; Indexed Indirect; and Indirect Indexed.
    /// e.g. if if A = 5, and M = 3, and were to find `5-3`, we do: `5 + (-3)`, as follows
    /// two's compliment conversion first
    ///```
    ///          M = 3    0000   0011
    /// Complemented M    1111   1100
    ///      Add C = 1              1
    ///        -M = -3    1111   1101
    /// then the addition
    ///           A = 5    0000   0101
    ///     Add -M = -3    1111   1101
    ///      Carry = /1/   0000   0010 = +2
    ///```
    pub(super) fn sbc(&mut self, mode: AddressingMode) -> bool {
        let a = u16::from(self.a);
        let (v, cross) = mode.load(self);
        // for single precision sub, the programmer has to set the carry to 1 before using the sbc op, so it will be a valid
        let twos_comp = if self.is_flag_set(flags::CARRY) {
            u16::from(!v) + 1
        } else {
            u16::from(!v)
        };
        let res = a + twos_comp;
        self.assert_flag(flags::CARRY, res > u16::from(u8::MAX));

        // The overflow flag is set when the result exceeds +127 or -127, otherwise it is reset.
        self.assert_flag(flags::OVERFLOW, check_overflow(a as u8, v, res as u8));

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

    /// jump with absolute addressing
    /// basically loads a new address into the pc unconditionally
    /// the cpu knows to always load the next instruction address from the pc
    pub(super) fn jmp(&mut self) -> bool {
        self.pc = self.load_u16_bump_pc();
        false
    }

    // the other version of jump, but with indirect addressing
    pub(super) fn jmp_indirect(&mut self) -> bool {
        let pc = self.load_u16_bump_pc();
        let lo = self.load_u8(pc);
        let hi = self.load_u8((pc & 0xff00) | ((pc + 1) & 0x00ff));
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

    // comeback
    // BRK initiates a software interrupt similar to a hardware interrupt (IRQ)
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
// All branches are relative mode and have a length of two bytes. branching ops do not affect any flag, but they depend on flag states.
// Add add one more cycle if the branch crosses a page boundary
//  To perform a conditional change of sequence, the microprocessor must interpret the instruction, test the value of a flag, and then change the P counter if the value
// agrees with the instruction.
// If the condition is not met, the program counter continues to increment in its normal fashion, i.e. sequentially.
// Notice that we use relative addressing in branching.
// This is to reduce the number of bytes needed for branching instructions, in effect reducing cpu load.
// In relative addressing, we add the value in the memory location following the OPCODE to the program counter.  This allows us to
// specify a new program counter location with only two bytes, one for the OPCODE and one for the value to be added.
impl Six502 {
    pub fn branch(&mut self, flag: u8, cond: bool) -> bool {
        // relative addressing. load just one byte.
        // casting the u8 as an i8, and from tere to u16 helps create the twos compliment of the number with length 16bits
        let v = self.load_u8_bump_pc() as i8 as u16;
        let old_pc = self.pc;
        if cond {
            if self.is_flag_set(flag) {
                self.pc = self.pc.wrapping_add(v);
            }
        } else {
            if !self.is_flag_set(flag) {
                self.pc = self.pc.wrapping_add(v);
            }
        }
        if (self.pc & 0xff00) != (old_pc & 0xff00) {
            // crossed page boundary
            true
        } else {
            false
        }
    }

    /// BPL - Branch on Result Plus
    pub(super) fn bpl(&mut self) -> bool {
        self.branch(flags::NEGATIVE, false)
    }

    ///  BMI - Branch on Result Minus
    pub(super) fn bmi(&mut self) -> bool {
        self.branch(flags::NEGATIVE, true)
    }

    /// BVC - Branch on Overflow Clear
    pub(super) fn bvc(&mut self) -> bool {
        self.branch(flags::OVERFLOW, false)
    }

    /// BVS - Branch on Overflow Set
    pub(super) fn bvs(&mut self) -> bool {
        self.branch(flags::OVERFLOW, true)
    }

    ///  BCC - Branch on Carry Clear
    pub(super) fn bcc(&mut self) -> bool {
        self.branch(flags::CARRY, false)
    }

    /// BCS - Branch on Carry Set
    pub(super) fn bcs(&mut self) -> bool {
        self.branch(flags::CARRY, true)
    }

    //  BNE - Branch on Result Not Zero
    pub(super) fn bne(&mut self) -> bool {
        self.branch(flags::ZERO, false)
    }

    /// BEQ - Branch on Result Zero
    pub(super) fn beq(&mut self) -> bool {
        self.branch(flags::ZERO, true)
    }
}

// status flag changes
// none of these ops have side effect of affecting other flags
impl Six502 {
    /// resets the carry flag to a 0
    pub(super) fn clc(&mut self) -> bool {
        self.clear_flag(flags::CARRY);
        false
    }

    /// This instruction initializes the carry flag to a 1
    pub(super) fn sec(&mut self) -> bool {
        self.set_flag(flags::CARRY);
        false
    }
    /// cli resets interrupt disable to a 0
    pub(super) fn cli(&mut self) -> bool {
        self.clear_flag(flags::IRQ);
        false
    }
    /// sei sets the interrupt disable flag (IRQ) to a 1
    pub(super) fn sei(&mut self) -> bool {
        self.set_flag(flags::IRQ);
        false
    }
    /// clears the overflow flag to a 0
    /// used in conjunction with the set overflow pin which can change the state of the overflow flag with an external signal
    // comeback to implement pins, incl this set overflow pin
    pub(super) fn clv(&mut self) -> bool {
        self.clear_flag(flags::OVERFLOW);
        false
    }
    // cld resets the decimal mode flag D to a 1
    pub(super) fn cld(&mut self) -> bool {
        self.clear_flag(flags::DECIMAL);
        false
    }
    // sed sets the decimal mode flag D to a 1
    pub(super) fn sed(&mut self) -> bool {
        self.set_flag(flags::DECIMAL);
        false
    }
}
