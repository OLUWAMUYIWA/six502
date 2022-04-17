#![allow(dead_code, unused_variables, unused_imports)]

use super::flags;
use super::Six502;
use std::ops::{AddAssign, BitOrAssign, Index, RangeBounds, Shl, Shr};

// https://www.masswerk.at/6502/6502_instruction_set.html

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Accumulator,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Immediate,
    //Implied
    //Indirect
    XIndexedIndirect, // operand is zeropage address; effective address is word in (LL + X, LL + X + 1), inc. without carry: C.w($00LL + X)
    IndirectYIndexed, // operand is zeropage address; effective address is word in (LL, LL + 1) incremented by Y with carry: C.w($00LL) + Y
    //Relative
    ZeroPage, //This type of addressing is called “zero page” - only the first page (the first 256 bytes) of memory is accessible
    ZeroPage_X,
    ZeroPage_Y,
}

impl AddressingMode {
    pub(super) fn load(&self, cpu: &mut Six502) -> u8 {
        match self {
            AddressingMode::Accumulator => cpu.a,
            AddressingMode::Absolute => cpu.load_u8(cpu.load_u16_bump_pc()),
            AddressingMode::Absolute_X => cpu.load_u8(cpu.load_u16_bump_pc() + (cpu.x as u16)), // with carry
            AddressingMode::Absolute_Y => cpu.load_u8(cpu.load_u16_bump_pc() + (cpu.y as u16)), //with carry
            AddressingMode::Immediate => cpu.load_u8_bump_pc(),
            AddressingMode::ZeroPage => cpu.load_u8(cpu.load_u8_bump_pc() as u16), //without carry
            AddressingMode::ZeroPage_X => cpu.load_u8((cpu.load_u8_bump_pc() + cpu.x) as u16), //without carry. that's why we add the `u8`s before converting to `u16`, so it won't carry into the high-byte
            AddressingMode::ZeroPage_Y => cpu.load_u8((cpu.load_u8_bump_pc() + cpu.y) as u16), //without carry
            AddressingMode::XIndexedIndirect => {
                let x = cpu.x;
                let v = cpu.load_u8_bump_pc();
                let addr = x + v;
                cpu.load_u8(
                    cpu.load_u8(addr as u16) as u16 | (cpu.load_u8((addr + 1) as u16) as u16) << 8,
                ) // without carry
            }
            AddressingMode::IndirectYIndexed => {
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
            AddressingMode::Absolute_X => cpu.store_u8(cpu.load_u16_bump_pc() + (cpu.x as u16), v),
            AddressingMode::Absolute_Y => cpu.store_u8(cpu.load_u16_bump_pc() + (cpu.y as u16), v),
            AddressingMode::Immediate => {} // do nothing
            AddressingMode::ZeroPage => cpu.store_u8(cpu.load_u8_bump_pc() as u16, v),
            AddressingMode::ZeroPage_X => cpu.store_u8((cpu.load_u8_bump_pc() + cpu.x) as u16, v),
            AddressingMode::ZeroPage_Y => cpu.store_u8((cpu.load_u8_bump_pc() + cpu.y) as u16, v),
            AddressingMode::XIndexedIndirect => {
                let val = cpu.load_u8_bump_pc();
                let x = cpu.x;
                let addr = x + val;
                cpu.store_u8(
                    cpu.load_u8(addr as u16) as u16 | (cpu.load_u8((addr + 1) as u16) as u16) << 8,
                    v,
                )
            }
            AddressingMode::IndirectYIndexed => {
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
