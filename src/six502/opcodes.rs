use super::six502::Six502;
use super::util::check_overflow;
use super::vectors::{self, IRQ, NMI};
use super::{addressing::AddressingMode, flags};
use crate::{ByteAccess, Addressing};
use super::WordAccess;
use crate::macros::impl_addr_modes;
use crate::Cpu;
use std::marker::PhantomData;
use std::ops::{BitAnd, BitOr, BitOrAssign, Shl, Shr};

const BRK: u16 = 0xfffe;

// load/store ops
impl Six502 {
    /// load accumulator with memory. data is transferred from memory into the accumulator
    /// zero flag is set if the acc is zero, otherwise resets
    //  negative flag is set if bit 7 of the accumulator is a 1, otherwise resets
    // address modes: Immediate; Absolute; Zero Page; Absolute,X; Absolute,Y; Zero Page,X; Indexed Indirect; and Indirect Indexed
    pub(super) fn lda(&mut self, mode: AddressingMode) {
        let v = self.dispatch_load(mode);
        self.a = v;
        self.update_zn_flags(v);
    }

    ///  Load the index register X from memory.
    pub(super) fn ldx(&mut self, mode: AddressingMode) {
        let v = self.dispatch_load(mode);
        self.x = v;
        self.update_zn_flags(v);
    }

    ///  Load the index register Y from memory.
    pub(super) fn ldy(&mut self, mode: AddressingMode) {
        let v = self.dispatch_load(mode);
        self.y = v;
        self.update_zn_flags(v);
    }

    // transfers the contents of the accumulator to memory.
    // possible address modes: Absolute; Zero Page; Absolute,X; Absolute,Y; Zero Page,X; Indexed Indirect; and Indirect Indexed
    pub(super) fn sta(&mut self, mode: AddressingMode) {
        self.dispatch_store(self.a, mode);
    }

    /// Transfers value of X register to addressed memory location. affects no flag
    pub(super) fn stx(&mut self, mode: AddressingMode) {
        self.dispatch_store(self.x, mode);
    }

    /// Transfers value of Y register to addressed memory location. affects no flag
    pub(super) fn sty(&mut self, mode: AddressingMode) {
        self.dispatch_store(self.y, mode);
    }
}

// comparisons
impl Six502 {
    // util for compare operations
    // reg is the register the value v (loaded from memory) will be subtracted from.
    fn compare(&mut self, reg: u8, v: u8) {
        let v = v as u16;
        let reg = reg as u16;
        // causes the carry to be set on if the absolute value of the index register X is equal to or greater than the data from memory.
        self.assert_flag(flags::CARRY, reg >= v);
        self.update_zn_flags(reg.wrapping_sub(v) as u8);
    }
    /// CMP - Compare Memory and Accumulator.
    /// subtracts the contents of memory from the contents of the accumulator.
    /// It sets flags as if a subtraction had been carried out between the accumulator and the operand
    /// Immediate; Zero Page; Zero Page,X; Absolute; Absolute,X; Absolute,Y; (Indirect,X); (Indirect) ,Y
    pub(super) fn cmp(&mut self, mode: AddressingMode) {
        let v = self.dispatch_load(mode);
        self.compare(self.a, v);
    }

    /// cpx: compare accumulator. It sets flags as if a subtraction had been carried out between the x register and the operand
    pub(super) fn cpx(&mut self, mode: AddressingMode) {
        let v = self.dispatch_load(mode);
        self.compare(self.x, v);
    }

    /// cpy: compare accumulator. It sets flags as if a subtraction had been carried out between the y register and the operand
    pub(super) fn cpy(&mut self, mode: AddressingMode) {
        let v = self.dispatch_load(mode);
        self.compare(self.y, v);
    }

    /// BIT - Test Bits in Memory with Accumulator
    /// performs an AND between a memory location and the accumulator but does not store the result of the AND into the accumulator.
    /// affects Z, N, and O
    pub(super) fn bit(&mut self, mode: AddressingMode) {
        let a = self.a;
        let b = self.dispatch_load(mode);
        self.assert_flag(flags::ZERO, a & b == 0);
        self.assert_flag(flags::NEGATIVE, b & 0x80 != 0);
        self.assert_flag(flags::OVERFLOW, b & 0b01000000 != 0);
    }
}

// register transfers
// these ops make use of implied addressing, and are one byte instructions
impl Six502 {
    /// tax transfers accumulator into x register, updating the z and n flags based on the value of a
    pub(super) fn tax(&mut self, _mode: AddressingMode) {
        self.x = self.a;
        self.update_zn_flags(self.a);
    }

    /// moves the value that is in the index register X to the accumulator A without disturbing the content of the index register X.
    /// affects Z, N
    pub(super) fn txa(&mut self, _mode: AddressingMode) {
        self.a = self.x;
        self.update_zn_flags(self.x);
    }

    ///  moves the value of the accumulator into index register Y without affecting the accumulator. affects Z, N
    pub(super) fn tay(&mut self, _mode: AddressingMode) {
        self.y = self.a;
        self.update_zn_flags(self.a);
    }

    // moves the value that is in the index register Y to accumulator A without disturbing the content of the register Y. affects Z, N
    pub(super) fn tya(&mut self, _mode: AddressingMode) {
        self.a = self.y;
        self.update_zn_flags(self.y);
    }

    /// tsx: Transfer Stack ptr to X, affects Z, N
    pub(super) fn tsx(&mut self, _mode: AddressingMode) {
        self.x = self.s;
        self.update_zn_flags(self.s);
    }

    /// txs: transfer x register to stack pointer
    pub(super) fn txs(&mut self, _mode: AddressingMode) {
        self.s = self.x;
        self.update_zn_flags(self.x);
    }
}

// stack ops
// single byte instructions. addressing mode implied
impl Six502 {
    /// transfers the current value of the accumulator the next location on the stack, automatically decrementing the stack to
    /// point to the next empty location.
    pub(super) fn pha(&mut self, _mode: AddressingMode) {
        self.push_u8(self.a);
    }

    /// adds 1 to the current value of the stack pointer and uses it to address the stack and loads the contents of the stack
    /// into the A register.
    pub(super) fn pla(&mut self, _mode: AddressingMode) {
        let v = self.pull_u8();
        self.update_zn_flags(v);
        self.a = v;
    }

    /// push processor status on stack
    pub(super) fn php(&mut self, _mode: AddressingMode) {
        let flags = self.p;
        // php sets both Break for th flag pushed onto the stack
        self.push_u8(flags | flags::BREAK);
    }

    /// plp pulls processor status
    /// transfers the next value on the stack to the Processor Status register, thereby changing all of the flags and
    /// setting the mode switches to the values from the stack.
    pub(super) fn plp(&mut self, _mode: AddressingMode) {
        let val = self.pull_u8();
        // set all the flags except the break flag, which remains as it was
        self.p = val & (self.p & flags::BREAK);
    }
}

// logical ops
impl Six502 {
    /// The AND instruction performs a bit-by-bit AND operation and stores the result back in the accumulator
    /// Addressing modes: Immediate; Absolute; Zero Page; Absolute,X; Absolute,Y; Zero Page,X; Indexed Indirect; and Indirect Indexed.
    // affects z and n flags
    pub(super) fn and(&mut self, mode: AddressingMode) {
        let v = self.dispatch_load(mode);
        self.a &= v;
        self.update_zn_flags(self.a);
    }

    /// The OR instruction performs a bit-by-bit OR operation and stores the result back in the accumulator
    /// Addressing modes: Immediate; Absolute; Zero Page; Absolute,X; Absolute,Y; Zero Page,X; Indexed Indirect; and Indirect Indexed.
    // affects z and n flags
    pub(super) fn ora(&mut self, mode: AddressingMode) {
        let v = self.dispatch_load(mode);
        self.a |= v;
        self.update_zn_flags(self.a);
    }
    /// The XOR instruction performs a bit-by-bit XOR operation and stores the result back in the accumulator
    /// Addressing modes: Immediate; Absolute; Zero Page; Absolute,X; Absolute,Y; Zero Page,X; Indexed Indirect; and Indirect Indexed.
    // affects z and n flags
    pub(super) fn eor(&mut self, mode: AddressingMode) {
        let v = self.dispatch_load(mode);
        self.a ^= v;
        self.update_zn_flags(self.a);
    }
}

// Arithmetic ops
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
    ///           
    ///                  0000   1101     13 = (A)*
    ///                  1101   0011    211 = (M)*
    ///                            1      1 = CARRY
    /// Carry  = /0/     1110   0001    225 = (A)
    pub(super) fn adc(&mut self, mode: AddressingMode) {
        // convert to u16 because we want to be able to know the 9th bit
        let a = u16::from(self.a);
        let v = self.dispatch_load(mode);
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
        self.update_zn_flags(a);
    }

    /// subtracts the value of memory and borrow from the value of the accumulator, using two's complement arithmetic, and stores the result in the accumulator
    ///  Borrow is defined as the carry flag complemented
    /// A - M - C -> A.
    /// It has addressing modes Immediate; Absolute; Zero Page; Absolute,X; Absolute,Y; Zero Page,X; Indexed Indirect; and Indirect Indexed.
    /// e.g. if if A = 5, and M = 3, and were to find `5-3`, we do: `5 + (-3)`, as follows
    /// two's compliment conversion first
    ///
    ///          M = 3    0000   0011
    /// Complemented M    1111   1100
    ///      Add C = 1              1
    ///        -M = -3    1111   1101
    /// then the addition
    ///           A = 5    0000   0101
    ///     Add -M = -3    1111   1101
    ///      Carry = /1/   0000   0010 = +2
    ///
    pub(super) fn sbc(&mut self, mode: AddressingMode) {
        let mut a = u16::from(self.a);
        let v = self.dispatch_load(mode);
        let (acc, mem )= (self.a, v);
        let mut v = v as u16;
        // for single precision sub (or the first sub in a multi-precision sub), the programmer has to set the carry to 1 before using the sbc op, to indicate that a 
        // borrow will not occur beacuse the compliment of the CARRY indicates a borrow.
        if !self.is_flag_set(flags::CARRY) { 
            v += 1;
        }
        // get twos compliment
        let twos_comp = !v + 1;

        // twos compliment addition
        let overflow;
        (a, overflow) = a.overflowing_add(twos_comp);

        self.assert_flag(flags::CARRY, overflow);
        let a = a as u8;
        // The overflow flag is set when the result exceeds +127 or -127, otherwise it is reset.
        self.assert_flag(flags::OVERFLOW, check_overflow(acc, mem, a));

        self.a = a;
        self.update_zn_flags(a);
    }


    // comeback
    pub(super) fn dec_adc(&mut self, mode: AddressingMode) {
        self.clc(mode); // clear carry flag
        self.sed(mode); // set decimal mode
        self.lda(mode);
        self.adc(mode);
        self.sta(mode);
    }

    // comeback
    pub(super) fn dec_sbc(&mut self, mode: AddressingMode) {
        self.clc(mode); // clear carry flag
        self.sed(mode); // set decimal mode
        self.lda(mode);
        self.sbc(mode);
        self.sta(mode);
    }

   
}

//incrs and decrs
impl Six502 {
    pub(super) fn inc(&mut self, mode: AddressingMode) {
        let v = self.dispatch_load(mode);
        let v = v.wrapping_add(1);
        self.update_zn_flags(v);
        self.dispatch_store(v, mode);
    }

    pub(super) fn dec(&mut self, mode: AddressingMode) {
        let v = self.dispatch_load(mode);
        let v = v.wrapping_sub(1);
        self.update_zn_flags(v);
        self.dispatch_store(v, mode);
    }

    ///   Increment X adds 1 to the current value of the X register.
    pub(super) fn inx(&mut self, _mode: AddressingMode) {
        let x = self.x.wrapping_add(1);
        self.update_zn_flags(x);
        self.x = x;
    }

    /// subtracts one from the current value of the index register X and stores the result in the index register X
    pub(super) fn dex(&mut self, _mode: AddressingMode) {
        let x = self.x.wrapping_sub(1);
        self.update_zn_flags(x);
        self.x = x;
    }

    ///   Increment Y adds 1 to the current value of the Y register.
    pub(super) fn iny(&mut self, _mode: AddressingMode) {
        let y = self.y.wrapping_add(1);
        self.update_zn_flags(y);
        self.y = y;
    }

    ///  subtracts one from the current value in the index register Y and stores the result into the index register y
    pub(super) fn dey(&mut self, _mode: AddressingMode) {
        let y = self.y.wrapping_sub(1);
        self.update_zn_flags(y);
        self.y = y;
    }
}



// shifts
impl Six502 {
    pub(super) fn rol(&mut self, mode: AddressingMode) {
        let b= self.dispatch_load(mode);
        let mut res: u8 = b.shl(1);
        if self.is_flag_set(flags::CARRY) {
            res.bitor_assign(1);
        }
        self.assert_flag(flags::CARRY, b & 0x80 != 0);

        self.update_zn_flags(res);
        self.dispatch_store( res, mode);
    }

    pub(super) fn asl(&mut self, mode: AddressingMode) {
        let b= self.dispatch_load(mode);
        let res: u8 = b.shl(1);
        self.assert_flag(flags::CARRY, b & 0x80 != 0);

        self.update_zn_flags(res);
        self.dispatch_store( res, mode);
    }

    pub(super) fn ror(&mut self, mode: AddressingMode) {
        let b= self.dispatch_load(mode);
        let mut res: u8 = b.shr(1);
        if self.is_flag_set(flags::CARRY) {
            res.bitor_assign(0x80);
        }
        self.assert_flag(flags::CARRY, (b & 0x1) != 0);
        self.update_zn_flags(res);
        self.dispatch_store( res, mode);
    }

    pub(super) fn lsr(&mut self, mode: AddressingMode) {
        let b= self.dispatch_load(mode);
        let res = b.shr(1);
        self.assert_flag(flags::CARRY, (b & 0x1) != 0);
        self.update_zn_flags(res);
        self.dispatch_store( res, mode);
    }
}

/// jumps and calls
impl Six502 {
    const BRK_VECTOR: u16 = 0xfffe;

    /// **Jump** with absolute addressing
    /// basically loads a new address into the pc unconditionally
    /// the cpu knows to always load the next instruction address from the pc
    ///  comeback: 3 or 4 clock cycles: seacrh kim1-6502 for 'The jump absolute therefore only requires 3 cycles.'
    pub(super) fn jmp(&mut self, _mode: AddressingMode) {
        self.pc = self.load_u16_bump_pc();
    }

    /// The other version of jump, but with indirect addressing
    pub(super) fn jmp_indirect(&mut self, _mode: AddressingMode) {
        let pc = self.load_u16_bump_pc();
        let lo = self.load_u8();
        self.addr_bus = (pc & 0xff00) | ((pc + 1) & 0x00ff);
        let hi = self.load_u8();
        self.pc = u16::from_le_bytes([lo, hi]);
    }

    /// jump to subroutine
    /// transfers control of the program counter to a sub- routine location but leaves a return pointer on the stack to allow the
    /// user to return to perform the next instruction in the main program after the subroutine is complete
    pub(super) fn jsr(&mut self, _mode: AddressingMode) {
        let pc = self.pc;
        let addr = self.load_u16_bump_pc();
        self.push_u16(pc - 1); // push curr pc-1 to the stack
        self.pc = addr;
    }

    /// loads the program Count low and program count high
    /// from the stack into the program counter and increments the program Counter
    ///  so that it points to the instruction following the JSR
    pub(super) fn rts(&mut self, _mode: AddressingMode) {
        let pos = self.pull_u16() + 1;
        self.pc = pos;
    }

    // BRK initiates a software interrupt similar to a hardware interrupt (IRQ)
    pub(super) fn brk(&mut self, _mode: AddressingMode) {
        self.push_u16(self.pc + 1); //Increase program counter by 1 before pusing on stack so computation returns to the correct place on RTI
                                    // push status register with break bits set
        self.push_u8(self.p | 0b00110000);
        // set interrupt disable flag
        self.set_flag(flags::IRQ);
        // set the pc to the IRQ vector
        self.addr_bus = vectors::IRQ;
        self.pc = self.load_u16();
        // implied addressing takes two cycles. the remaining operation taes 5
    }

    /// retrieves the Processor Status Word (flags) and the Program Counter from the stack in that order
    /// The status register is pulled with the break flag and bit 5 ignored. Then PC is pulled from the stack.
    /// All interrupts end with an RTI
    /// Because the interrupt disable had to be off for an interrupt request to have been honored,
    /// the return from interrupt which loads the processor status from before the interrupt occured has the effect of
    /// clearing the interrupt disable bit.
    /// There is no automatic save of any of the other registers in the microprocessor.  Because the interrupt occurred to allow data to be trans-
    /// ferred using the microprocessor, the programmer must save the various internal registers at the time the interrupt is taken
    /// and restore them prior to returning from the interrupt. This is done on the stack
    pub(super) fn rti(&mut self, _mode: AddressingMode) {
        let flags = self.pull_u8(); // pop the cpu flags from the stack
                                    // set flag
        self.set_flag(flags);
        // ignore break flag
        self.clear_flag(flags::BREAK);
        // inore unused
        self.clear_flag(flags::UNUSED);
        // then pop the 16-bit pc from the stack
        self.pc = self.pull_u16();
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
    /// base routine for branching. cond parameter states that you wan the flag to be either set/unset
    /// If a branch is normally not taken, assume 2 cycles for the branch.
    /// If the branch is normally taken but it does not across the page boundary, assume 3 cycles for the branch.
    /// If the branch crosses over a page boundary, then assume 4 cycles for the  branch.
    pub fn branch(&mut self, flag: u8, cond: bool) {
        // self.cy+=1;
        // relative addressing. load just one byte.
        // casting the u8 as an i8, and from there to u16 helps create the twos compliment of the number with length 16bits
        let off = self.load_u8_bump_pc() as i8 as u16;
        let old_pc = self.pc;
        let mut num_cy = 0;
        if cond && self.is_flag_set(flag) {
            self.pc = self.pc.wrapping_add(off);
            num_cy += 1; // branch was taken. branching truly occured
        } else {
            // !cond && !self.is_flag_set(flag)
            self.pc = self.pc.wrapping_add(off);
            num_cy += 1; // branch was taken. branching truly occured
        }
        if (self.pc & 0xff00) != (old_pc & 0xff00) {
            // crossed page boundary
            num_cy += 1;
        }
    }

    /// BPL - Branch on Result Plus
    pub(super) fn bpl(&mut self, _mode: AddressingMode) {
        self.branch(flags::NEGATIVE, false)
    }

    ///  BMI - Branch on Result Minus
    pub(super) fn bmi(&mut self, _mode: AddressingMode) {
        self.branch(flags::NEGATIVE, true)
    }

    /// BVC - Branch on Overflow Clear
    pub(super) fn bvc(&mut self, _mode: AddressingMode) {
        self.branch(flags::OVERFLOW, false)
    }

    /// BVS - Branch on Overflow Set
    pub(super) fn bvs(&mut self, _mode: AddressingMode) {
        self.branch(flags::OVERFLOW, true)
    }

    ///  BCC - Branch on Carry Clear
    pub(super) fn bcc(&mut self, _mode: AddressingMode) {
        self.branch(flags::CARRY, false)
    }

    /// BCS - Branch on Carry Set
    pub(super) fn bcs(&mut self, _mode: AddressingMode) {
        self.branch(flags::CARRY, true)
    }

    //  BNE - Branch on Result Not Zero
    pub(super) fn bne(&mut self, _mode: AddressingMode) {
        self.branch(flags::ZERO, false)
    }

    /// BEQ - Branch on Result Zero
    pub(super) fn beq(&mut self, _mode: AddressingMode) {
        self.branch(flags::ZERO, true)
    }
}

/// Status flag changes
/// All implied addressing
/// none of these ops have side effect of affecting other flags
impl Six502 {
    /// resets the carry flag to a 0
    /// typically precedes an `adc` loop. 
    /// IMPLIED addressing
    pub(super) fn clc(&mut self, _mode: AddressingMode) {
        self.clear_flag(flags::CARRY);
    }

    /// This instruction initializes the carry flag to a 1
    /// typically precedes an `sbc` loop. 
    /// IMPLIED addressing
    pub(super) fn sec(&mut self, _mode: AddressingMode) {
        self.set_flag(flags::CARRY);
    }
    /// cli resets interrupt disable to a 0
    /// IMPLIED addressing
    pub(super) fn cli(&mut self, _mode: AddressingMode) {
        self.clear_flag(flags::IRQ);
    }
    /// sei sets the interrupt disable flag (IRQ) to a 1
    /// IMPLIED addressing
    pub(super) fn sei(&mut self, _mode: AddressingMode) {
        self.set_flag(flags::IRQ);
    }
    /// clears the overflow flag to a 0
    /// used in conjunction with the set overflow pin which can change the state of the overflow flag with an external signal
    // comeback to implement pins, incl this set overflow pin
    pub(super) fn clv(&mut self, _mode: AddressingMode) {
        self.clear_flag(flags::OVERFLOW);
    }
    /// `cld` resets the decimal mode flag D to a 1
    /// IMPLIED addressing
    pub(super) fn cld(&mut self, _mode: AddressingMode) {
        self.clear_flag(flags::DECIMAL);
    }
    /// `sed` sets the decimal mode flag D to a 1
    /// IMPLIED addressing
    pub(super) fn sed(&mut self, _mode: AddressingMode) {
        self.set_flag(flags::DECIMAL);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use parameterized::parameterized;

    #[parameterized(inp = {1,2,3}, out ={2,3,4})]
    fn test_adc(inp: i32, out: i32) {
        assert_eq!(inp+1, out);
    }
}