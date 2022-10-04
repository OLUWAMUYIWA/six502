
impl AddressingMode {
    /// load loads a byte from memory based on the addressing mode. It returns a tuple; the byte being fetched, and a boolean
    /// indicating if there is a page cross while loading the byte.
    pub(super) fn load(&self, cpu: &mut Six502) -> (u8, bool) {
        match self {
            AddressingMode::Acc_Addrs => {
                cpu.atom(|c| {
                    // comeback
                });
                (cpu.a, false)
            }
            AddressingMode::Abs_Addrs => {
                let (mut p1, mut p2, mut v) = (0, 0, 0);
                cpu.atom(|c| p1 = c.load_u8_bump_pc());
                cpu.atom(|c| p2 = c.load_u8_bump_pc());
                cpu.atom(|c| {
                    let addr = u16::from_le_bytes([p1, p2]);
                    v = c.load_with_addr_u8(addr);
                });

                // let addr = cpu.load_u16_bump_pc();
                (v, false)
            }
            AddressingMode::AbsX_Idxd => {
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
                        v = c.load_with_addr_u8(addr);
                    });
                } else {
                    let addr = u16::from_le_bytes([p1, p2]);
                    v = c.load_with_addr_u8(addr);
                };

                // let op = cpu.load_u16_bump_pc();

                // // check if it'll overflow into the zero page
                // let lb_op = op as u8;
                // let (_, carry) = lb_op.overflowing_add(cpu.x);

                (v, false)
            }
            AddressingMode::AbsY_Idxd => {
                let op = cpu.load_u16_bump_pc();

                // check if it'll overflow into the zero page
                let lb_op = op as u8;
                let (_, carry) = lb_op.overflowing_add(cpu.y);
                (cpu.load_with_addr_u8(op + (cpu.y as u16)), carry)
            }

            AddressingMode::Immediate => {
                let mut v = 0;
                cpu.atom(|c| v = c.load_u8_bump_pc());
                (v, false)
            }
            AddressingMode::Zero_Page => {
                let (mut addr, mut v) = (0, 0);
                cpu.atom(|c| {
                    addr = c.load_u8_bump_pc();
                });
                cpu.atom(|c| {
                    let addr = addr as u16;
                    v = c.load_with_addr_u8(addr);
                });
                // let addr = cpu.load_u8_bump_pc();
                (v, false)
            }
            //without carry
            AddressingMode::ZP_X_Idxd => {
                let addr = cpu.load_u8_bump_pc();
                (cpu.load_with_addr_u8((addr.wrapping_add(cpu.x)) as u16), false)
            } //without carry. that's why we add the `u8`s before converting to `u16`, so it won't carry into the high-byte

            //   If the base address plus X or Y exceeds the value that
            //   can be stored in a single byte, no carry is generated, therefore there is no page crossing phenomena
            //    A wrap-around will occur within Page Zero
            AddressingMode::ZP_Y_Idxd => {
                let addr = cpu.load_u8_bump_pc();
                (cpu.load_with_addr_u8((addr.wrapping_add(cpu.y)) as u16), false)
            }
            // The major use of indexed indirect is in picking up data from a table or list of addresses to perform an operation.
            AddressingMode::X_Idx_Ind => {
                let v = cpu.load_u8_bump_pc();
                // zero page addition. Never crosses page. wraps around
                let comp = cpu.x.wrapping_add(v);
                let lo_addr = cpu.load_with_addr_u8(comp as u16);
                let hi_addr = cpu.load_with_addr_u8((comp + 1) as u16);
                // say comp is 0x05 effective address becomes 0x0605
                let eff_addr = u16::from_le_bytes([lo_addr, hi_addr]);
                (cpu.load_with_addr_u8(eff_addr), false) // never crosses pae as the indexing is done in the zero page
            }
            AddressingMode::Ind_Y_Idx => {
                let y = cpu.y;
                let v = cpu.load_u8_bump_pc();
                let lo_addr = cpu.load_with_addr_u8(v as u16);
                let hi_addr = cpu.load_with_addr_u8((v + 1) as u16);
                // say v is 0x05 effective address becomes 0x0605
                let eff_addr = u16::from_le_bytes([lo_addr, hi_addr]);
                let (_, carry) = lo_addr.overflowing_add(y);
                (cpu.load_with_addr_u8(eff_addr.wrapping_add(y as u16)), carry) // might cross page
            }
            AddressingMode::Impl_Addr => {
                // basically, nothing happens here, except tha the opcode fetched in last cycle is decoded.
                // so we just tick. the new opcode is decoded, and the pc os not incremented
                cpu.tick();
                // in the next cycle, the old opcode is executed and the opcode ignored in the above is decoded
                (0, false)
            }
            AddressingMode::Rel_Addrs => {
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
            AddressingMode::Ind_Addrs => todo!(),
            AddressingMode::None => todo!(),
        }


    pub(super) fn store(&self, cpu: &mut Six502, v: u8) -> bool {
        match self {
            AddressingMode::Acc_Addrs => {
                cpu.a = v;
                false
            }
            AddressingMode::Abs_Addrs => {
                let addr = cpu.load_u16_bump_pc();
                cpu.addr_bus = addr;
                cpu.store_u8(addr);
                false
            }

            AddressingMode::AbsX_Idxd => {
                let op = cpu.load_u16_bump_pc();

                // check if it'll overflow into the zero page
                let lb_op = op as u8;
                let (_, carry) = lb_op.overflowing_add(cpu.x);
                let addr = cpu.load_u16_bump_pc();
                cpu.addr_bus = addr + (cpu.x as u16);
                cpu.store_u8(v);
                carry
            }
            AddressingMode::AbsY_Idxd => {
                let op = cpu.load_u16_bump_pc();
                // check if it'll overflow into the zero page
                let lb_op = op as u8; // truncates
                let (_, carry) = lb_op.overflowing_add(cpu.y);
                let addr = cpu.load_u16_bump_pc();
                cpu.addr_bus = addr + (cpu.y as u16);
                cpu.store_u8(v);
                carry
            }

            AddressingMode::Immediate => false, // do nothing
            // AddressingMode::Indirect => false,
            AddressingMode::Zero_Page => {
                let addr = cpu.load_u8_bump_pc();
                cpu.addr_bus = addr as u16;
                cpu.store_u8(v);
                false
            }

            //   If the base address plus X or Y exceeds the value that
            //   can be stored in a single byte, no carry is generated, therefore there is no page crossing phenomena
            //    A wrap-around will occur within Page Zero
            AddressingMode::ZP_X_Idxd => {
                let addr = cpu.load_u8_bump_pc();
                cpu.addr_bus = (addr.wrapping_add(cpu.x)) as u16;
                cpu.store_u8( v);
                false
            }
            AddressingMode::ZP_Y_Idxd => {
                let addr = cpu.load_u8_bump_pc();
                cpu.addr_bus = (addr.wrapping_add(cpu.y)) as u16;
                cpu.store_u8(v);
                false
            }

            AddressingMode::X_Idx_Ind => {
                let v = cpu.load_u8_bump_pc();
                // zero page addition. Never crosses page. wraps around
                let comp = cpu.x.wrapping_add(v);
                cpu.addr_bus = comp as u16;
                let lo_addr = cpu.load_u8();
                cpu.addr_bus = comp.wrapping_add(1) as u16;
                let hi_addr = cpu.load_u8();
                let eff_addr = u16::from_le_bytes([lo_addr, hi_addr]);
                cpu.addr_bus = eff_addr;
                cpu.store_u8(v);
                false // never crosses page as the indexing is done in the zero page
            }
            AddressingMode::Ind_Y_Idx => {
                let v = cpu.load_u8_bump_pc();
                let y = cpu.y;
                cpu.addr_bus = v as u16;
                let lo_addr = cpu.load_u8();
                cpu.addr_bus = (v + 1) as u16;
                let hi_addr = cpu.load_u8();
                // say v is 0x05 effective address becomes 0x0605
                let eff_addr = u16::from_le_bytes([lo_addr, hi_addr]);
                let (_, carry) = lo_addr.overflowing_add(y);
                cpu.addr_bus = eff_addr.wrapping_add(y as u16);
                cpu.store_u8(v);
                carry // might cross page
            }
            AddressingMode::Impl_Addr => todo!(),
            AddressingMode::Rel_Addrs => todo!(),
            AddressingMode::Ind_Addrs => todo!(),
            AddressingMode::None => todo!(),
        }
 }
    }


    fn load_with_addr_u8(&mut self, addr: u16) -> u8 {
        self.bus.load_u8(addr)
    }

    fn store_with_addr_u8(&mut self, addr: u16, v: u8) {
        self.bus.store_u8(addr, v);
    }