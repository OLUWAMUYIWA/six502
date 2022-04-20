use addressing::AddressingMode::*;
mod addressing;
mod memory;
mod opcodes;
mod util;

const STACK_OFFSET: usize = 0x100;
const NMI_VECTOR: usize = 0xfffa;
const RESET_VECTOR: usize = 0xfffc;
const IRQ_VECTOR: usize = 0xfffe;
const BRK_VECTOR: u16 = 0xfffe;

bitflags::bitflags! {
    pub struct Flags: u8 {
        const CARRY = 0b00000001;
        const ZERO = 0b00000010; //set to 1 on equality
        const IRQ = 0b00000100;
        const DECIMAL= 0b00001000;
        const BREAK =    0b00010000;
        //unused bit pos
        const OVERFLOW = 0b01000000;
        const NEGATIVE= 0b10000000;
    }
}
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

//comeback: where is jmpi

use std::collections::HashMap;

use self::{addressing::AddressingMode, memory::Ram};

pub struct Six502 {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    s: u8,
    flags: u8,
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
            flags: 0x24,
            ram: Ram::new(),
        }
    }
    pub fn step(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    fn load_u16(&self, addr: u16) -> u16 {
        u16::from_be_bytes(
            self.ram[(addr as usize)..=(addr + 1) as usize]
                .try_into()
                .expect("It is certainly 2"),
        )
    }

    pub(super) fn load_u8_bump_pc(&mut self) -> u8 {
        let addr = self.pc;
        self.pc += 1;
        self.load_u8(addr)
    }

    pub(super) fn load_u16_bump_pc(&mut self) -> u16 {
        let addr = self.pc;
        self.pc += 2;
        self.load_u16(addr)
    }

    pub(super) fn store_u16(&mut self, addr: u16, v: u16) {
        self.store_u8(addr, (v >> 8) as u8);
        self.store_u8(addr + 1, (v & 0x00FF) as u8);
    }

    pub(super) fn update_zero_neg_flags(&mut self, v: u8) {
        if v == 0 {
            self.flag_on(flags::ZERO);
        }

        if v & 0x80 != 0 {
            self.flag_on(flags::NEGATIVE);
        }
    }
}

lazy_static::lazy_static! {
    static ref CYCLES: [u8; 256] = [
        //       0, 1, 2, 3, 4, 5, 6, 7, 8, 9, A, B, C, D, E, F
        /*0x00*/ 7, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6,
        /*0x10*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
        /*0x20*/ 6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6,
        /*0x30*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
        /*0x40*/ 6, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6,
        /*0x50*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
        /*0x60*/ 6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6,
        /*0x70*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
        /*0x80*/ 2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
        /*0x90*/ 2, 6, 2, 6, 4, 4, 4, 4, 2, 5, 2, 5, 5, 5, 5, 5,
        /*0xA0*/ 2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
        /*0xB0*/ 2, 5, 2, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4,
        /*0xC0*/ 2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
        /*0xD0*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
        /*0xE0*/ 2, 6, 3, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
        /*0xF0*/ 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    ];
}

impl Six502 {
    pub fn exec(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let op = self.load_u8_bump_pc();
        match op {
            // load/stores
            0xa1 => {
                self.lda(XIndexedIndirect);
            }
            0xa5 => {
                self.lda(ZeroPage);
            }
            0xa9 => {
                self.lda(Immediate);
            }
            0xad => {
                self.lda(Absolute);
            }
            0xb1 => {
                self.lda(IndirectYIndexed);
            }
            0xb5 => {
                self.lda(ZeroPage_X);
            }
            0xb9 => {
                self.lda(Absolute_Y);
            }
            0xbd => {
                self.lda(Absolute_X);
            }

            0xa2 => {
                self.ldx(Immediate);
            }
            0xa6 => {
                self.ldx(ZeroPage);
            }
            0xb6 => {
                self.ldx(ZeroPage_Y);
            }
            0xae => {
                self.ldx(Absolute);
            }
            0xbe => {
                self.ldx(Absolute_Y);
            }

            0xa0 => {
                self.ldy(Immediate);
            }
            0xa4 => {
                self.ldy(ZeroPage);
            }
            0xb4 => {
                self.ldy(ZeroPage_X);
            }
            0xac => {
                self.ldy(Absolute);
            }
            0xbc => {
                self.ldy(Absolute_X);
            }

            0x85 => {
                self.sta(ZeroPage);
            }
            0x95 => {
                self.sta(ZeroPage_X);
            }
            0x8d => {
                self.sta(Absolute);
            }
            0x9d => {
                self.sta(Absolute_X);
            }
            0x99 => {
                self.sta(Absolute_Y);
            }
            0x81 => {
                self.sta(XIndexedIndirect);
            }
            0x91 => {
                self.sta(IndirectYIndexed);
            }

            0x86 => {
                self.stx(ZeroPage);
            }
            0x96 => {
                self.stx(ZeroPage_Y);
            }
            0x8e => {
                self.stx(Absolute);
            }

            0x84 => {
                self.sty(ZeroPage);
            }
            0x94 => {
                self.sty(ZeroPage_X);
            }
            0x8c => {
                self.sty(Absolute);
            }

            // comparisons
            0xc9 => {
                self.cmp(Immediate);
            }
            0xc5 => {
                self.cmp(ZeroPage);
            }
            0xd5 => {
                self.cmp(ZeroPage_X);
            }
            0xcd => {
                self.cmp(Absolute);
            }
            0xdd => {
                self.cmp(Absolute_X);
            }
            0xd9 => {
                self.cmp(Absolute_Y);
            }
            0xc1 => {
                self.cmp(XIndexedIndirect);
            }
            0xd1 => {
                self.cmp(IndirectYIndexed);
            }

            0xe0 => {
                self.cpx(Immediate);
            }
            0xe4 => {
                self.cpx(ZeroPage);
            }
            0xec => {
                self.cpx(Absolute);
            }

            0xc0 => {
                self.cpy(Immediate);
            }
            0xc4 => {
                self.cpy(ZeroPage);
            }
            0xcc => {
                self.cpy(Absolute);
            }

            // transfers
            0xaa => this.tax(),
            0xa8 => this.tay(),
            0x8a => this.txa(),
            0x98 => this.tya(),
            0x9a => this.txs(),
            0xba => this.tsx(),

            // stack ops
            0x48 => this.pha(),
            0x68 => this.pla(),
            0x08 => this.php(),
            0x28 => this.plp(),

            // logical ops
            0x29 => {
                self.and(Immediate);
            }
            0x25 => {
                self.and(ZeroPage);
            }
            0x35 => {
                self.and(ZeroPage_X);
            }
            0x2d => {
                self.and(Absolute);
            }
            0x3d => {
                self.and(Absolute_X);
            }
            0x39 => {
                self.and(Absolute_Y);
            }
            0x21 => {
                self.and(XIndexedIndirect);
            }
            0x31 => {
                self.and(IndirectYIndexed);
            }

            0x09 => {
                self.ora(Immediate);
            }
            0x05 => {
                self.ora(ZeroPage);
            }
            0x15 => {
                self.ora(ZeroPage_X);
            }
            0x0d => {
                self.ora(Absolute);
            }
            0x1d => {
                self.ora(Absolute_X);
            }
            0x19 => {
                self.ora(Absolute_Y);
            }
            0x01 => {
                self.ora(XIndexedIndirect);
            }
            0x11 => {
                self.ora(IndirectYIndexed);
            }

            0x49 => {
                self.eor(Immediate);
            }
            0x45 => {
                self.eor(ZeroPage);
            }
            0x55 => {
                self.eor(ZeroPage_X);
            }
            0x4d => {
                self.eor(Absolute);
            }
            0x5d => {
                self.eor(Absolute_X);
            }
            0x59 => {
                self.eor(Absolute_Y);
            }
            0x41 => {
                self.eor(XIndexedIndirect);
            }
            0x51 => {
                self.eor(IndirectYIndexed);
            }

            0x24 => {
                self.bit(ZeroPage);
            }
            0x2c => {
                self.bit(Absolute);
            }

            // arithmetic ops
            0x69 => {
                self.adc(Immediate);
            }
            0x65 => {
                self.adc(ZeroPage);
            }
            0x75 => {
                self.adc(ZeroPage_X);
            }
            0x6d => {
                self.adc(Absolute);
            }
            0x7d => {
                self.adc(Absolute_X);
            }
            0x79 => {
                self.adc(Absolute_Y);
            }
            0x61 => {
                self.adc(XIndexedIndirect);
            }
            0x71 => {
                self.adc(IndirectYIndexed);
            }

            0xe9 => {
                self.sbc(Immediate);
            }
            0xe5 => {
                self.sbc(ZeroPage);
            }
            0xf5 => {
                self.sbc(ZeroPage_X);
            }
            0xed => {
                self.sbc(Absolute);
            }
            0xfd => {
                self.sbc(Absolute_X);
            }
            0xf9 => {
                self.sbc(Absolute_Y);
            }
            0xe1 => {
                self.sbc(XIndexedIndirect);
            }
            0xf1 => {
                self.sbc(IndirectYIndexed);
            }

            //incrs and decrs
            0xe6 => self.inc(ZeroPage),
            0xf6 => self.inc(ZeroPage_X),
            0xee => self.inc(Absolute),
            0xfe => self.inc(Absolute_X),

            0xc6 => self.dec(ZeroPage),
            0xd6 => self.dec(ZeroPage_X),
            0xce => self.dec(Absolute),
            0xde => self.dec(Absolute_X),

            0xe8 => this.inx(),
            0xca => this.dex(),
            0xc8 => this.iny(),
            0x88 => this.dey(),

            // shifts
            0x2a => {
                self.rol(Accumulator);
            }
            0x26 => {
                self.rol(ZeroPage);
            }
            0x36 => {
                self.rol(ZeroPage_X);
            }
            0x2e => {
                self.rol(Absolute);
            }
            0x3e => {
                self.rol(Absolute_X);
            }

            0x6a => {
                self.ror(Accumulator);
            }
            0x66 => {
                self.ror(ZeroPage);
            }
            0x76 => {
                self.ror(ZeroPage_X);
            }
            0x6e => {
                self.ror(Absolute);
            }
            0x7e => {
                self.ror(Absolute_X);
            }

            0x0a => self.asl(Accumulator),
            0x06 => self.asl(ZeroPage),
            0x16 => self.asl(ZeroPage_X),
            0x0e => self.asl(Absolute),
            0x1e => self.asl(Absolute_X),

            0x4a => self.lsr(Accumulator),
            0x46 => self.lsr(ZeroPage),
            0x56 => self.lsr(ZeroPage_X),
            0x4e => self.lsr(Absolute),
            0x5e => self.lsr(Absolute_X),

            // jumps and calls
            0x4c => this.jmp(),
            0x6c => this.jmpi(),

            0x20 => this.jsr(),
            0x60 => this.rts(),
            0x00 => this.brk(),
            0x40 => this.rti(),

            // branches
            0x10 => this.bpl(),
            0x30 => this.bmi(),
            0x50 => this.bvc(),
            0x70 => this.bvs(),
            0x90 => this.bcc(),
            0xb0 => this.bcs(),
            0xd0 => this.bne(),
            0xf0 => this.beq(),

            // status flag changes
            0x18 => this.clc(),
            0x38 => this.sec(),
            0x58 => this.cli(),
            0x78 => this.sei(),
            0xb8 => this.clv(),
            0xd8 => this.cld(),
            0xf8 => this.sed(),

            // no-op
            0xea => this.nop(),

            _ => unimplemented!("op not unimplemented: {}", op),
        }
        Ok(())
    }
}
