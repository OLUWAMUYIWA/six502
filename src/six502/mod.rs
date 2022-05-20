use self::{addressing::AddressingMode, memory::Ram};
use addressing::AddressingMode::*;
use bitflags::*;
use lazy_static::lazy_static;
use std::collections::HashMap;

mod addressing;
mod disasm;
mod interrupt;
mod memory;
mod opcodes;
mod util;

const STACK_OFFSET: usize = 0x100;
const NMI_VECTOR: usize = 0xfffa;
const RESET_VECTOR: usize = 0xfffc;
const IRQ_VECTOR: usize = 0xfffe;
const BRK_VECTOR: u16 = 0xfffe;

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
    pub const CARRY: u8 = 1 << 0;
    pub const ZERO: u8 = 1 << 1; //set to 1 on equality
    pub const IRQ: u8 = 1 << 2;
    pub const DECIMAL: u8 = 1 << 3;
    pub const BREAK: u8 = 1 << 4;
    //unused bit pos
    pub const OVERFLOW: u8 = 1 << 6;
    pub const NEGATIVE: u8 = 1 << 7;
}

pub struct Six502 {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    s: u8,
    cy: u64,
    p: u8, // flags
    pub ram: Ram,
}

impl Six502 {
    pub(crate) fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0xc000,
            s: 0xfd,
            cy: u64,
            p: 0x24,
            ram: Ram::new(),
        }
    }
    pub fn step(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub(super) fn update_z(&mut self, v: u8) {
        if v == 0 {
            self.flag_on(flags::ZERO);
        }
    }

    pub(super) fn update_n(&mut self, v: u8) {
        if v & 0x80 != 0 {
            self.flag_on(flags::NEGATIVE);
        }
    }
}

lazy_static! {
    static ref CYCLES: [u8; 256] = [
        //       0, 1, 2, 3, 4, 5, 6, 7, 8, 9, A, B, C, D, E, F
        /*0*/    7, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6,
        /*1*/    2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
        /*2*/    6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6,
        /*3*/    2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
        /*4*/    6, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6,
        /*5*/    2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
        /*6*/    6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6,
        /*7*/    2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
        /*8*/    2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
        /*9*/    2, 6, 2, 6, 4, 4, 4, 4, 2, 5, 2, 5, 5, 5, 5, 5,
        /*A*/    2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
        /*B*/    2, 5, 2, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4,
        /*C*/    2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
        /*D*/    2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
        /*E*/    2, 6, 3, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
        /*F*/    2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    ];
}

impl Six502 {
    pub fn exec(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
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
            0xaa => self.tax(),
            0xa8 => self.tay(),
            0x8a => self.txa(),
            0x98 => self.tya(),
            0x9a => self.txs(),
            0xba => self.tsx(),

            // stack ops
            0x08 => self.php(), //implied addressing
            0x28 => self.plp(), //implied addressing
            0x48 => self.pha(), //implied addressing
            0x68 => self.pla(), //implied addressing

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

            0xe8 => self.inx(),
            0xca => self.dex(),
            0xc8 => self.iny(),
            0x88 => self.dey(),

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
            0x4c => self.jmp(),  // absolute
            0x6c => self.jmpi(), // indirect

            0x20 => self.jsr(), // absolute
            0x60 => self.rts(), // implied. In an implied instruction, the data and/or destination is mandatory for the instruction
            0x00 => self.brk(), // implied
            0x40 => self.rti(), // implied

            // branches
            0x10 => self.bpl(), // relative The byte after the opcode is the branch offset.
            0x30 => self.bmi(), // relative
            0x50 => self.bvc(), // relative
            0x70 => self.bvs(), // relative
            0x90 => self.bcc(), // relative
            0xb0 => self.bcs(), // relative
            0xd0 => self.bne(), // relative
            0xf0 => self.beq(), // relative

            // status flag changes
            0x18 => self.clc(), // implied. In an implied instruction, the data and/or destination is mandatory for the instruction
            0x38 => self.sec(), // implied
            0x58 => self.cli(), // implied
            0x78 => self.sei(), // implied
            0xb8 => self.clv(), // implied
            0xd8 => self.cld(), // implied
            0xf8 => self.sed(), // implied

            // no-op
            0xea => self.nop(),

            _ => unimplemented!("op not unimplemented: {}", op),
        };
        self.cy
            .wrapping_add(CYCLES[op as usize] as u64)
            .wrapping_add(page_cross as u64);

        Ok(())
    }

    pub(super) fn apply_pg_cross(&mut self, over: bool) {
        if over {
            self.cy.wrapping_add(1);
        }
    }
}
