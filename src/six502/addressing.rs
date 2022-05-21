#![allow(dead_code, unused_variables, unused_imports)]

use super::flags;
use super::Six502;
use crate::bus::{ByteAccess, WordAccess};
use std::ops::{AddAssign, BitOrAssign, Index, RangeBounds, Shl, Shr};

/// [reference](https://www.masswerk.at/6502/6502_instruction_set.html)
/// The 6502 has the ability to do indexed addressing, where the X or Y register is used as an extra offset to the address being accessed
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    // OPC means `opcode`.
    // operand is the accumulator. single byte instruction
    Accumulator,

    // OPC $LLHH: operand is address $HHLL (i.e. read little-endian)
    Absolute,

    // OPC $LLHH,X: operand is address; effective address is address incremented by X with carry
    Abs_X_Idxd,

    // OPC $LLHH,Y: operand is address; effective address is address incremented by Y with carry
    Abs_Y_Idxd,

    // OPC #$BB: operand is the byte BB, as is
    Immediate,

    // OPC: operand is implied, not specified inline
    // Implied

    // OPC ($LLHH): operand is address; effective address is contents of word at address: C.w($HHLL)
    // Indirect, // Indirect was excluded because it yields a u16 value and is only useful in the `jmpi` instruction

    // operand is zeropage address; effective address is word in (LL + X, LL + X + 1), inc. without carry: C.w($00LL + X)
    XIdxd_Indirect,

    // operand is zeropage address; effective address is word in (LL, LL + 1) incremented by Y with carry: C.w($00LL) + Y
    Indirect_Y_Idxd,
    //Relative

    //This type of addressing is called “zero page” - only the first page (the first 256 bytes) of memory is accessible
    ZP,

    // OPC $LL,X    operand is zeropage address; effective address is address incremented by X without carry
    ZP_X_Idxd,

    // OPC $LL,Y    operand is zeropage address; effective address is address incremented by Y without carry
    ZP_Y_Idxd,
}

impl AddressingMode {
    /// load loads a byte from memory based on the addressing mode. It returns a tuple; the byte being fetched, and a boolean
    /// indicating if there is a page cross while loading the byte.
    pub(super) fn load(&self, cpu: &mut Six502) -> (u8, bool) {
        match self {
            AddressingMode::Accumulator => (cpu.a, false),
            AddressingMode::Absolute => (cpu.load_u8(cpu.load_u16_bump_pc()), false),
            AddressingMode::Abs_X_Idxd => {
                // with carry
                let op = cpu.load_u16_bump_pc(); // op means operand
                let lb_op = op as u8;
                // chec if it'll overflow into the zero page
                let (_, carry) = lb_op.overflowing_add(cpu.x);
                (cpu.load_u8(op + (cpu.x as u16)), carry)
            }
            AddressingMode::Abs_Y_Idxd => {
                let op = cpu.load_u16_bump_pc();
                let lb_op = op as u8;
                let (_, carry) = lb_op.overflowing_add(cpu.y);
                (cpu.load_u8(op + (cpu.y as u16)), carry)
            }

            AddressingMode::Immediate => (cpu.load_u8_bump_pc(), false),
            // AddressingMode::Indirect => {
            //     let op = cpu.load_u16_bump_pc();
            //     let lo = cpu.load_u8(op);
            //     let hi = cpu.load_u8((op & 0xff00) | ((op + 1) & 0x00ff));
            //     (u16::from_le_bytes([lo, hi]), false)
            // }
            AddressingMode::ZP => (cpu.load_u8(cpu.load_u8_bump_pc() as u16), false), //without carry
            AddressingMode::ZP_X_Idxd => (
                cpu.load_u8((cpu.load_u8_bump_pc().wrapping_add(cpu.x)) as u16),
                false,
            ), //without carry. that's why we add the `u8`s before converting to `u16`, so it won't carry into the high-byte
            AddressingMode::ZP_Y_Idxd => (
                cpu.load_u8((cpu.load_u8_bump_pc().wrapping_add(cpu.y)) as u16),
                false,
            ), //without carry
            AddressingMode::XIdxd_Indirect => {
                let v = cpu.load_u8_bump_pc();
                let addr = cpu.x + v;
                (cpu.load_u8(cpu.load_u16_no_carry(addr)), false) // without carry
            }
            AddressingMode::Indirect_Y_Idxd => {
                let y = cpu.y;
                let v = cpu.load_u8_bump_pc();
                // (cpu.load_u8(v as u16) as u16 | (cpu.load_u8((v + 1) as u16) as u16) << 8)
                (cpu.load_u8(cpu.load_u16_no_carry(v) + y as u16), false) //with carry
            }
        }
    }

    pub(super) fn store(&self, cpu: &mut Six502, v: u8) -> bool {
        match self {
            AddressingMode::Accumulator => {
                cpu.a = v;
                false
            }
            AddressingMode::Absolute => {
                cpu.store_u8(cpu.load_u16_bump_pc(), v);
                false
            }

            AddressingMode::Abs_X_Idxd => {
                let op = cpu.load_u16_bump_pc();
                let lb_op = op as u8;
                let (_, carry) = lb_op.overflowing_add(cpu.x);
                cpu.store_u8(cpu.load_u16_bump_pc() + (cpu.x as u16), v);
                carry
            }
            AddressingMode::Abs_Y_Idxd => {
                let op = cpu.load_u16_bump_pc();
                let lb_op = op as u8; // truncates
                let (_, carry) = lb_op.overflowing_add(cpu.y);
                cpu.store_u8(cpu.load_u16_bump_pc() + (cpu.y as u16), v);
                carry
            }

            AddressingMode::Immediate => false, // do nothing
            // AddressingMode::Indirect => false,
            AddressingMode::ZP => {
                cpu.store_u8(cpu.load_u8_bump_pc() as u16, v);
                false
            }

            AddressingMode::ZP_X_Idxd => {
                cpu.store_u8((cpu.load_u8_bump_pc() + cpu.x) as u16, v);
                false
            }
            AddressingMode::ZP_Y_Idxd => {
                cpu.store_u8((cpu.load_u8_bump_pc() + cpu.y) as u16, v);
                false
            }

            AddressingMode::XIdxd_Indirect => {
                let val = cpu.load_u8_bump_pc();
                let addr = cpu.x.wrapping_add(val);
                cpu.store_u8(cpu.load_u16_no_carry(addr), v);
                false
            }
            AddressingMode::Indirect_Y_Idxd => {
                let v = cpu.load_u8_bump_pc();
                let y = cpu.y;
                cpu.store_u8(cpu.load_u16_no_carry(v) + y as u16, v);
                false
            }
        }
    }
}
