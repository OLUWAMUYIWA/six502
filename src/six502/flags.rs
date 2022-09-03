// |   |   |   |   |   |   |   |   |
// | N | V |   | B | D | I | Z | C |     PROCESSOR STATUS REGISTER
// |   |   |   |   |   |   |   |   |
// |   |   |   |   |   |   |   |
// |   |   |   |   |   |   |   +------ CARRY
// |   |   |   |   |   |   |
// |   |   |   |   |   |   +---------- ZERO RESULT
// |   |   |   |   |   |
// |   |   |   |   |   +-------------- INTERRUPT DISABLE
// |   |   |   |   |
// |   |   |   |   +------------------ DECIMAL MODE
// |   |   |   |
// |   |   |   +---------------------- BREAK COMMAND
// |   |   |
// |   |   +-------------------------- EXPANSION
// |
// |   +------------------------------ OVERFLOW
// |
// +---------------------------------- NEGATIVE RESULT
// http://users.telenet.be/kim1-6502/6502/proman.html#3

// In the case of shift and rotate instruction, the carry bit is used as a ninth bit as it is in the arithmetic operation
// Operations which affect the carry are ADC, ASL, CLC, CMP, CPX, CPY, LSR, PLP, ROL, RTI, SBC, SEC
pub const CARRY: u8 = 1 << 0;
// automatically set by the microprocessor during any data movement or calculation operation when the 8 bits of results of the operation are 0
// uses: interna check by the processor when decrementing, so as not go go below .
// affected by:  ADC, AND, ASL, BIT, CMP, CPY, CPX, DEC, DEX, DEY, EOR, INC, INX, INY, LDA, LDX, LDY, LSR, ORA, PLA, PLP, ROL, RTI, SBC, TAX, TAY, TXA, TYA.
pub const ZERO: u8 = 1 << 1;
// interrupt disable flag
// the purpose is to disable the effects of the interrupt request pin
// IRQ is set by the microprocessor during reset and interrupt commands
// It is reset by the CLI instruction or the PLP instruction, or at a return from interrupt in which the interrupt disable was reset prior to the interrupt
pub const IRQ: u8 = 1 << 2;
// given that the adder is in charge oarithmetic ops, this flag is useed to specify if the arithmetic should be done as straight binary nums or as decimals
pub const DECIMAL: u8 = 1 << 3;
// set only by the microprocessor and is used to determine during an interrupt service sequence whether or not the interrupt was caused by BRK command or by a real interrupt
pub const BREAK: u8 = 1 << 4;
// expansion bit
pub const UNUSED: u8 = 1 << 5;
// used in signed aritmetic. user who is not using signed arithmetic  can totally ignore this flag
pub const OVERFLOW: u8 = 1 << 6;
// the NEGATIVE flag is set equal to bit 7 of the resulting value in all data movement and data arithmetic
// This means, for instance, after a signed add one can determine the sign of the
// result by sampling the N flag directly rather than finding a way to isolate bit 7
pub const NEGATIVE: u8 = 1 << 7;
