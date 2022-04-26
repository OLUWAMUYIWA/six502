#![allow(dead_code, unused_variables, unused_imports)]

use super::flags;
use super::Six502;
use std::ops::{AddAssign, BitOrAssign, Index, RangeBounds, Shl, Shr};

/// [reference](https://www.masswerk.at/6502/6502_instruction_set.html)
/// The 6502 has the ability to do indexed addressing, where the X or Y register is used as an extra offset to the address being accessed
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    // OPC means `opcode`.
    // operand is the accumulator. sinle byte instruction
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
    Indirect,

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
    pub(super) fn load(&self, cpu: &mut Six502) -> u8 {
        match self {
            AddressingMode::Accumulator => cpu.a,
            AddressingMode::Absolute => cpu.load_u8(cpu.load_u16_bump_pc()),
            AddressingMode::Abs_X_Idxd => cpu.load_u8(cpu.load_u16_bump_pc() + (cpu.x as u16)), // with carry
            AddressingMode::Abs_Y_Idxd => cpu.load_u8(cpu.load_u16_bump_pc() + (cpu.y as u16)), //with carry
            AddressingMode::Immediate => cpu.load_u8_bump_pc(),
            AddressingMode::Indirect => {}
            AddressingMode::ZP => cpu.load_u8(cpu.load_u8_bump_pc() as u16), //without carry
            AddressingMode::ZP_X_Idxd => cpu.load_u8((cpu.load_u8_bump_pc() + cpu.x) as u16), //without carry. that's why we add the `u8`s before converting to `u16`, so it won't carry into the high-byte
            AddressingMode::ZP_Y_Idxd => cpu.load_u8((cpu.load_u8_bump_pc() + cpu.y) as u16), //without carry
            AddressingMode::XIdxd_Indirect => {
                let x = cpu.x;
                let v = cpu.load_u8_bump_pc();
                let addr = x + v;
                cpu.load_u8(
                    cpu.load_u8(addr as u16) as u16 | (cpu.load_u8((addr + 1) as u16) as u16) << 8,
                ) // without carry
            }
            AddressingMode::Indirect_Y_Idxd => {
                let y = cpu.y;
                let v = cpu.load_u8_bump_pc();
                cpu.load_u8(
                    (cpu.load_u8(v as u16) as u16 | (cpu.load_u8((v + 1) as u16) as u16) << 8)
                        + y as u16,
                ) //with carry
            }
        }
    }
    pub(super) fn store(&self, cpu: &mut Six502, v: u8) {
        match self {
            AddressingMode::Accumulator => cpu.a = v,
            AddressingMode::Absolute => cpu.store_u8(cpu.load_u16_bump_pc(), v),
            AddressingMode::Abs_X_Idxd => cpu.store_u8(cpu.load_u16_bump_pc() + (cpu.x as u16), v),
            AddressingMode::Abs_Y_Idxd => cpu.store_u8(cpu.load_u16_bump_pc() + (cpu.y as u16), v),
            AddressingMode::Immediate => {} // do nothing
            AddressingMode::Indirect => {}
            AddressingMode::ZP => cpu.store_u8(cpu.load_u8_bump_pc() as u16, v),
            AddressingMode::ZP_X_Idxd => cpu.store_u8((cpu.load_u8_bump_pc() + cpu.x) as u16, v),
            AddressingMode::ZP_Y_Idxd => cpu.store_u8((cpu.load_u8_bump_pc() + cpu.y) as u16, v),
            AddressingMode::XIdxd_Indirect => {
                let val = cpu.load_u8_bump_pc();
                let x = cpu.x;
                let addr = x + val;
                cpu.store_u8(
                    cpu.load_u8(addr as u16) as u16 | (cpu.load_u8((addr + 1) as u16) as u16) << 8,
                    v,
                )
            }
            AddressingMode::Indirect_Y_Idxd => {
                let v = cpu.load_u8_bump_pc();
                let y = cpu.y;
                cpu.store_u8(
                    (cpu.load_u8(v as u16) as u16 | (cpu.load_u8((v + 1) as u16) as u16) << 8)
                        + y as u16,
                    v,
                )
            }
        }
    }
}
