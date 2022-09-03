///! six502 emulates the MOS 6502 CPU
///! Best resource to understand ths is the [MCS book](http://users.telenet.be/kim1-6502/6502/proman.html#90)
///! Other resources include: Masswerks description of the opcodes ar [masswerk](https://www.masswerk.at/6502/6502_instruction_set.html)
///! and the [6502 org website](http://www.6502.org/tutorials/6502opcodes.html)
///! The MCS6502 is an 8-bit microprocessor. This means that 8 bits of data are transferred or operated upon during each instruction cycle or operation cycle.
use crate::bus::{ByteAccess, DataBus, WordAccess};

use self::{addr_mode::AddressingMode, disasm::INSTRUCTIONS, ram::Ram, six502::Six502};
use addr_mode::AddressingMode::*;
use bitflags::bitflags;
use std::collections::HashMap;

pub(crate) mod addr_mode;
pub(crate) mod disasm;
mod opcodes;
pub(crate) mod ram;
pub(crate) mod six502;
mod util;

mod flags;

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

const CYCLES: [u8; 256] = [
    //    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, A, B, C, D, E, F  // lo bit
    /*0*/ 7, 6, 2, 8, 3,
    3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6, /*1*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    /*2*/ 6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6, /*3*/ 2, 5, 2, 8, 4, 4, 6, 6,
    2, 4, 2, 7, 4, 4, 7, 7, /*4*/ 6, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6,
    /*5*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7, /*6*/ 6, 6, 2, 8, 3, 3, 5, 5,
    4, 2, 2, 2, 5, 4, 6, 6, /*7*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    /*8*/ 2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4, /*9*/ 2, 6, 2, 6, 4, 4, 4, 4,
    2, 5, 2, 5, 5, 5, 5, 5, /*A*/ 2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
    /*B*/ 2, 5, 2, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4, /*C*/ 2, 6, 2, 8, 3, 3, 5, 5,
    2, 2, 2, 2, 4, 4, 6, 6, /*D*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    /*E*/ 2, 6, 3, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6, /*F*/ 2, 5, 2, 8, 4, 4, 6, 6,
    2, 4, 2, 7, 4, 4, 7, 7,
    // hi bit
];

pub struct Op
{
    curr_op: fn(&mut Six502, AddressingMode) -> u8,
    curr_op_num: u8,
    curr_op_str: &'static str,
    addr_mode: AddressingMode,
}

impl Default for Op 
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

impl Op 
{
    fn new() -> Self {
        Self {
            curr_op: Six502::nop,
            curr_op_num: 0,
            curr_op_str: "",
            addr_mode: Implied,
        }
    }

    // in 6502, as is in any processor, opcode decoding is a different process from opcode feching.
    // I chose to model this system to respect that difference
    pub(super) fn decode_op(&mut self) {
        match self.curr_op_num {
            0xa1 => {
                self.curr_op = Six502::lda;
                self.addr_mode = XIdxd_Indirect;
            }
            0xa5 => {
                self.curr_op = Six502::lda;
                self.addr_mode = ZP;
            }
            0xa9 => {
                self.curr_op = Six502::lda;
                self.addr_mode = Immediate
            }
            0xad => {
                self.curr_op = Six502::lda;
                self.addr_mode = Absolute
            }
            0xb1 => {
                self.curr_op = Six502::lda;
                self.addr_mode = Indirect_Y_Idxd
            } //(Indirect_Y_Idxd),
            0xb5 => {
                self.curr_op = Six502::lda;
                self.addr_mode = ZP_X_Idxd
            } //(ZP_X_Idxd),
            0xb9 => {
                self.curr_op = Six502::lda;
                self.addr_mode = Abs_Y_Idxd
            } //(Abs_Y_Idxd),
            0xbd => {
                self.curr_op = Six502::lda;
                self.addr_mode = Abs_X_Idxd
            } //(Abs_X_Idxd),

            0xa2 => {
                self.curr_op = Six502::ldx;
                self.addr_mode = Immediate
            } //(Immediate),
            0xa6 => {
                self.curr_op = Six502::ldx;
                self.addr_mode = ZP
            } //(ZP),
            0xae => {
                self.curr_op = Six502::ldx;
                self.addr_mode = Absolute
            } //(Absolute),
            0xb6 => {
                self.curr_op = Six502::ldx;
                self.addr_mode = ZP_Y_Idxd
            } //(ZP_Y_Idxd),
            0xbe => {
                self.curr_op = Six502::ldx;
                self.addr_mode = Abs_Y_Idxd
            } //(Abs_Y_Idxd),

            0xa0 => {
                self.curr_op = Six502::ldy;
                self.addr_mode = Immediate
            } //(Immediate),
            0xa4 => {
                self.curr_op = Six502::ldy;
                self.addr_mode = ZP
            } //(ZP),
            0xac => {
                self.curr_op = Six502::ldy;
                self.addr_mode = Absolute
            } //(Absolute),
            0xb4 => {
                self.curr_op = Six502::ldy;
                self.addr_mode = ZP_X_Idxd
            } //(ZP_X_Idxd),
            0xbc => {
                self.curr_op = Six502::ldy;
                self.addr_mode = Abs_X_Idxd
            } //(Abs_X_Idxd),

            0x81 => {
                self.curr_op = Six502::sta;
                self.addr_mode = XIdxd_Indirect
            } //(XIdxd_Indirect),
            0x85 => {
                self.curr_op = Six502::sta;
                self.addr_mode = ZP
            } //(ZP),
            0x8d => {
                self.curr_op = Six502::sta;
                self.addr_mode = Absolute
            } //(Absolute),
            0x91 => {
                self.curr_op = Six502::sta;
                self.addr_mode = Indirect_Y_Idxd
            } //(Indirect_Y_Idxd),
            0x95 => {
                self.curr_op = Six502::sta;
                self.addr_mode = ZP_X_Idxd
            } //(ZP_X_Idxd),
            0x99 => {
                self.curr_op = Six502::sta;
                self.addr_mode = Abs_Y_Idxd
            } //(Abs_Y_Idxd),
            0x9d => {
                self.curr_op = Six502::sta;
                self.addr_mode = Abs_X_Idxd
            } //(Abs_X_Idxd),

            0x86 => {
                self.curr_op = Six502::stx;
                self.addr_mode = ZP
            } //(ZP),
            0x8e => {
                self.curr_op = Six502::stx;
                self.addr_mode = Absolute
            } //(Absolute),
            0x96 => {
                self.curr_op = Six502::stx;
                self.addr_mode = ZP_Y_Idxd
            } //(ZP_Y_Idxd),

            0x84 => {
                self.curr_op = Six502::sty;
                self.addr_mode = ZP
            } //(ZP),
            0x8c => {
                self.curr_op = Six502::sty;
                self.addr_mode = Absolute
            } //(Absolute),
            0x94 => {
                self.curr_op = Six502::sty;
                self.addr_mode = ZP_X_Idxd
            } //(ZP_X_Idxd),

            // comparisons
            0xc1 => {
                self.curr_op = Six502::cmp;
                self.addr_mode = XIdxd_Indirect
            } //(XIdxd_Indirect),
            0xc5 => {
                self.curr_op = Six502::cmp;
                self.addr_mode = ZP
            } //(ZP),
            0xc9 => {
                self.curr_op = Six502::cmp;
                self.addr_mode = Immediate
            } //(Immediate),
            0xcd => {
                self.curr_op = Six502::cmp;
                self.addr_mode = Absolute
            } //(Absolute),
            0xd1 => {
                self.curr_op = Six502::cmp;
                self.addr_mode = Indirect_Y_Idxd
            } //(Indirect_Y_Idxd),
            0xd5 => {
                self.curr_op = Six502::cmp;
                self.addr_mode = ZP_X_Idxd
            } //(ZP_X_Idxd),
            0xd9 => {
                self.curr_op = Six502::cmp;
                self.addr_mode = Abs_Y_Idxd
            } //(Abs_Y_Idxd),
            0xdd => {
                self.curr_op = Six502::cmp;
                self.addr_mode = Abs_X_Idxd
            } //(Abs_X_Idxd),

            0xe0 => {
                self.curr_op = Six502::cpx;
                self.addr_mode = Immediate
            } //(Immediate),
            0xe4 => {
                self.curr_op = Six502::cpx;
                self.addr_mode = ZP
            } //(ZP),
            0xec => {
                self.curr_op = Six502::cpx;
                self.addr_mode = Absolute
            } //(Absolute),

            0xc0 => {
                self.curr_op = Six502::cpy;
                self.addr_mode = Immediate
            } //(Immediate),
            0xc4 => {
                self.curr_op = Six502::cpy;
                self.addr_mode = ZP
            } //(ZP),
            0xcc => {
                self.curr_op = Six502::cpy;
                self.addr_mode = Absolute
            } //(Absolute),

            // transfers
            0xaa => {
                self.curr_op = Six502::tax;
                self.addr_mode = Implied
            } //(),
            0xa8 => {
                self.curr_op = Six502::tay;
                self.addr_mode = Implied
            } //(),
            0x8a => {
                self.curr_op = Six502::txa;
                self.addr_mode = Implied
            } //(),
            0x98 => {
                self.curr_op = Six502::tya;
                self.addr_mode = Implied
            } //(),
            0x9a => {
                self.curr_op = Six502::txs;
                self.addr_mode = Implied
            } //(),
            0xba => {
                self.curr_op = Six502::tsx;
                self.addr_mode = Implied
            } //(),

            // stack ops
            0x08 => {
                self.curr_op = Six502::php;
                self.addr_mode = Implied
            } //(), //implied addressing
            0x28 => {
                self.curr_op = Six502::plp;
                self.addr_mode = Implied
            } //(), //implied addressing
            0x48 => {
                self.curr_op = Six502::pha;
                self.addr_mode = Implied
            } //(), //implied addressing
            0x68 => {
                self.curr_op = Six502::pla;
                self.addr_mode = Implied
            } //(), //implied addressing

            // logical ops
            0x21 => {
                self.curr_op = Six502::and;
                self.addr_mode = XIdxd_Indirect
            } //(XIdxd_Indirect),
            0x25 => {
                self.curr_op = Six502::and;
                self.addr_mode = ZP
            } //(ZP),
            0x29 => {
                self.curr_op = Six502::and;
                self.addr_mode = Immediate
            } //(Immediate),
            0x2d => {
                self.curr_op = Six502::and;
                self.addr_mode = Absolute
            } //(Absolute),
            0x35 => {
                self.curr_op = Six502::and;
                self.addr_mode = ZP_X_Idxd
            } //(ZP_X_Idxd),
            0x31 => {
                self.curr_op = Six502::and;
                self.addr_mode = Indirect_Y_Idxd
            } //(Indirect_Y_Idxd),
            0x39 => {
                self.curr_op = Six502::and;
                self.addr_mode = Abs_Y_Idxd
            } //(Abs_Y_Idxd),
            0x3d => {
                self.curr_op = Six502::and;
                self.addr_mode = Abs_X_Idxd
            } //(Abs_X_Idxd),

            0x01 => {
                self.curr_op = Six502::ora;
                self.addr_mode = XIdxd_Indirect
            } //(XIdxd_Indirect),
            0x05 => {
                self.curr_op = Six502::ora;
                self.addr_mode = ZP
            } //(ZP),
            0x09 => {
                self.curr_op = Six502::ora;
                self.addr_mode = Immediate
            } //(Immediate),
            0x0d => {
                self.curr_op = Six502::ora;
                self.addr_mode = Absolute
            } //(Absolute),
            0x11 => {
                self.curr_op = Six502::ora;
                self.addr_mode = Indirect_Y_Idxd
            } //(Indirect_Y_Idxd),
            0x15 => {
                self.curr_op = Six502::ora;
                self.addr_mode = ZP_X_Idxd
            } //(ZP_X_Idxd),
            0x1d => {
                self.curr_op = Six502::ora;
                self.addr_mode = Abs_X_Idxd
            } //(Abs_X_Idxd),
            0x19 => {
                self.curr_op = Six502::ora;
                self.addr_mode = Abs_Y_Idxd
            } //(Abs_Y_Idxd),

            0x41 => {
                self.curr_op = Six502::eor;
                self.addr_mode = XIdxd_Indirect
            } //(XIdxd_Indirect),
            0x45 => {
                self.curr_op = Six502::eor;
                self.addr_mode = ZP
            } //(ZP),
            0x49 => {
                self.curr_op = Six502::eor;
                self.addr_mode = Immediate
            } //(Immediate),
            0x4d => {
                self.curr_op = Six502::eor;
                self.addr_mode = Absolute
            } //(Absolute),
            0x51 => {
                self.curr_op = Six502::eor;
                self.addr_mode = Indirect_Y_Idxd
            } //(Indirect_Y_Idxd),
            0x55 => {
                self.curr_op = Six502::eor;
                self.addr_mode = ZP_X_Idxd
            } //(ZP_X_Idxd),
            0x5d => {
                self.curr_op = Six502::eor;
                self.addr_mode = Abs_X_Idxd
            } //(Abs_X_Idxd),
            0x59 => {
                self.curr_op = Six502::eor;
                self.addr_mode = Abs_Y_Idxd
            } //(Abs_Y_Idxd),

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
            0x61 => {
                self.curr_op = Six502::adc;
                self.addr_mode = XIdxd_Indirect
            } //(XIdxd_Indirect),
            0x65 => {
                self.curr_op = Six502::adc;
                self.addr_mode = ZP
            } //(ZP),
            0x69 => {
                self.curr_op = Six502::adc;
                self.addr_mode = Immediate
            } //(Immediate),
            0x6d => {
                self.curr_op = Six502::adc;
                self.addr_mode = Absolute
            } //(Absolute),
            0x71 => {
                self.curr_op = Six502::adc;
                self.addr_mode = Indirect_Y_Idxd
            } //(Indirect_Y_Idxd),
            0x75 => {
                self.curr_op = Six502::adc;
                self.addr_mode = ZP_X_Idxd
            } //(ZP_X_Idxd),
            0x79 => {
                self.curr_op = Six502::adc;
                self.addr_mode = Abs_Y_Idxd
            } //(Abs_Y_Idxd),
            0x7d => {
                self.curr_op = Six502::adc;
                self.addr_mode = Abs_X_Idxd
            } //(Abs_X_Idxd),

            0xe1 => {
                self.curr_op = Six502::sbc;
                self.addr_mode = XIdxd_Indirect
            } //(XIdxd_Indirect),
            0xe5 => {
                self.curr_op = Six502::sbc;
                self.addr_mode = ZP
            } //(ZP),
            0xe9 => {
                self.curr_op = Six502::sbc;
                self.addr_mode = Immediate
            } //(Immediate),
            0xed => {
                self.curr_op = Six502::sbc;
                self.addr_mode = Absolute
            } //(Absolute),
            0xf1 => {
                self.curr_op = Six502::sbc;
                self.addr_mode = Indirect_Y_Idxd
            } //(Indirect_Y_Idxd),
            0xf5 => {
                self.curr_op = Six502::sbc;
                self.addr_mode = ZP_X_Idxd
            } //(ZP_X_Idxd),
            0xf9 => {
                self.curr_op = Six502::sbc;
                self.addr_mode = Abs_Y_Idxd
            } //(Abs_Y_Idxd),
            0xfd => {
                self.curr_op = Six502::sbc;
                self.addr_mode = Abs_X_Idxd
            } //(Abs_X_Idxd),

            //incrs and decrs
            0xe6 => {
                self.curr_op = Six502::inc;
                self.addr_mode = ZP
            } //(ZP),
            0xee => {
                self.curr_op = Six502::inc;
                self.addr_mode = Absolute
            } //(Absolute),
            0xf6 => {
                self.curr_op = Six502::inc;
                self.addr_mode = ZP_X_Idxd
            } //(ZP_X_Idxd),
            0xfe => {
                self.curr_op = Six502::inc;
                self.addr_mode = Abs_X_Idxd
            } //(Abs_X_Idxd),

            0xc6 => {
                self.curr_op = Six502::dec;
                self.addr_mode = ZP
            } //(ZP),
            0xce => {
                self.curr_op = Six502::dec;
                self.addr_mode = Absolute
            } //(Absolute),
            0xd6 => {
                self.curr_op = Six502::dec;
                self.addr_mode = ZP_X_Idxd
            } //(ZP_X_Idxd),
            0xde => {
                self.curr_op = Six502::dec;
                self.addr_mode = Abs_X_Idxd
            } //(Abs_X_Idxd),

            0xe8 => {
                self.curr_op = Six502::inx;
                self.addr_mode = Implied
            } //(),
            0xca => {
                self.curr_op = Six502::dex;
                self.addr_mode = Implied
            } //(),
            0xc8 => {
                self.curr_op = Six502::iny;
                self.addr_mode = Implied
            } //(),
            0x88 => {
                self.curr_op = Six502::dey;
                self.addr_mode = Implied
            } //(),

            // shifts
            0x26 => {
                self.curr_op = Six502::rol;
                self.addr_mode = ZP
            } //(ZP),
            0x2a => {
                self.curr_op = Six502::rol;
                self.addr_mode = Accumulator
            } //(Accumulator),
            0x2e => {
                self.curr_op = Six502::rol;
                self.addr_mode = Absolute
            } //(Absolute),
            0x36 => {
                self.curr_op = Six502::rol;
                self.addr_mode = ZP_X_Idxd
            } //(ZP_X_Idxd),
            0x3e => {
                self.curr_op = Six502::rol;
                self.addr_mode = Abs_X_Idxd
            } //(Abs_X_Idxd),

            0x66 => {
                self.curr_op = Six502::ror;
                self.addr_mode = ZP
            } //(ZP),
            0x6a => {
                self.curr_op = Six502::ror;
                self.addr_mode = Accumulator
            } //(Accumulator),
            0x6e => {
                self.curr_op = Six502::ror;
                self.addr_mode = Absolute
            } //(Absolute),
            0x76 => {
                self.curr_op = Six502::ror;
                self.addr_mode = ZP_X_Idxd
            } //(ZP_X_Idxd),
            0x7e => {
                self.curr_op = Six502::ror;
                self.addr_mode = Abs_X_Idxd
            } //(Abs_X_Idxd),

            0x06 => {
                self.curr_op = Six502::asl;
                self.addr_mode = ZP
            } //(ZP),
            0x0e => {
                self.curr_op = Six502::asl;
                self.addr_mode = Absolute
            } //(Absolute),
            0x0a => {
                self.curr_op = Six502::asl;
                self.addr_mode = Accumulator
            } //(Accumulator),
            0x16 => {
                self.curr_op = Six502::asl;
                self.addr_mode = ZP_X_Idxd
            } //(ZP_X_Idxd),
            0x1e => {
                self.curr_op = Six502::asl;
                self.addr_mode = Abs_X_Idxd
            } //(Abs_X_Idxd),

            0x4a => {
                self.curr_op = Six502::lsr;
                self.addr_mode = Accumulator
            } //(Accumulator),
            0x46 => {
                self.curr_op = Six502::lsr;
                self.addr_mode = ZP
            } //(ZP),
            0x4e => {
                self.curr_op = Six502::lsr;
                self.addr_mode = Absolute
            } //(Absolute),
            0x56 => {
                self.curr_op = Six502::lsr;
                self.addr_mode = ZP_X_Idxd
            } //(ZP_X_Idxd),
            0x5e => {
                self.curr_op = Six502::lsr;
                self.addr_mode = Abs_X_Idxd
            } //(Abs_X_Idxd),

            // jumps and calls
            0x4c => {
                self.curr_op = Six502::jmp;
                self.addr_mode = Absolute
            } //(),          // absolute
            0x6c => {
                self.curr_op = Six502::jmp_indirect;
                self.addr_mode = Indirect
            } //(), // indirect

            0x20 => {
                self.curr_op = Six502::jsr;
                self.addr_mode = Absolute
            } //(), // absolute
            0x60 => {
                self.curr_op = Six502::rts;
                self.addr_mode = Implied
            } //(), // implied. In an implied instruction, the data and/or destination is mandatory for the instruction
            0x00 => {
                self.curr_op = Six502::brk;
                self.addr_mode = Implied
            } //(), // implied
            0x40 => {
                self.curr_op = Six502::rti;
                self.addr_mode = Implied
            } //(), // implied

            // branches
            0x10 => {
                self.curr_op = Six502::bpl;
                self.addr_mode = Relative
            } //(), // relative The byte after the opcode is the branch offset.
            0x30 => {
                self.curr_op = Six502::bmi;
                self.addr_mode = Relative
            } //(), // relative
            0x50 => {
                self.curr_op = Six502::bvc;
                self.addr_mode = Relative
            } //(), // relative
            0x70 => {
                self.curr_op = Six502::bvs;
                self.addr_mode = Relative
            } //(), // relative
            0x90 => {
                self.curr_op = Six502::bcc;
                self.addr_mode = Relative
            } //(), // relative
            0xb0 => {
                self.curr_op = Six502::bcs;
                self.addr_mode = Relative
            } //(), // relative
            0xd0 => {
                self.curr_op = Six502::bne;
                self.addr_mode = Relative
            } //(), // relative
            0xf0 => {
                self.curr_op = Six502::beq;
                self.addr_mode = Relative
            } //(), // relative

            // status flag changes
            0x18 => {
                self.curr_op = Six502::clc;
                self.addr_mode = Implied
            } // (), // implied. In an implied instruction, the data and/or destination is mandatory for the instruction
            0x38 => {
                self.curr_op = Six502::sec;
                self.addr_mode = Implied
            } // (); self.addr_mode = Implied} // implied
            0x58 => {
                self.curr_op = Six502::cli;
                self.addr_mode = Implied
            } // (), // implied
            0x78 => {
                self.curr_op = Six502::sei;
                self.addr_mode = Implied
            } // (), // implied
            0xb8 => {
                self.curr_op = Six502::clv;
                self.addr_mode = Implied
            } // (), // implied
            0xd8 => {
                self.curr_op = Six502::cld;
                self.addr_mode = Implied
            } // (), // implied
            0xf8 => {
                self.curr_op = Six502::sed;
                self.addr_mode = Implied
            } // (), // implied

            // no-op
            0xea => {
                self.curr_op = Six502::nop;
                self.addr_mode = Implied
            } //(Implied),

            _ => unimplemented!("op not unimplemented: {}", self.curr_op_num),
        };
    }
}
