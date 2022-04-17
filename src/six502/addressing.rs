#![allow(dead_code, unused_variables, unused_imports)]

use std::{
    ops::{AddAssign, BitOrAssign, Index, RangeBounds, Shl, Shr},
    simd::u32x16,
};

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    Accumulator,
    Absolute,
    ZeroPage, //This type of addressing is called “zero page” - only the first page (the first 256 bytes) of memory is accessible
    ZeroPage_X,
    ZeroPage_Y,
    Absolute_X,
    Absolute_Y,
    IndexedIndirect,
    IndirectIndexed,
}

impl AddressingMode {
    fn load(&self, cpu: &mut Six502) -> u8 {
        match self {
            AddressingMode::Immediate => cpu.load_u8_bump_pc(),
            AddressingMode::Accumulator => cpu.a,
            AddressingMode::Absolute => cpu.load_u8(cpu.load_u16_bump_pc()),
            AddressingMode::Absolute_X => cpu.load_u8(cpu.load_u16_bump_pc() + (cpu.x as u16)),
            AddressingMode::Absolute_Y => cpu.load_u8(cpu.load_u16_bump_pc() + (cpu.y as u16)),
            AddressingMode::ZeroPage => cpu.load_u8(cpu.load_u8_bump_pc() as u16),
            AddressingMode::ZeroPage_X => cpu.load_u8((cpu.load_u8_bump_pc() + cpu.x) as u16),
            AddressingMode::ZeroPage_Y => cpu.load_u8((cpu.load_u8_bump_pc() + cpu.y) as u16),
            AddressingMode::IndexedIndirect => {
                let v = cpu.load_u8_bump_pc();
                let x = self.x;
                cpu.load_u8(cpu.load_u16(v + x as u16))
            }
            AddressingMode::IndirectIndexed => {
                let v = cpu.load_u8_bump_pc();
                let y = self.y;
                cpu.load_u8(cpu.load_u16(v + y as u16))
            }
        }
    }
    fn store(&self, cpu: &mut Six502, v: u8) {
        match self {
            AddressingMode::Immediate => {} // do nothing
            AddressingMode::Accumulator => cpu.a = v,
            AddressingMode::Absolute => cpu.store_u8(cpu.load_u16_bump_pc(), v),
            AddressingMode::Absolute_X => cpu.store_u8(cpu.load_u16_bump_pc() + (cpu.x as u16), v),
            AddressingMode::Absolute_Y => cpu.store_u8(cpu.load_u16_bump_pc() + (cpu.y as u16), v),
            AddressingMode::ZeroPage => cpu.store_u8(cpu.load_u8_bump_pc() as u16, v),
            AddressingMode::ZeroPage_X => cpu.store_u8((cpu.load_u8_bump_pc() + cpu.x) as u16, v),
            AddressingMode::ZeroPage_Y => cpu.store_u8((cpu.load_u8_bump_pc() + cpu.y) as u16, v),
            AddressingMode::IndexedIndirect => {
                let val = cpu.load_u8_bump_pc();
                let x = self.x;
                cpu.store_u8(cpu.load_u16(val + x as u16), v)
            }
            AddressingMode::IndirectIndexed => {
                let val = cpu.load_u8_bump_pc();
                let y = self.y;
                cpu.store_u8(cpu.load_u16(val + y as u16), v)
            }
        }
    }
}

impl Six502 {
    fn load_u16(&self, addr: u16) -> u16 {
        u16::from_be_bytes(
            self.ram[(addr as usize)..=(addr + 1) as usize]
                .try_into()
                .expect("It is certainly 2"),
        )
    }

    fn load_u8_bump_pc(&mut self) -> u8 {
        let addr = self.pc;
        self.pc += 1;
        self.load_u8(addr)
    }

    fn load_u16_bump_pc(&mut self) -> u16 {
        let addr = self.pc;
        self.pc += 2;
        self.load_u16(addr)
    }

    fn store_u16(&mut self, addr: u16, v: u16) {
        self.store_u8(addr, (v >> 8) as u8);
        self.store_u8(addr + 1, (v & 0x00FF) as u8);
    }

    fn update_zero_neg_flags(&mut self, v: u8) {
        if v == 0 {
            self.flag_on(flags::ZERO);
        }

        if v & 0x80 != 0 {
            self.flag_on(flags::NEGATIVE);
        }
    }
}
