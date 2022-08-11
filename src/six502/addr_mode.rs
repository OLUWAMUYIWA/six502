#![allow(dead_code, unused_variables, unused_imports, unused_parens)]

use super::flags;
use super::Six502;
use crate::bus::{ByteAccess, WordAccess};
use std::ops::{AddAssign, BitOrAssign, Index, RangeBounds, Shl, Shr};

/// [reference](https://www.masswerk.at/6502/6502_instruction_set.html)
/// The 6502 has the ability to do indexed addressing, where the X or Y register is used as an extra offset to the address being accessed
/// The addressing modes of the MCS6500 family can be grouped into two major categories:  Indexed and Non-Indexed Addressing
/// Implied addressing is not encoded here because the opcode usually contains the source and the dest for the op (e.g. tsx). morally, there is no need for loading any value

#[derive(Debug)]
#[allow(non_camel_case_types)]

// Two major kinds of addressing exist.
// 1.Direct addressing: where the address is plainl what is after the opcode. e.g. absolute, zero_page, immediate.
// 2. i.  Indexed addressing uses an address which is computed by means of modifying the address data accessed by.
//        the program counter with an internal register called an index register.
//        e.g. Abs_X_Idxd, Abs_Y_Idxd, ZP_X_Idxd, ZP_Y_Idxd
//    ii. Indirect addressing uses a  computed and stored address which is accessed by
//        an indirect pointer in the programming sequence.
pub enum AddressingMode {
    // OPC means `opcode`.
    // operand is the accumulator. for single byte instructions
    Accumulator,

    // OPC $LLHH: operand is address $HHLL (i.e. read little-endian)
    Absolute,

    // Next two are Absolute Indexed.
    // Absolute indexed address is absolute addressing with an index register added to the absolute address.

    // OPC $LLHH,X: operand is address; effective address is address incremented by X with carry
    Abs_X_Idxd,

    // OPC $LLHH,Y: operand is address; effective address is address incremented by Y with carry
    Abs_Y_Idxd,

    // OPC #$BB: operand is the byte BB, as is.
    Immediate,

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

    // The instruction is just one byte. Addressing is implicit
    Implied,

    // my cause page crossing or not
    Relative,
}

impl AddressingMode {
    /// load loads a byte from memory based on the addressing mode. It returns a tuple; the byte being fetched, and a boolean
    /// indicating if there is a page cross while loading the byte.
    pub(super) fn load(&self, cpu: &mut Six502) -> (u8, bool) {
        match self {
            AddressingMode::Accumulator => {
                cpu.atom(|c| {
                    // comeback
                });
                (cpu.a, false)
            }
            AddressingMode::Absolute => {
                let (mut p1, mut p2, mut v) = (0, 0, 0);
                cpu.atom(|c| p1 = c.load_u8_bump_pc());
                cpu.atom(|c| p2 = c.load_u8_bump_pc());
                cpu.atom(|c| {
                    let addr = u16::from_le_bytes([p1, p2]);
                    v = c.load_u8(addr);
                });

                // let addr = cpu.load_u16_bump_pc();
                (v, false)
            }
            AddressingMode::Abs_X_Idxd => {
                let (mut p1, mut p2, mut v) = (0, 0, 0);
                let mut over = false;
                cpu.atom(|c| p1 = c.load_u8_bump_pc());
                let x = cpu.x;
                cpu.atom(|c| {
                    p2 = c.load_u8_bump_pc();
                    (p1, over) = p1.overflowing_add(x);
                });

                if over {
                    cpu.atom(|c| {
                        p1 += 1;
                        let addr = u16::from_le_bytes([p1, p2]);
                        v = c.load_u8(addr);
                    });
                } else {
                    let addr = u16::from_le_bytes([p1, p2]);
                    v = cpu.load_u8(addr);
                };

                // let op = cpu.load_u16_bump_pc();

                // // check if it'll overflow into the zero page
                // let lb_op = op as u8;
                // let (_, carry) = lb_op.overflowing_add(cpu.x);

                (v, false)
            }
            AddressingMode::Abs_Y_Idxd => {
                let op = cpu.load_u16_bump_pc();

                // check if it'll overflow into the zero page
                let lb_op = op as u8;
                let (_, carry) = lb_op.overflowing_add(cpu.y);
                (cpu.load_u8(op + (cpu.y as u16)), carry)
            }

            AddressingMode::Immediate => {
                let mut v = 0;
                cpu.atom(|c| v = c.load_u8_bump_pc());
                (v, false)
            }
            AddressingMode::ZP => {
                let (mut addr, mut v) = (0, 0);
                cpu.atom(|c| {
                    addr = c.load_u8_bump_pc();
                });
                cpu.atom(|c| {
                    let addr = addr as u16;
                    v = c.load_u8(addr);
                });
                // let addr = cpu.load_u8_bump_pc();
                (v, false)
            }
            //without carry
            AddressingMode::ZP_X_Idxd => {
                let addr = cpu.load_u8_bump_pc();
                (cpu.load_u8((addr.wrapping_add(cpu.x)) as u16), false)
            } //without carry. that's why we add the `u8`s before converting to `u16`, so it won't carry into the high-byte

            //   If the base address plus X or Y exceeds the value that
            //   can be stored in a single byte, no carry is generated, therefore there is no page crossing phenomena
            //    A wrap-around will occur within Page Zero
            AddressingMode::ZP_Y_Idxd => {
                let addr = cpu.load_u8_bump_pc();
                (cpu.load_u8((addr.wrapping_add(cpu.y)) as u16), false)
            }
            // The major use of indexed indirect is in picking up data from a table or list of addresses to perform an operation.
            AddressingMode::XIdxd_Indirect => {
                let v = cpu.load_u8_bump_pc();
                // zero page addition. Never crosses page. wraps around
                let comp = cpu.x.wrapping_add(v);
                let lo_addr = cpu.load_u8(comp as u16);
                let hi_addr = cpu.load_u8((comp + 1) as u16);
                // say comp is 0x05 effective address becomes 0x0605
                let eff_addr = u16::from_le_bytes([lo_addr, hi_addr]);
                (cpu.load_u8(eff_addr), false) // never crosses pae as the indexing is done in the zero page
            }
            AddressingMode::Indirect_Y_Idxd => {
                let y = cpu.y;
                let v = cpu.load_u8_bump_pc();
                let lo_addr = cpu.load_u8(v as u16);
                let hi_addr = cpu.load_u8((v + 1) as u16);
                // say v is 0x05 effective address becomes 0x0605
                let eff_addr = u16::from_le_bytes([lo_addr, hi_addr]);
                let (_, carry) = lo_addr.overflowing_add(y);
                (cpu.load_u8(eff_addr.wrapping_add(y as u16)), carry) // might cross page
            }
            AddressingMode::Implied => {
                let mut v: u8 = 0;
                cpu.atom(|cpu| {
                    v = cpu.load_u8_bump_pc();
                });

                (0, false)
            }
            AddressingMode::Relative => {
                let (mut off) = (0);
                cpu.atom(|c| {
                    off = c.load_u8_bump_pc() as i8 as u16;
                });
                let mut overflowed = false;
                // comeback to deal with page transiions
                cpu.atom(|c| {
                    (c.pc, overflowed) = c.pc.overflowing_add(off);
                });
                if overflowed {
                    cpu.tick();
                }
                (0, false)
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
                let addr = cpu.load_u16_bump_pc();
                cpu.store_u8(addr, v);
                false
            }

            AddressingMode::Abs_X_Idxd => {
                let op = cpu.load_u16_bump_pc();

                // check if it'll overflow into the zero page
                let lb_op = op as u8;
                let (_, carry) = lb_op.overflowing_add(cpu.x);
                let addr = cpu.load_u16_bump_pc();

                cpu.store_u8(addr + (cpu.x as u16), v);
                carry
            }
            AddressingMode::Abs_Y_Idxd => {
                let op = cpu.load_u16_bump_pc();
                // check if it'll overflow into the zero page
                let lb_op = op as u8; // truncates
                let (_, carry) = lb_op.overflowing_add(cpu.y);
                let addr = cpu.load_u16_bump_pc();
                cpu.store_u8(addr + (cpu.y as u16), v);
                carry
            }

            AddressingMode::Immediate => false, // do nothing
            // AddressingMode::Indirect => false,
            AddressingMode::ZP => {
                let addr = cpu.load_u8_bump_pc();
                cpu.store_u8(addr as u16, v);
                false
            }

            //   If the base address plus X or Y exceeds the value that
            //   can be stored in a single byte, no carry is generated, therefore there is no page crossing phenomena
            //    A wrap-around will occur within Page Zero
            AddressingMode::ZP_X_Idxd => {
                let addr = cpu.load_u8_bump_pc();
                cpu.store_u8((addr.wrapping_add(cpu.x)) as u16, v);
                false
            }
            AddressingMode::ZP_Y_Idxd => {
                let addr = cpu.load_u8_bump_pc();
                cpu.store_u8((addr.wrapping_add(cpu.y)) as u16, v);
                false
            }

            AddressingMode::XIdxd_Indirect => {
                let v = cpu.load_u8_bump_pc();
                // zero page addition. Never crosses page. wraps around
                let comp = cpu.x.wrapping_add(v);
                let lo_addr = cpu.load_u8(comp as u16);
                let hi_addr = cpu.load_u8(comp.wrapping_add(1) as u16);
                let eff_addr = u16::from_le_bytes([lo_addr, hi_addr]);
                cpu.store_u8(eff_addr, v);
                false // never crosses page as the indexing is done in the zero page
            }
            AddressingMode::Indirect_Y_Idxd => {
                let v = cpu.load_u8_bump_pc();
                let y = cpu.y;
                let lo_addr = cpu.load_u8(v as u16);
                let hi_addr = cpu.load_u8((v + 1) as u16);
                // say v is 0x05 effective address becomes 0x0605
                let eff_addr = u16::from_le_bytes([lo_addr, hi_addr]);
                let (_, carry) = lo_addr.overflowing_add(y);
                cpu.store_u8(eff_addr.wrapping_add(y as u16), v);
                carry // might cross page
            }
            AddressingMode::Implied => todo!(),
            AddressingMode::Relative => todo!(),
        }
    }
}
