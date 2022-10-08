#![allow(dead_code, unused_variables, unused_imports, unused_parens)]

use super::flags;
use crate::Addressing;
use super::Six502;
use crate::ByteAccess;
use crate::Cpu;
use std::ops::{AddAssign, BitOrAssign, Index, RangeBounds, Shl, Shr};

#[repr(transparent)]
pub(crate) struct AddrBus(u16);

/// [reference](https://www.masswerk.at/6502/6502_instruction_set.html)
/// The 6502 has the ability to do indexed addressing, where the X or Y register is used as an extra offset to the address being accessed
/// The addressing modes of the MCS6500 family can be grouped into two major categories:  Indexed and Non-Indexed Addressing
/// Implied addressing is not encoded here because the opcode usually contains the source and the dest for the op (e.g. tsx). morally, there is no need for loading any value
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
// Two major kinds of addressing exist.
// 1.Direct addressing: where the address is plainl what is after the opcode. e.g. absolute, zero_page, immediate.
// 2. i.  Indexed addressing uses an address which is computed by means of modifying the address data accessed by.
//        the program counter with an internal register called an index register.
//        e.g. Abs_X_Idxd, Abs_Y_Idxd, ZP_X_Idxd, ZP_Y_Idxd
//    ii. Indirect addressing uses a  computed and stored address which is accessed by
//        an indirect pointer in the programming sequence.
pub enum AddressingMode {
    /// Accumulator
    /// OPC means `opcode`.
    /// operand is the accumulator. for single byte instructions
    Acc_Addrs,

    /// Absolute
    ///  OPC $LLHH: operand is address $HHLL (i.e. read little-endian)
    Abs_Addrs,

    // Next two are Absolute Indexed.
    // Absolute indexed address is absolute addressing with an index register added to the absolute address.

    /// Absolute__Indexed
    ///  OPC $LLHH,X: operand is address; effective address is address incremented by X with carry
    AbsX_Idxd,

    /// Absolute_Y_Indexed
    ///  OPC $LLHH,Y: operand is address; effective address is address incremented by Y with carry
    AbsY_Idxd,

    /// Immediate addressing
    /// OPC #$BB: operand is the byte BB, as is.
    Immediate,

    /// Indirect
    Ind_Addrs,

    /// X_Indexed_Indirect
    /// OPC ($LLHH): operand is address; effective address is contents of word at address: C.w($HHLL)
    /// Indirect, // Indirect was excluded because it yields a u16 value and is only useful in the `jmpi` instruction
    /// operand is zeropage address; effective address is word in (LL + X, LL + X + 1), inc. without carry: C.w($00LL + X)
    X_Idx_Ind,

    /// Indirect_Y_Indexed
    ///  operand is zeropage address; effective address is word in (LL, LL + 1) incremented by Y with carry: C.w($00LL) + Y
    Ind_Y_Idx,

    ///ero_Page
    /// This type of addressing is called “zero page” - only the first page (the first 256 bytes) of memory is accessible
    Zero_Page,

    // OPC $LL,X    operand is zeropage address; effective address is address incremented by X without carry
    ZP_X_Idxd,

    // OPC $LL,Y    operand is zeropage address; effective address is address incremented by Y without carry
    ZP_Y_Idxd,

    // The instruction is just one byte. Addressing is implicit
    Impl_Addr,

    // my cause page crossing or not
    Rel_Addrs,

    /// for `brk` and future expansions
    None
}


impl Addressing for Six502 {
    fn dispatch_load(&mut self, mode: AddressingMode) -> u8 {
        use AddressingMode::*;
        match mode {
            Acc_Addrs => {
                self.atom(|c| {
                    // comeback
                });
                self.a
            }
            Abs_Addrs => {
                let (mut p1, mut p2, mut v) = (0, 0, 0);
                self.atom(|c| p1 = c.load_u8_bump_pc());
                self.atom(|c| p2 = c.load_u8_bump_pc());
                self.atom(|c| {
                    let addr = u16::from_le_bytes([p1, p2]);
                    c.addr_bus = addr;
                    v = c.load_u8();
                });

                // let addr = self.load_u16_bump_pc();
                v
            }
            AbsX_Idxd => {
                let (mut p1, mut p2, mut v) = (0, 0, 0);
                let mut over = false;
                self.atom(|c| p1 = c.load_u8_bump_pc());
                let x = self.x;
                self.atom(|c| {
                    p2 = c.load_u8_bump_pc();
                    (p1, over) = p1.overflowing_add(x);
                });

                if over {
                    self.atom(|c| {
                        p1 += 1;
                        c.addr_bus = u16::from_le_bytes([p1, p2]);
                        v = c.load_u8();
                    });
                } else {
                    self.addr_bus = u16::from_le_bytes([p1, p2]);
                    v = self.load_u8();
                };

                // let op = self.load_u16_bump_pc();

                // // check if it'll overflow into the zero page
                // let lb_op = op as u8;
                // let (_, carry) = lb_op.overflowing_add(self.x);

                v
            }
            AbsY_Idxd => {
                let op = self.load_u16_bump_pc();

                // check if it'll overflow into the zero page
                let lb_op = op as u8;
                let (_, carry) = lb_op.overflowing_add(self.y);
                self.addr_bus = op + (self.y as u16);
                self.load_u8()
            }

            Immediate => {
                let mut v = 0;
                self.atom(|c| v = c.load_u8_bump_pc());
                v
            }
            Zero_Page => {
                let (mut addr, mut v) = (0, 0);
                self.atom(|c| {
                    addr = c.load_u8_bump_pc();
                });
                self.atom(|c| {
                    c.addr_bus = addr as u16;
                    v = c.load_u8();
                });
                v
            }
            //without carry
            ZP_X_Idxd => {
                let addr = self.load_u8_bump_pc();
                self.addr_bus = (addr.wrapping_add(self.x)) as u16;
                self.load_u8()
            } //without carry. that's why we add the `u8`s before converting to `u16`, so it won't carry into the high-byte

            //   If the base address plus X or Y exceeds the value that
            //   can be stored in a single byte, no carry is generated, therefore there is no page crossing phenomena
            //    A wrap-around will occur within Page Zero
            ZP_Y_Idxd => {
                let addr = self.load_u8_bump_pc();
                self.addr_bus = (addr.wrapping_add(self.y)) as u16;
                self.load_u8()
            }
            // The major use of indexed indirect is in picking up data from a table or list of addresses to perform an operation.
            X_Idx_Ind => {
                let v = self.load_u8_bump_pc();
                // zero page addition. Never crosses page. wraps around
                let comp = self.x.wrapping_add(v);
                self.addr_bus = comp as u16;
                let lo_addr = self.load_u8();
                self.addr_bus = (comp + 1) as u16;
                let hi_addr = self.load_u8();
                // say comp is 0x05 effective address becomes 0x0605
                let eff_addr = u16::from_le_bytes([lo_addr, hi_addr]);
                self.addr_bus = eff_addr;
                self.load_u8() // never crosses pae as the indexing is done in the zero page
            }
            Ind_Y_Idx => {
                let y = self.y;
                let v = self.load_u8_bump_pc();
                self.addr_bus = v as u16;
                let lo_addr = self.load_u8();
                self.addr_bus = (v + 1) as u16;
                let hi_addr = self.load_u8();
                // say v is 0x05 effective address becomes 0x0605
                let eff_addr = u16::from_le_bytes([lo_addr, hi_addr]);
                let (_, carry) = lo_addr.overflowing_add(y);
                self.addr_bus = eff_addr.wrapping_add(y as u16);
                self.load_u8() // might cross page
            }
            Impl_Addr => {
                // basically, nothing happens here, except tha the opcode fetched in last cycle is decoded.
                // so we just tick. the new opcode is decoded, and the pc os not incremented
                self.tick();
                // in the next cycle, the old opcode is executed and the opcode ignored in the above is decoded
                0
            }
            Rel_Addrs => {
                let (mut off) = (0);
                self.atom(|c| {
                    off = c.load_u8_bump_pc() as i8 as u16;
                });
                let mut overflowed = false;
                // comeback to deal with page transiions
                self.atom(|c| {
                    (c.pc, overflowed) = c.pc.overflowing_add(off);
                });
                if overflowed {
                    self.tick();
                }
                0
            }
            Ind_Addrs => todo!(),
            None => todo!(),
        }
    }

    fn dispatch_store(&mut self, v: u8, mode: AddressingMode) {
        use AddressingMode::*;

        match mode {
            Acc_Addrs => {
                self.a = v;
            }
            Abs_Addrs => {
                self.addr_bus = self.load_u16_bump_pc();
                self.store_u8(v);
            }

            AbsX_Idxd => {
                let op = self.load_u16_bump_pc();

                // check if it'll overflow into the zero page
                let lb_op = op as u8;
                let (_, carry) = lb_op.overflowing_add(self.x);
                let addr = self.load_u16_bump_pc();
                self.addr_bus = addr + (self.x as u16);
                self.store_u8(v);
            }
            AbsY_Idxd => {
                let op = self.load_u16_bump_pc();
                // check if it'll overflow into the zero page
                let lb_op = op as u8; // truncates
                let (_, carry) = lb_op.overflowing_add(self.y);
                let addr = self.load_u16_bump_pc();
                self.addr_bus  =addr + (self.y as u16);
                self.store_u8(v);
            }

            Immediate => (), // do nothing
            // Indirect => false,
            Zero_Page => {
                let addr = self.load_u8_bump_pc();
                self.addr_bus = addr as u16;
                self.store_u8(v);
            }

            //   If the base address plus X or Y exceeds the value that
            //   can be stored in a single byte, no carry is generated, therefore there is no page crossing phenomena
            //    A wrap-around will occur within Page Zero
            ZP_X_Idxd => {
                let addr = self.load_u8_bump_pc();
                self.addr_bus  =  (addr.wrapping_add(self.x)) as u16;
                self.store_u8(v);
            }
            ZP_Y_Idxd => {
                let addr = self.load_u8_bump_pc();
                self.addr_bus = (addr.wrapping_add(self.y)) as u16;
                self.store_u8(v);
            }

            X_Idx_Ind => {
                let v = self.load_u8_bump_pc();
                // zero page addition. Never crosses page. wraps around
                let comp = self.x.wrapping_add(v);
                self.addr_bus = comp as u16;
                let lo_addr = self.load_u8();
                self.addr_bus = comp.wrapping_add(1) as u16;
                let hi_addr = self.load_u8();
                let eff_addr = u16::from_le_bytes([lo_addr, hi_addr]);
                self.addr_bus  = eff_addr;
                self.store_u8(v);
                // never crosses page as the indexing is done in the zero page
            }
            Ind_Y_Idx => {
                let v = self.load_u8_bump_pc();
                let y = self.y;
                self.addr_bus = v as u16;
                let lo_addr = self.load_u8();
                self.addr_bus = (v + 1) as u16;
                let hi_addr = self.load_u8();
                // say v is 0x05 effective address becomes 0x0605
                let eff_addr = u16::from_le_bytes([lo_addr, hi_addr]);
                let (_, carry) = lo_addr.overflowing_add(y);
                self.addr_bus = eff_addr.wrapping_add(y as u16);
                self.store_u8(v);
                // might cross page
            }
            Impl_Addr => todo!(),
            Rel_Addrs => todo!(),
            Ind_Addrs => todo!(),
            None => todo!(),
        }
    }
}

pub(crate) mod table {
    pub(crate) use super::AddressingMode::{
        self,
        Acc_Addrs as Acc,
        Abs_Addrs as Abs,
        AbsX_Idxd as Abx,
        AbsY_Idxd as Aby,
        Impl_Addr as Imm,
        Ind_Addrs as Ind,
        X_Idx_Ind as Xin,
        Ind_Y_Idx as Yin,
        Zero_Page as Zep,
        ZP_X_Idxd as Zpx,
        ZP_Y_Idxd as Zpy,
        Impl_Addr as Imp,
        Rel_Addrs as Rel,
        None as Non,
    };

    /// We use lookup tables because lookup-tables are more efficient than large match statements
    /// The machine code generated only has to be the one `rust` will generate for array lookup and bounds checking 
    pub(crate) const AddrTable: [AddressingMode; 256] = [
        //    0,   1,   2,   3,   4,   5,   6,   7,   8,   9,   A,   B,   C,   D,   E,   F  // lo bit
        /*0*/ Imp, Xin, Non, Non, Non, Zep, Zep, Non, Imp, Imm, Acc, Non, Non, Abs, Abs, Non, 
        /*1*/ Rel, Yin, Non, Non, Non, Zpx, Zpx, Non, Imp, Aby, Non, Non, Non, Abx, Abx, Non,
        /*2*/ Abs, Xin, Non, Non, Zep, Zep, Zep, Non, Imp, Imm, Acc, Non, Abs, Abs, Abs, Non, 
        /*3*/ Rel, Yin, Non, Non, Non, Zpx, Zpx, Non, Imp, Aby, Non, Non, Non, Abx, Abx, Non, 
        /*4*/ Imp, Xin, Non, Non, Non, Zep, Zep, Non, Imp, Imm, Acc, Non, Abs, Abs, Abs, Non,
        /*5*/ Rel, Yin, Non, Non, Non, Zpx, Zpx, Non, Imp, Aby, Non, Non, Non, Abx, Abx, Non, 
        /*6*/ Imp, Xin, Non, Non, Non, Zep, Zep, Non, Imp, Imm, Acc, Non, Ind, Abs, Abs, Non, 
        /*7*/ Rel, Yin, Non, Non, Non, Zpx, Zpx, Non, Imp, Aby, Non, Non, Non, Abx, Abx, Non,
        /*8*/ Non, Xin, Non, Non, Zep, Zep, Zep, Non, Imp, Non, Imp, Non, Abs, Abs, Abs, Non, 
        /*9*/ Rel, Yin, Non, Non, Zpx, Zpx, Zpy, Non, Imp, Aby, Imp, Non, Non, Abx, Non, Non, 
        /*A*/ Imm, Xin, Imm, Non, Zep, Zep, Zep, Non, Imp, Imm, Imp, Non, Abs, Abs, Abs, Non,
        /*B*/ Rel, Yin, Non, Non, Zpx, Zpx, Zpy, Non, Imp, Aby, Imp, Non, Abx, Abx, Aby, Non, 
        /*C*/ Imm, Xin, Non, Non, Zep, Zep, Zep, Non, Imp, Imm, Imp, Non, Abs, Abs, Abs, Non, 
        /*D*/ Rel, Yin, Non, Non, Non, Zpx, Zpx, Non, Imp, Aby, Non, Non, Non, Abx, Abx, Non,
        /*E*/ Imm, Xin, Non, Non, Zep, Zep, Zep, Non, Imp, Imm, Imp, Non, Abs, Abs, Abs, Non, 
        /*F*/ Rel, Yin, Non, Non, Non, Zpx, Zpx, Non, Imp, Aby, Non, Non, Non, Abx, Abx, Non,
        // hi bit
    ];
}
