///! six502 emulates the MOS 6502 CPU
///! Best resource to understand ths is the [MCS book](http://users.telenet.be/kim1-6502/6502/proman.html#90)
///! Other resources include: Masswerks description of the opcodes ar [masswerk](https://www.masswerk.at/6502/6502_instruction_set.html)
///! and the [6502 org website](http://www.6502.org/tutorials/6502opcodes.html)
///! The MCS6502 is an 8-bit microprocessor. This means that 8 bits of data are transferred or operated upon during each instruction cycle or operation cycle.
use crate::bus::{ByteAccess, DataBus, WordAccess};

use self::{addr_mode::AddressingMode, disasm::INSTRUCTIONS, ram::Ram};
use addr_mode::AddressingMode::*;
use bitflags::bitflags;
use std::collections::HashMap;

pub(crate) mod addr_mode;
pub(crate) mod disasm;
mod opcodes;
pub(crate) mod ram;
mod util;

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

pub(super) mod flags {
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
}

// SYSTEM VECTORS
// A vector pointer consists of a program counter high and program counter low value which, under control of
// the microprocessor, is loaded in the program counter when certain external events occur.
// The word vector is developed from the fact that the microprocessor directly controls the memory location from which a particular operation
// Locations FFFA through FFFF are reserved for Vector pointers for the microprocessor.
pub(super) mod vectors {
    pub(super) const NMI: u16 = 0xfffa; // NMI (Non-Maskable Interrupt) vector, 16-bit (LB, HB)
    pub(super) const IRQ: u16 = 0xfffe; // IRQ (Interrupt Request) vector, 16-bit (LB, HB)
    pub(super) const RESET: u16 = 0xfffc; // 16-bit (LB, HB)
}

pub struct Six502
{
    // the major use for the accumulator is transferring data from memory to the accumulator or from the accumulator to memory.
    // mathematical amd logical operations can then be done to data inside the accumulator. It is where intermediate values are normally  stored
    a: u8,
    x: u8,
    y: u8,
    // the program counter (program address pointer) is used to choose (address) the next memory location and the value which the memory
    // sends hack is decoded in order to determine what operation the MCS650X is going to perform next
    // it must always be addressing the operation the user wants to perform next
    // The microprocessor puts the value of the program counter Onto the address bus, transferring
    // the 8 bits of data at that memory address into the instruction decode. It then autoincreases by one
    // to change the sequence of ops, the programmer can only do it by changing the value of the pc
    pc: u16,
    s: u8,
    cy: u64,
    p: u8, // flags
    // Sixteen bits of address allow access to 65,536 memory locations, each of which, in the MCS650X family, consists of 8 bits of data
    pub(crate) bus: DataBus,

}




const CYCLES: [u8; 256] = [
    //    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, A, B, C, D, E, F  // lo bit
    /*0*/ 7, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6, 
    /*1*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    /*2*/ 6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6, 
    /*3*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7, 
    /*4*/ 6, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6,
    /*5*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7, 
    /*6*/ 6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6, 
    /*7*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    /*8*/ 2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4, 
    /*9*/ 2, 6, 2, 6, 4, 4, 4, 4, 2, 5, 2, 5, 5, 5, 5, 5, 
    /*A*/ 2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
    /*B*/ 2, 5, 2, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4, 
    /*C*/ 2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6, 
    /*D*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    /*E*/ 2, 6, 3, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6, 
    /*F*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    // hi bit
];

impl Six502 {
    pub(crate) fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0xc000,
            s: 0xfd,
            cy: 0,
            p: 0x24,
            bus: DataBus::new(),
        }
    }

    /// sets the program counter to the value the RESET vector pointer holds
    /// Instructions exist for the initialization and loading of all other registers in the microprocessor except for the initial setting of the
    /// program counter.  It is for this initial setting of the program counter to a fixed location in the restart vector location specified by the micro-
    /// processor programmer that the reset line in the microprocessor is primarily used.
    pub(super) fn reset(&mut self) {
        // There are two major facts to remember about initialization.  One, the only automatic operations of the microprocessor during reset are to turn
        // on the interrupt disable bit and to force the program counter to the vector location specified in locations
        // FFFC and FFFD and to load the first instruction from that location.
        // force the program counter to the vector location specified in locations FFFC and FFFD
        self.pc = self.load_u16(vectors::RESET);
        self.p = 0b00110100;

        // just to be sure
        self.a = 0x00;
        self.x = 0x00;
        self.y = 0x00;

        // comeback. number of cycles should be 8, byt should include
    }

    // the internal state of the pc and io should be deterministic at the beginning.
    // The reset line is controlled during power on initialization and is a common line which is connected to all devices in the microcomputer
    // which have to be initialized to a known state.
    // In the MCS650X, power on or reset control operates at two levels.
    // First, by holding of an external line to ground, and having this external
    //  line connected to all the devices connected to the microprocessor during power up,
    // the entire microcomputer system is initialized to a known disabled state
    // Second, the release of the reset line from the ground or TTL zero
    // condition to a TTL one condition causes the microprocessor to be automatically
    // initialized, first by the internal hardware vector (RESET) which causes it
    // to be pointed to a known program location (PC), and secondly by what the programmer writes as the first set of instructions
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // While the reset line is in the low state, it can be assumed that internal registers may be initialized to any random condition; therefore,
        // no conditions about the internal state of the microprocessor are assumed other than that the microprocessor will, one cycle after the reset line
        // goes high, implement the following sequence:
        self.reset();
        // comeback. the loaded program begins in the 8th cycle
        self.cy += 7;
        // the first operation in any normal program will be to initialize the stack
        // Once this is accomplished, the two non variable operations of the machine are under control.
        // The program counter is initialized and under
        // programmer control and the stack is initialized and under program control.
        Ok(())
    }

    // fetch uses the address bus to fetch an opcode. It gets the u8 and bumps the pc
    fn fetch_op(&mut self, op: &mut Op) {
        let op_num = self.load_u8_bump_pc();
        op.curr_op_num = op_num;
        let op_str = INSTRUCTIONS[op_num as usize];
        op.curr_op_str = op_str;
    }
    /// The first byte of an instruction is called the OP CODE and is coded to contain the basic operation such as LDA
    /// then it has the data necessary to allow the microprocessor to interpret the address of the data on which the operation will occur
    /// the pc will increment after picking up the opcode (executing). it will then pick up the address of data the opcode is to act on
    /// and incrementing again after. for a full operation, it may incr 1,2,3 or more times
    /// an instance is LDA absolute addressing. three increments. one for opcode. one for low addr byte. one for high addr byte
    pub fn exec(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let op = self.load_u8_bump_pc();
        let page_cross = match op {
            // load/stores
            0xa1 => self.lda(XIdxd_Indirect),
            0xa5 => self.lda(ZP),
            0xa9 => self.lda(Immediate),
            0xad => self.lda(Absolute),
            0xb1 => self.lda(Indirect_Y_Idxd),
            0xb5 => self.lda(ZP_X_Idxd),
            0xb9 => self.lda(Abs_Y_Idxd),
            0xbd => self.lda(Abs_X_Idxd),

            0xa2 => self.ldx(Immediate),
            0xa6 => self.ldx(ZP),
            0xae => self.ldx(Absolute),
            0xb6 => self.ldx(ZP_Y_Idxd),
            0xbe => self.ldx(Abs_Y_Idxd),

            0xa0 => self.ldy(Immediate),
            0xa4 => self.ldy(ZP),
            0xac => self.ldy(Absolute),
            0xb4 => self.ldy(ZP_X_Idxd),
            0xbc => self.ldy(Abs_X_Idxd),

            0x81 => self.sta(XIdxd_Indirect),
            0x85 => self.sta(ZP),
            0x8d => self.sta(Absolute),
            0x91 => self.sta(Indirect_Y_Idxd),
            0x95 => self.sta(ZP_X_Idxd),
            0x99 => self.sta(Abs_Y_Idxd),
            0x9d => self.sta(Abs_X_Idxd),

            0x86 => self.stx(ZP),
            0x8e => self.stx(Absolute),
            0x96 => self.stx(ZP_Y_Idxd),

            0x84 => self.sty(ZP),
            0x8c => self.sty(Absolute),
            0x94 => self.sty(ZP_X_Idxd),

            // comparisons
            0xc1 => self.cmp(XIdxd_Indirect),
            0xc5 => self.cmp(ZP),
            0xc9 => self.cmp(Immediate),
            0xcd => self.cmp(Absolute),
            0xd1 => self.cmp(Indirect_Y_Idxd),
            0xd5 => self.cmp(ZP_X_Idxd),
            0xd9 => self.cmp(Abs_Y_Idxd),
            0xdd => self.cmp(Abs_X_Idxd),

            0xe0 => self.cpx(Immediate),
            0xe4 => self.cpx(ZP),
            0xec => self.cpx(Absolute),

            0xc0 => self.cpy(Immediate),
            0xc4 => self.cpy(ZP),
            0xcc => self.cpy(Absolute),

            // transfers
            0xaa => self.tax(Implied),
            0xa8 => self.tay(Implied),
            0x8a => self.txa(Implied),
            0x98 => self.tya(Implied),
            0x9a => self.txs(Implied),
            0xba => self.tsx(Implied),

            // stack ops
            0x08 => self.php(Implied), //implied addressing
            0x28 => self.plp(Implied), //implied addressing
            0x48 => self.pha(Implied), //implied addressing
            0x68 => self.pla(Implied), //implied addressing

            // logical ops
            0x21 => self.and(XIdxd_Indirect),
            0x25 => self.and(ZP),
            0x29 => self.and(Immediate),
            0x2d => self.and(Absolute),
            0x35 => self.and(ZP_X_Idxd),
            0x31 => self.and(Indirect_Y_Idxd),
            0x39 => self.and(Abs_Y_Idxd),
            0x3d => self.and(Abs_X_Idxd),

            0x01 => self.ora(XIdxd_Indirect),
            0x05 => self.ora(ZP),
            0x09 => self.ora(Immediate),
            0x0d => self.ora(Absolute),
            0x11 => self.ora(Indirect_Y_Idxd),
            0x15 => self.ora(ZP_X_Idxd),
            0x1d => self.ora(Abs_X_Idxd),
            0x19 => self.ora(Abs_Y_Idxd),

            0x41 => self.eor(XIdxd_Indirect),
            0x45 => self.eor(ZP),
            0x49 => self.eor(Immediate),
            0x4d => self.eor(Absolute),
            0x51 => self.eor(Indirect_Y_Idxd),
            0x55 => self.eor(ZP_X_Idxd),
            0x5d => self.eor(Abs_X_Idxd),
            0x59 => self.eor(Abs_Y_Idxd),

            // bit test
            0x24 => {
                self.bit(ZP) //bit test
            }
            0x2c => {
                self.bit(Absolute) // bit test
            }

            // arithmetic ops
            0x61 => self.adc(XIdxd_Indirect),
            0x65 => self.adc(ZP),
            0x69 => self.adc(Immediate),
            0x6d => self.adc(Absolute),
            0x71 => self.adc(Indirect_Y_Idxd),
            0x75 => self.adc(ZP_X_Idxd),
            0x79 => self.adc(Abs_Y_Idxd),
            0x7d => self.adc(Abs_X_Idxd),

            0xe1 => self.sbc(XIdxd_Indirect),
            0xe5 => self.sbc(ZP),
            0xe9 => self.sbc(Immediate),
            0xed => self.sbc(Absolute),
            0xf1 => self.sbc(Indirect_Y_Idxd),
            0xf5 => self.sbc(ZP_X_Idxd),
            0xf9 => self.sbc(Abs_Y_Idxd),
            0xfd => self.sbc(Abs_X_Idxd),

            //incrs and decrs
            0xe6 => self.inc(ZP),
            0xee => self.inc(Absolute),
            0xf6 => self.inc(ZP_X_Idxd),
            0xfe => self.inc(Abs_X_Idxd),

            0xc6 => self.dec(ZP),
            0xce => self.dec(Absolute),
            0xd6 => self.dec(ZP_X_Idxd),
            0xde => self.dec(Abs_X_Idxd),

            0xe8 => self.inx(Implied),
            0xca => self.dex(Implied),
            0xc8 => self.iny(Implied),
            0x88 => self.dey(Implied),

            // shifts
            0x26 => self.rol(ZP),
            0x2a => self.rol(Accumulator),
            0x2e => self.rol(Absolute),
            0x36 => self.rol(ZP_X_Idxd),
            0x3e => self.rol(Abs_X_Idxd),

            0x66 => self.ror(ZP),
            0x6a => self.ror(Accumulator),
            0x6e => self.ror(Absolute),
            0x76 => self.ror(ZP_X_Idxd),
            0x7e => self.ror(Abs_X_Idxd),

            0x06 => self.asl(ZP),
            0x0e => self.asl(Absolute),
            0x0a => self.asl(Accumulator),
            0x16 => self.asl(ZP_X_Idxd),
            0x1e => self.asl(Abs_X_Idxd),

            0x4a => self.lsr(Accumulator),
            0x46 => self.lsr(ZP),
            0x4e => self.lsr(Absolute),
            0x56 => self.lsr(ZP_X_Idxd),
            0x5e => self.lsr(Abs_X_Idxd),

            // jumps and calls
            0x4c => self.jmp(Implied),          // absolute
            0x6c => self.jmp_indirect(Implied), // indirect

            0x20 => self.jsr(Implied), // absolute
            0x60 => self.rts(Implied), // implied. In an implied instruction, the data and/or destination is mandatory for the instruction
            0x00 => self.brk(Implied), // implied
            0x40 => self.rti(Implied), // implied

            // branches
            0x10 => self.bpl(Implied), // relative The byte after the opcode is the branch offset.
            0x30 => self.bmi(Implied), // relative
            0x50 => self.bvc(Implied), // relative
            0x70 => self.bvs(Implied), // relative
            0x90 => self.bcc(Implied), // relative
            0xb0 => self.bcs(Implied), // relative
            0xd0 => self.bne(Implied), // relative
            0xf0 => self.beq(Implied), // relative

            // status flag changes
            0x18 => self.clc(Implied), // implied. In an implied instruction, the data and/or destination is mandatory for the instruction
            0x38 => self.sec(Implied), // implied
            0x58 => self.cli(Implied), // implied
            0x78 => self.sei(Implied), // implied
            0xb8 => self.clv(Implied), // implied
            0xd8 => self.cld(Implied), // implied
            0xf8 => self.sed(Implied), // implied

            // no-op
            0xea => self.nop(Implied),

            _ => unimplemented!("op not unimplemented: {}", op),
        };
        self.cy = self
            .cy
            .wrapping_add(CYCLES[op as usize] as u64)
            .wrapping_add(page_cross as u64);

        Ok(())
    }
}

impl ByteAccess for Six502 {
    fn load_u8(&mut self, addr: u16) -> u8 {
        self.bus.load_u8(addr)
    }

    fn store_u8(&mut self, addr: u16, v: u8) {
        self.bus.store_u8(addr, v);
    }
}


struct Op//<T>
//where T:  FnMut(& mut Six502, AddressingMode) -> u8,
{
    curr_op: fn(& mut Six502, AddressingMode) -> u8,
    curr_op_num: u8,
    curr_op_str: &'static str,
    addr_mode: AddressingMode,
}

impl/*< T>*/ Default for Op//< T> 
// where T: FnMut(&mut Six502, AddressingMode) -> u8, 
{
    fn default() -> Self {
        Self { 
            curr_op: Six502::nop, 
            curr_op_num: 0, 
            curr_op_str: "",
            addr_mode: Implied,
        }
    }
}


impl /*<T>*/ Op//<T>
// where T:  FnMut(& mut Six502, AddressingMode) -> u8,
{
    fn new() -> Self {
        Self {
            curr_op: Six502::nop,
            curr_op_num: 0,
            curr_op_str: "",
            addr_mode: Implied
        }
    }

    // in 6502, as is in any processor, opcode decoding is a different process from opcode feching.
    // I chose to model this system to respect that difference
    pub(super) fn decode_op(&mut self) {
        match self.curr_op_num {
            0xa1 => {
                self.curr_op =  Six502::lda;
                self.addr_mode = XIdxd_Indirect;
            }, 
            0xa5 => {
                self.curr_op = Six502::lda;
                self.addr_mode = ZP;
            },
            0xa9 => {self.curr_op = Six502::lda; self.addr_mode = Immediate},
            0xad => {self.curr_op = Six502::lda; self.addr_mode= Absolute},
            0xb1 => {self.curr_op = Six502::lda; self.addr_mode = Indirect_Y_Idxd}, //(Indirect_Y_Idxd),
            0xb5 => {self.curr_op = Six502::lda; self.addr_mode = ZP_X_Idxd}, //(ZP_X_Idxd),
            0xb9 => {self.curr_op = Six502::lda; self.addr_mode = Abs_Y_Idxd}, //(Abs_Y_Idxd),
            0xbd => {self.curr_op =  Six502::lda; self.addr_mode = Abs_X_Idxd}, //(Abs_X_Idxd),

            0xa2 => {self.curr_op =Six502::ldx; self.addr_mode = Immediate}, //(Immediate),
            0xa6 => {self.curr_op =Six502::ldx; self.addr_mode =ZP}, //(ZP),
            0xae => {self.curr_op =Six502::ldx; self.addr_mode = Absolute}, //(Absolute),
            0xb6 => {self.curr_op =Six502::ldx; self.addr_mode = ZP_Y_Idxd}, //(ZP_Y_Idxd),
            0xbe => {self.curr_op =Six502::ldx; self.addr_mode = Abs_Y_Idxd}, //(Abs_Y_Idxd),

            0xa0 => {self.curr_op= Six502::ldy; self.addr_mode = Immediate}, //(Immediate),
            0xa4 => {self.curr_op= Six502::ldy; self.addr_mode = ZP}, //(ZP),
            0xac => {self.curr_op= Six502::ldy; self.addr_mode = Absolute}, //(Absolute),
            0xb4 => {self.curr_op= Six502::ldy; self.addr_mode = ZP_X_Idxd}, //(ZP_X_Idxd),
            0xbc => {self.curr_op= Six502::ldy; self.addr_mode = Abs_X_Idxd}, //(Abs_X_Idxd),

            0x81 =>{self.curr_op = Six502::sta; self.addr_mode = XIdxd_Indirect}, //(XIdxd_Indirect),
            0x85 =>{self.curr_op = Six502::sta; self.addr_mode = ZP}, //(ZP),
            0x8d =>{self.curr_op = Six502::sta; self.addr_mode = Absolute}, //(Absolute),
            0x91 =>{self.curr_op = Six502::sta; self.addr_mode = Indirect_Y_Idxd}, //(Indirect_Y_Idxd),
            0x95 =>{self.curr_op = Six502::sta; self.addr_mode = ZP_X_Idxd}, //(ZP_X_Idxd),
            0x99 =>{self.curr_op = Six502::sta; self.addr_mode = Abs_Y_Idxd}, //(Abs_Y_Idxd),
            0x9d =>{self.curr_op = Six502::sta; self.addr_mode = Abs_X_Idxd}, //(Abs_X_Idxd),

            0x86 => {self.curr_op = Six502::stx; self.addr_mode = ZP}, //(ZP),
            0x8e => {self.curr_op = Six502::stx; self.addr_mode = Absolute}, //(Absolute),
            0x96 => {self.curr_op = Six502::stx; self.addr_mode = ZP_Y_Idxd}, //(ZP_Y_Idxd),

            0x84 => {self.curr_op = Six502::sty; self.addr_mode = ZP}, //(ZP),
            0x8c => {self.curr_op = Six502::sty; self.addr_mode = Absolute}, //(Absolute),
            0x94 => {self.curr_op = Six502::sty; self.addr_mode = ZP_X_Idxd}, //(ZP_X_Idxd),

            // comparisons
            0xc1 => {self.curr_op = Six502::cmp; self.addr_mode = XIdxd_Indirect}, //(XIdxd_Indirect),
            0xc5 => {self.curr_op = Six502::cmp; self.addr_mode = ZP}, //(ZP),
            0xc9 => {self.curr_op = Six502::cmp; self.addr_mode = Immediate}, //(Immediate),
            0xcd => {self.curr_op = Six502::cmp; self.addr_mode = Absolute}, //(Absolute),
            0xd1 => {self.curr_op = Six502::cmp; self.addr_mode = Indirect_Y_Idxd}, //(Indirect_Y_Idxd),
            0xd5 => {self.curr_op = Six502::cmp; self.addr_mode = ZP_X_Idxd}, //(ZP_X_Idxd),
            0xd9 => {self.curr_op = Six502::cmp; self.addr_mode = Abs_Y_Idxd}, //(Abs_Y_Idxd),
            0xdd => {self.curr_op = Six502::cmp; self.addr_mode = Abs_X_Idxd}, //(Abs_X_Idxd),

            0xe0 => {self.curr_op = Six502::cpx; self.addr_mode = Immediate}, //(Immediate),
            0xe4 => {self.curr_op = Six502::cpx; self.addr_mode = ZP}, //(ZP),
            0xec => {self.curr_op = Six502::cpx; self.addr_mode = Absolute}, //(Absolute),

            0xc0 => {self.curr_op = Six502::cpy; self.addr_mode = Immediate}, //(Immediate),
            0xc4 => {self.curr_op = Six502::cpy; self.addr_mode = ZP}, //(ZP),
            0xcc => {self.curr_op = Six502::cpy; self.addr_mode = Absolute}, //(Absolute),

            // transfers
            0xaa => {self.curr_op = Six502::tax; self.addr_mode = Implied}, //(),
            0xa8 => {self.curr_op = Six502::tay; self.addr_mode = Implied}, //(),
            0x8a => {self.curr_op = Six502::txa; self.addr_mode = Implied}, //(),
            0x98 => {self.curr_op = Six502::tya; self.addr_mode = Implied}, //(),
            0x9a => {self.curr_op = Six502::txs; self.addr_mode = Implied}, //(),
            0xba => {self.curr_op = Six502::tsx; self.addr_mode = Implied}, //(),

            // stack ops
            0x08 => {self.curr_op = Six502::php; self.addr_mode = Implied}, //(), //implied addressing
            0x28 => {self.curr_op = Six502::plp; self.addr_mode = Implied}, //(), //implied addressing
            0x48 => {self.curr_op = Six502::pha; self.addr_mode = Implied}, //(), //implied addressing
            0x68 => {self.curr_op = Six502::pla; self.addr_mode = Implied}, //(), //implied addressing

            // logical ops
            0x21 => {self.curr_op = Six502::and; self.addr_mode =XIdxd_Indirect }, //(XIdxd_Indirect),
            0x25 => {self.curr_op = Six502::and; self.addr_mode = ZP}, //(ZP),
            0x29 => {self.curr_op = Six502::and; self.addr_mode = Immediate}, //(Immediate),
            0x2d => {self.curr_op = Six502::and; self.addr_mode = Absolute}, //(Absolute),
            0x35 => {self.curr_op = Six502::and; self.addr_mode = ZP_X_Idxd}, //(ZP_X_Idxd),
            0x31 => {self.curr_op = Six502::and; self.addr_mode = Indirect_Y_Idxd}, //(Indirect_Y_Idxd),
            0x39 => {self.curr_op = Six502::and; self.addr_mode = Abs_Y_Idxd}, //(Abs_Y_Idxd),
            0x3d => {self.curr_op = Six502::and; self.addr_mode = Abs_X_Idxd}, //(Abs_X_Idxd),

            0x01 => {self.curr_op =  Six502::ora; self.addr_mode = XIdxd_Indirect}, //(XIdxd_Indirect),
            0x05 => {self.curr_op =  Six502::ora; self.addr_mode = ZP}, //(ZP),
            0x09 => {self.curr_op =  Six502::ora; self.addr_mode = Immediate}, //(Immediate),
            0x0d => {self.curr_op =  Six502::ora; self.addr_mode = Absolute}, //(Absolute),
            0x11 => {self.curr_op =  Six502::ora; self.addr_mode = Indirect_Y_Idxd}, //(Indirect_Y_Idxd),
            0x15 => {self.curr_op =  Six502::ora; self.addr_mode = ZP_X_Idxd}, //(ZP_X_Idxd),
            0x1d => {self.curr_op =  Six502::ora; self.addr_mode = Abs_X_Idxd}, //(Abs_X_Idxd),
            0x19 => {self.curr_op =  Six502::ora; self.addr_mode = Abs_Y_Idxd}, //(Abs_Y_Idxd),

            0x41 => {self.curr_op = Six502::eor; self.addr_mode = XIdxd_Indirect}, //(XIdxd_Indirect),
            0x45 => {self.curr_op = Six502::eor; self.addr_mode = ZP}, //(ZP),
            0x49 => {self.curr_op = Six502::eor; self.addr_mode = Immediate}, //(Immediate),
            0x4d => {self.curr_op = Six502::eor; self.addr_mode = Absolute}, //(Absolute),
            0x51 => {self.curr_op = Six502::eor; self.addr_mode = Indirect_Y_Idxd}, //(Indirect_Y_Idxd),
            0x55 => {self.curr_op = Six502::eor; self.addr_mode = ZP_X_Idxd}, //(ZP_X_Idxd),
            0x5d => {self.curr_op = Six502::eor; self.addr_mode = Abs_X_Idxd}, //(Abs_X_Idxd),
            0x59 => {self.curr_op = Six502::eor; self.addr_mode = Abs_Y_Idxd}, //(Abs_Y_Idxd),

            // bit test
            0x24 => {
                self.curr_op = Six502::bit;
                self.addr_mode = ZP //(ZP) //bit test
            }
            0x2c => {
                self.curr_op = Six502::bit; //(Absolute) // bit test
                self.addr_mode = Absolute;
            }

            // arithmetic ops
            0x61 => {self.curr_op = Six502::adc; self.addr_mode = XIdxd_Indirect}, //(XIdxd_Indirect),
            0x65 => {self.curr_op = Six502::adc; self.addr_mode =ZP }, //(ZP),
            0x69 => {self.curr_op = Six502::adc; self.addr_mode = Immediate}, //(Immediate),
            0x6d => {self.curr_op = Six502::adc; self.addr_mode = Absolute}, //(Absolute),
            0x71 => {self.curr_op = Six502::adc; self.addr_mode = Indirect_Y_Idxd}, //(Indirect_Y_Idxd),
            0x75 => {self.curr_op = Six502::adc; self.addr_mode = ZP_X_Idxd}, //(ZP_X_Idxd),
            0x79 => {self.curr_op = Six502::adc; self.addr_mode = Abs_Y_Idxd}, //(Abs_Y_Idxd),
            0x7d => {self.curr_op = Six502::adc; self.addr_mode = Abs_X_Idxd}, //(Abs_X_Idxd),

            0xe1 => {self.curr_op = Six502::sbc; self.addr_mode = XIdxd_Indirect}, //(XIdxd_Indirect),
            0xe5 => {self.curr_op = Six502::sbc; self.addr_mode = ZP}, //(ZP),
            0xe9 => {self.curr_op = Six502::sbc; self.addr_mode = Immediate}, //(Immediate),
            0xed => {self.curr_op = Six502::sbc; self.addr_mode = Absolute}, //(Absolute),
            0xf1 => {self.curr_op = Six502::sbc; self.addr_mode = Indirect_Y_Idxd}, //(Indirect_Y_Idxd),
            0xf5 => {self.curr_op = Six502::sbc; self.addr_mode = ZP_X_Idxd}, //(ZP_X_Idxd),
            0xf9 => {self.curr_op = Six502::sbc; self.addr_mode = Abs_Y_Idxd}, //(Abs_Y_Idxd),
            0xfd => {self.curr_op = Six502::sbc; self.addr_mode = Abs_X_Idxd}, //(Abs_X_Idxd),

            //incrs and decrs
            0xe6 => {self.curr_op = Six502::inc; self.addr_mode = ZP}, //(ZP),
            0xee => {self.curr_op = Six502::inc; self.addr_mode = Absolute}, //(Absolute),
            0xf6 => {self.curr_op = Six502::inc; self.addr_mode = ZP_X_Idxd}, //(ZP_X_Idxd),
            0xfe => {self.curr_op = Six502::inc; self.addr_mode = Abs_X_Idxd}, //(Abs_X_Idxd),

            0xc6 => {self.curr_op = Six502::dec; self.addr_mode = ZP}, //(ZP),
            0xce => {self.curr_op = Six502::dec; self.addr_mode = Absolute}, //(Absolute),
            0xd6 => {self.curr_op = Six502::dec; self.addr_mode = ZP_X_Idxd}, //(ZP_X_Idxd),
            0xde => {self.curr_op = Six502::dec; self.addr_mode = Abs_X_Idxd}, //(Abs_X_Idxd),

            0xe8 => {self.curr_op = Six502::inx; self.addr_mode = Implied}, //(),
            0xca => {self.curr_op =  Six502::dex; self.addr_mode = Implied}, //(),
            0xc8 => {self.curr_op = Six502::iny; self.addr_mode = Implied}, //(),
            0x88 => {self.curr_op = Six502::dey; self.addr_mode = Implied}, //(),

            // shifts
            0x26 => {self.curr_op = Six502::rol; self.addr_mode = ZP}, //(ZP),
            0x2a => {self.curr_op = Six502::rol; self.addr_mode = Accumulator}, //(Accumulator),
            0x2e => {self.curr_op = Six502::rol; self.addr_mode = Absolute}, //(Absolute),
            0x36 => {self.curr_op = Six502::rol; self.addr_mode = ZP_X_Idxd}, //(ZP_X_Idxd),
            0x3e => {self.curr_op = Six502::rol; self.addr_mode = Abs_X_Idxd}, //(Abs_X_Idxd),

            0x66 => {self.curr_op = Six502::ror; self.addr_mode = ZP}, //(ZP),
            0x6a => {self.curr_op = Six502::ror; self.addr_mode = Accumulator}, //(Accumulator),
            0x6e => {self.curr_op = Six502::ror; self.addr_mode = Absolute}, //(Absolute),
            0x76 => {self.curr_op = Six502::ror; self.addr_mode = ZP_X_Idxd}, //(ZP_X_Idxd),
            0x7e => {self.curr_op = Six502::ror; self.addr_mode = Abs_X_Idxd}, //(Abs_X_Idxd),

            0x06 => {self.curr_op = Six502::asl; self.addr_mode = ZP} //(ZP),
            0x0e => {self.curr_op = Six502::asl; self.addr_mode = Absolute} //(Absolute),
            0x0a => {self.curr_op = Six502::asl; self.addr_mode = Accumulator} //(Accumulator),
            0x16 => {self.curr_op = Six502::asl; self.addr_mode = ZP_X_Idxd} //(ZP_X_Idxd),
            0x1e => {self.curr_op = Six502::asl; self.addr_mode = Abs_X_Idxd} //(Abs_X_Idxd),

            0x4a => {self.curr_op = Six502::lsr; self.addr_mode = Accumulator} //(Accumulator),
            0x46 => {self.curr_op = Six502::lsr; self.addr_mode = ZP} //(ZP),
            0x4e => {self.curr_op = Six502::lsr; self.addr_mode = Absolute} //(Absolute),
            0x56 => {self.curr_op = Six502::lsr; self.addr_mode = ZP_X_Idxd} //(ZP_X_Idxd),
            0x5e => {self.curr_op = Six502::lsr; self.addr_mode = Abs_X_Idxd} //(Abs_X_Idxd),

            // jumps and calls
            0x4c => {self.curr_op = Six502::jmp; self.addr_mode = Absolute}          //(),          // absolute
            0x6c => {self.curr_op = Six502::jmp_indirect; self.addr_mode = Indirect}, //(), // indirect

            0x20 => {self.curr_op = Six502::jsr; self.addr_mode = Absolute}, //(), // absolute
            0x60 => {self.curr_op = Six502::rts; self.addr_mode = Implied}, //(), // implied. In an implied instruction, the data and/or destination is mandatory for the instruction
            0x00 => {self.curr_op = Six502::brk; self.addr_mode = Implied}, //(), // implied
            0x40 => {self.curr_op = Six502::rti; self.addr_mode = Implied}, //(), // implied

            // branches
            0x10 => {self.curr_op = Six502::bpl; self.addr_mode = Relative}, //(), // relative The byte after the opcode is the branch offset.
            0x30 => {self.curr_op = Six502::bmi ; self.addr_mode = Relative} //(), // relative
            0x50 => {self.curr_op = Six502::bvc; self.addr_mode = Relative} //(), // relative
            0x70 => {self.curr_op = Six502::bvs ; self.addr_mode = Relative}, //(), // relative
            0x90 => {self.curr_op = Six502::bcc; self.addr_mode = Relative} //(), // relative
            0xb0 => {self.curr_op = Six502::bcs; self.addr_mode = Relative} //(), // relative
            0xd0 => {self.curr_op = Six502::bne; self.addr_mode = Relative} //(), // relative
            0xf0 => {self.curr_op = Six502::beq; self.addr_mode = Relative} //(), // relative

            // status flag changes
            0x18 => {self.curr_op = Six502::clc; self.addr_mode = Implied}, // (), // implied. In an implied instruction, the data and/or destination is mandatory for the instruction
            0x38 => {self.curr_op= Six502::sec; self.addr_mode = Implied} // (); self.addr_mode = Implied} // implied
            0x58 => {self.curr_op = Six502::cli; self.addr_mode = Implied}, // (), // implied
            0x78 => {self.curr_op= Six502::sei; self.addr_mode = Implied} // (), // implied
            0xb8 => {self.curr_op= Six502::clv; self.addr_mode = Implied} // (), // implied
            0xd8 => {self.curr_op= Six502::cld; self.addr_mode = Implied} // (), // implied
            0xf8 => {self.curr_op= Six502::sed; self.addr_mode = Implied} // (), // implied

            // no-op
            0xea => {self.curr_op = Six502::nop; self.addr_mode = Implied}, //(Implied),

            _ => unimplemented!("op not unimplemented: {}", self.curr_op_num),
        };
    }
}