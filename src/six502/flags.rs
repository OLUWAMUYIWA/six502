//! |   |   |   |   |   |   |   |   |
//! | N | V |   | B | D | I | Z | C |     PROCESSOR STATUS REGISTER
//! |   |   |   |   |   |   |   |   |
//! |   |   |   |   |   |   |   |
//! |   |   |   |   |   |   |   +------ CARRY
//! |   |   |   |   |   |   |
//! |   |   |   |   |   |   +---------- ZERO RESULT
//! |   |   |   |   |   |
//! |   |   |   |   |   +-------------- INTERRUPT DISABLE
//! |   |   |   |   |
//! |   |   |   |   +------------------ DECIMAL MODE
//! |   |   |   |
//! |   |   |   +---------------------- BREAK COMMAND
//! |   |   |
//! |   |   +-------------------------- EXPANSION
//! |
//! |   +------------------------------ OVERFLOW
//! |
//! +---------------------------------- NEGATIVE RESULT
//! http://users.telenet.be/kim1-6502/6502/proman.html#3
//! All these are flip flops


/// Generally ust the night bit in operations that affect it. `sec` sets it, `clc` resets it
/// In the case of shift and rotate instruction, the carry bit is used as a ninth bit as it is in the arithmetic operation
/// Operations which affect the carry are ADC, ASL, CLC, CMP, CPX, CPY, LSR, PLP, ROL, RTI, SBC, SEC
/// It is mostly used in arithmetic operations. e.g. in `sbc` it determines whether there is a borrow. its compliment
/// indicates a borrow exists. In `adc` is tests for a simple carry upon addition.
pub const CARRY: u8 = 1 << 0;

/// Automatically set by the microprocessor during any data movement or calculation operation when the 8 bits of results of the operation are 0
/// Use 1: Programmer is able to check the 8th bit of values(in signed arithmetic ops) to know if the result of an operation 
/// is negative or not. The overflow tells them whether the 7-bit they're looking at is containing a result that is larger than 7 bits
/// Use 2: internal check by the processor when decrementing, so as not to go below .
/// affected by:  ADC, AND, ASL, BIT, CMP, CPY, CPX, DEC, DEX, DEY, EOR, INC, INX, INY, LDA, LDX, LDY, LSR, ORA, PLA, PLP, ROL, RTI, SBC, TAX, TAY, TXA, TYA.
pub const ZERO: u8 = 1 << 1;

/// interrupt disable flag
/// the purpose is to disable the effects of the interrupt request pin
/// IRQ is set by the microprocessor during reset and interrupt commands
/// It is reset by the CLI instruction or the PLP instruction, or at a return from interrupt in which the interrupt disable was reset prior to the interrupt
pub const IRQ: u8 = 1 << 2;

/// given that the adder is in charge oarithmetic ops, this flag is useed to specify if the arithmetic should be done as straight binary nums or as decimals
pub const DECIMAL: u8 = 1 << 3;

/// set only by the microprocessor and
/// used to determine during an interrupt service sequence whether or not the interrupt was caused by BRK command or by a real interrupt
pub const BREAK: u8 = 1 << 4;

/// expansion bit
pub const UNUSED: u8 = 1 << 5;

/// Used to indicate that a value greater han 7 bits is the actual result of the computatio
/// what this means is that the sign bit is not actually a sign bit but an overflow from the lower seven bits
/// its major purpose is to monitor this
/// used in signed aritmetic. user who is not using signed arithmetic  can totally ignore this flag
/// 
pub const OVERFLOW: u8 = 1 << 6;

/// the NEGATIVE flag is set equal to bit 7 of the resulting value in all data movement and data arithmetic
/// This means, for instance, after a signed add one can determine the sign of the
/// result by sampling the N flag directly rather than finding a way to isolate bit 7
pub const NEGATIVE: u8 = 1 << 7;
