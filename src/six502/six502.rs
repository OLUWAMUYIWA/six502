use super::addressing::AddressingMode::*;
use super::{Op, CYCLES};
use crate::bus::{DataBus, BusAccess};
use crate::ByteAccess;
use crate::{AddressingMode, Cpu};
use super::WordAccess;

use super::{disasm::INSTRUCTIONS, vectors};

pub struct Six502 {
    /// the major use for the accumulator is transferring data from memory to the accumulator or from the accumulator to memory.
    /// mathematical amd logical operations can then be done to data inside the accumulator. It is where intermediate values are normally  stored
    pub(super) a: u8,
    pub(super) x: u8,
    pub(super) y: u8,
    /// the program counter (program address pointer) is used to choose (address) the next memory location and the value which the memory
    /// sends hack is decoded in order to determine what operation the MCS650X is going to perform next
    /// it must always be addressing the operation the user wants to perform next
    /// The microprocessor puts the value of the program counter Onto the address bus, transferring
    /// the 8 bits of data at that memory address into the instruction decode. It then autoincreases by one
    /// to change the sequence of ops, the programmer can only do it by changing the value of the pc
    pub(super) pc: u16,
    pub(super) s: u8,
    pub(super) cy: u64,
    /// flags
    pub(super) p: u8, 
    /// Sixteen bits of address allow access to 65,536 memory locations, each of which, in the MCS650X family, consists of 8 bits of data
    pub(crate) bus: DataBus,
    pub(crate) data: u8,

    pub(crate) addr_bus: u16,
}


impl ByteAccess for Six502 {
    fn load_u8(&mut self) -> u8 {
        self.bus.load_u8(self.addr_bus)
    }

    fn store_u8(&mut self, v: u8) {
        self.bus.store_u8(self.addr_bus, v);
    }

    fn bump(&mut self) {
        self.addr_bus += 1;
    }

    
}

impl Default for Six502 {
    fn default() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0xc000,
            s: 0xfd,
            cy: 0,
            p: 0x24,
            bus: DataBus::new(),
            addr_bus: 0,
            data: 0,
        }
    }
}

impl Cpu for Six502 {
    fn new() -> Self {
        Default::default()
    }

    /// 
    /// does not tick. expects the caller to
    fn load_u8_bump_pc(&mut self) -> u8 {
        // first set the address bus
        self.addr_bus = self.pc;
        // bump pc
        self.pc = self.pc.wrapping_add(1);
        // place the pc on the address bus and load. the result is stored in cpu.data 
        self.data = self.load_u8();
        self.data
    }
    
    // comeback maybe we dont need this
    fn load_u16_bump_pc(&mut self) -> u16 {
        let mut addr = self.pc;
        self.pc = self.pc.wrapping_add(1);
        self.data = self.load_u8();
        self.tick();
        let lo = self.data;
        
        // bump the address in the addr bus
        self.bump();

        addr = self.pc;
        self.data = self.load_u8();
        self.pc = self.pc.wrapping_add(1);
        self.tick();
        let hi = self.data;

        u16::from_le_bytes([lo, hi])
    }

    // the internal state of the pc and io should be deterministic at the beginning.
    // The reset line is controlled during power on initialization and is a common line which is connected to all devices in the microcomputer
    // which have to be initialized to a known state.
    // In the MCS650X, power on or reset control operates at two levels.
    // First, by holding of an external line to ground, and having this external
    //  line connected to all the devices connected to the microprocessor during power up,
    // the entire microcomputer system is initialized to a known disabled state
    // Second, the release of the reset line from the ground or TTL zero
    // condition to a TTL one condition causes the microprocessor to be automatically
    // initialized, first by the internal hardware vector (RESET) which causes it
    // to be pointed to a known program location (PC), and secondly by what the programmer writes as the first set of instructions
    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // While the reset line is in the low state, it can be assumed that internal registers may be initialized to any random condition; therefore,
        // no conditions about the internal state of the microprocessor are assumed other than that the microprocessor will, one cycle after the reset line
        // goes high, implement the following sequence:
        self.reset();
        // comeback. the loaded program begins in the 8th cycle
        self.cy += 7;
        // the first operation in any normal program will be to initialize the stack
        // Once this is accomplished, the two non variable operations of the machine are under control.
        // The program counter is initialized and under
        // programmer control and the stack is initialized and under program control.
        Ok(())
    }
    
    /// `fetch_op` uses the address bus to fetch an opcode. It gets the u8 and bumps the pc
    /// `fetch_op` cannot be an atomic op because the internal operation of [Six502] during `fetch_op` varies
    fn fetch_op(&mut self) {
        //get the current op from the pc
        self.load_u8_bump_pc();
    }

    /// decodes the op fetched by setting the [Op]'s internal values, i.e. the `addr_mode`, `curr_up`, and `curr_op_num` 
    fn decode_op(&mut self, op: &mut Op) {
        use crate::six502::addressing::table::AddrTable;
        op.curr_op_num = self.data;
        op.addr_mode = AddrTable[op.curr_op_num as usize];
        op.curr_op = todo!();
    }


    /// The first byte of an instruction is called the OP CODE and is coded to contain the basic operation such as LDA
    /// then it has the data necessary to allow the microprocessor to interpret the address of the data on which the operation will occur
    /// the pc will increment after picking up the opcode (executing). it will then pick up the address of data the opcode is to act on
    /// and incrementing again after. for a full operation, it may incr 1,2,3 or more times
    /// an instance is LDA absolute addressing. three increments. one for opcode. one for low addr byte. one for high addr byte
    fn exec(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.load_u8_bump_pc();
        let op = self.data;

        match op {
            // load/stores
            0xa1 => self.lda(X_Idx_Ind),
            0xa5 => self.lda(Zero_Page),
            0xa9 => self.lda(Immediate),
            0xad => self.lda(Abs_Addrs),
            0xb1 => self.lda(Ind_Y_Idx),
            0xb5 => self.lda(ZP_X_Idxd),
            0xb9 => self.lda(AbsY_Idxd),
            0xbd => self.lda(AbsX_Idxd),

            0xa2 => self.ldx(Immediate),
            0xa6 => self.ldx(Zero_Page),
            0xae => self.ldx(Abs_Addrs),
            0xb6 => self.ldx(ZP_Y_Idxd),
            0xbe => self.ldx(AbsY_Idxd),

            0xa0 => self.ldy(Immediate),
            0xa4 => self.ldy(Zero_Page),
            0xac => self.ldy(Abs_Addrs),
            0xb4 => self.ldy(ZP_X_Idxd),
            0xbc => self.ldy(AbsX_Idxd),

            0x81 => self.sta(X_Idx_Ind),
            0x85 => self.sta(Zero_Page),
            0x8d => self.sta(Abs_Addrs),
            0x91 => self.sta(Ind_Y_Idx),
            0x95 => self.sta(ZP_X_Idxd),
            0x99 => self.sta(AbsY_Idxd),
            0x9d => self.sta(AbsX_Idxd),

            0x86 => self.stx(Zero_Page),
            0x8e => self.stx(Abs_Addrs),
            0x96 => self.stx(ZP_Y_Idxd),

            0x84 => self.sty(Zero_Page),
            0x8c => self.sty(Abs_Addrs),
            0x94 => self.sty(ZP_X_Idxd),

            // comparisons
            0xc1 => self.cmp(X_Idx_Ind),
            0xc5 => self.cmp(Zero_Page),
            0xc9 => self.cmp(Immediate),
            0xcd => self.cmp(Abs_Addrs),
            0xd1 => self.cmp(Ind_Y_Idx),
            0xd5 => self.cmp(ZP_X_Idxd),
            0xd9 => self.cmp(AbsY_Idxd),
            0xdd => self.cmp(AbsX_Idxd),

            0xe0 => self.cpx(Immediate),
            0xe4 => self.cpx(Zero_Page),
            0xec => self.cpx(Abs_Addrs),

            0xc0 => self.cpy(Immediate),
            0xc4 => self.cpy(Zero_Page),
            0xcc => self.cpy(Abs_Addrs),

            // transfers
            0xaa => self.tax(Impl_Addr),
            0xa8 => self.tay(Impl_Addr),
            0x8a => self.txa(Impl_Addr),
            0x98 => self.tya(Impl_Addr),
            0x9a => self.txs(Impl_Addr),
            0xba => self.tsx(Impl_Addr),

            // stack ops
            0x08 => self.php(Impl_Addr), //Implied addressing
            0x28 => self.plp(Impl_Addr), //Implied addressing
            0x48 => self.pha(Impl_Addr), //Implied addressing
            0x68 => self.pla(Impl_Addr), //Implied addressing

            // logical ops
            0x21 => self.and(X_Idx_Ind),
            0x25 => self.and(Zero_Page),
            0x29 => self.and(Immediate),
            0x2d => self.and(Abs_Addrs),
            0x35 => self.and(ZP_X_Idxd),
            0x31 => self.and(Ind_Y_Idx),
            0x39 => self.and(AbsY_Idxd),
            0x3d => self.and(AbsX_Idxd),

            0x01 => self.ora(X_Idx_Ind),
            0x05 => self.ora(Zero_Page),
            0x09 => self.ora(Immediate),
            0x0d => self.ora(Abs_Addrs),
            0x11 => self.ora(Ind_Y_Idx),
            0x15 => self.ora(ZP_X_Idxd),
            0x1d => self.ora(AbsX_Idxd),
            0x19 => self.ora(AbsY_Idxd),

            0x41 => self.eor(X_Idx_Ind),
            0x45 => self.eor(Zero_Page),
            0x49 => self.eor(Immediate),
            0x4d => self.eor(Abs_Addrs),
            0x51 => self.eor(Ind_Y_Idx),
            0x55 => self.eor(ZP_X_Idxd),
            0x5d => self.eor(AbsX_Idxd),
            0x59 => self.eor(AbsY_Idxd),

            // bit test
            0x24 => {
                self.bit(Zero_Page) //bit test
            }
            0x2c => {
                self.bit(Abs_Addrs) // bit test
            }

            // arithmetic ops
            0x61 => self.adc(X_Idx_Ind),
            0x65 => self.adc(Zero_Page),
            0x69 => self.adc(Immediate),
            0x6d => self.adc(Abs_Addrs),
            0x71 => self.adc(Ind_Y_Idx),
            0x75 => self.adc(ZP_X_Idxd),
            0x79 => self.adc(AbsY_Idxd),
            0x7d => self.adc(AbsX_Idxd),

            0xe1 => self.sbc(X_Idx_Ind),
            0xe5 => self.sbc(Zero_Page),
            0xe9 => self.sbc(Immediate),
            0xed => self.sbc(Abs_Addrs),
            0xf1 => self.sbc(Ind_Y_Idx),
            0xf5 => self.sbc(ZP_X_Idxd),
            0xf9 => self.sbc(AbsY_Idxd),
            0xfd => self.sbc(AbsX_Idxd),

            //incrs and decrs
            0xe6 => self.inc(Zero_Page),
            0xee => self.inc(Abs_Addrs),
            0xf6 => self.inc(ZP_X_Idxd),
            0xfe => self.inc(AbsX_Idxd),

            0xc6 => self.dec(Zero_Page),
            0xce => self.dec(Abs_Addrs),
            0xd6 => self.dec(ZP_X_Idxd),
            0xde => self.dec(AbsX_Idxd),

            0xe8 => self.inx(Impl_Addr),
            0xca => self.dex(Impl_Addr),
            0xc8 => self.iny(Impl_Addr),
            0x88 => self.dey(Impl_Addr),

            // shifts
            0x26 => self.rol(Zero_Page),
            0x2a => self.rol(Acc_Addrs),
            0x2e => self.rol(Abs_Addrs),
            0x36 => self.rol(ZP_X_Idxd),
            0x3e => self.rol(AbsX_Idxd),

            0x66 => self.ror(Zero_Page),
            0x6a => self.ror(Acc_Addrs),
            0x6e => self.ror(Abs_Addrs),
            0x76 => self.ror(ZP_X_Idxd),
            0x7e => self.ror(AbsX_Idxd),

            0x06 => self.asl(Zero_Page),
            0x0e => self.asl(Abs_Addrs),
            0x0a => self.asl(Acc_Addrs),
            0x16 => self.asl(ZP_X_Idxd),
            0x1e => self.asl(AbsX_Idxd),

            0x4a => self.lsr(Acc_Addrs),
            0x46 => self.lsr(Zero_Page),
            0x4e => self.lsr(Abs_Addrs),
            0x56 => self.lsr(ZP_X_Idxd),
            0x5e => self.lsr(AbsX_Idxd),

            // jumps and calls
            0x4c => self.jmp(Impl_Addr),          // absolute
            0x6c => self.jmp_indirect(Impl_Addr), // indirect

            0x20 => self.jsr(Impl_Addr), // absolute
            0x60 => self.rts(Impl_Addr), // Impl_Addr. In an Impl_Addr instruction, the data and/or destination is mandatory for the instruction
            0x00 => self.brk(Impl_Addr), // Impl_Addr
            0x40 => self.rti(Impl_Addr), // Impl_Addr

            // branches
            0x10 => self.bpl(Impl_Addr), // The byte after the opcode is the branch offset.
            0x30 => self.bmi(Impl_Addr), 
            0x50 => self.bvc(Impl_Addr), 
            0x70 => self.bvs(Impl_Addr), 
            0x90 => self.bcc(Impl_Addr), 
            0xb0 => self.bcs(Impl_Addr), 
            0xd0 => self.bne(Impl_Addr), 
            0xf0 => self.beq(Impl_Addr), 

            // status flag changes
            0x18 => self.clc(Impl_Addr), // Impl_Addr. In an Impl_Addr instruction, the data and/or destination is mandatory for the instruction
            0x38 => self.sec(Impl_Addr), // Impl_Addr
            0x58 => self.cli(Impl_Addr), // Impl_Addr
            0x78 => self.sei(Impl_Addr), // Impl_Addr
            0xb8 => self.clv(Impl_Addr), // Impl_Addr
            0xd8 => self.cld(Impl_Addr), // Impl_Addr
            0xf8 => self.sed(Impl_Addr), // Impl_Addr

            // no-op
            0xea => self.nop(Impl_Addr),

            _ => unimplemented!("op not unimplemented: {}", op),
        };
        self.cy = self
            .cy
            .wrapping_add(CYCLES[op as usize] as u64);

        Ok(())
    }

    /// sets the program counter to the value the RESET vector pointer holds
    /// Instructions exist for the initialization and loading of all other registers in the microprocessor except for the initial setting of the
    /// program counter.  It is for this initial setting of the program counter to a fixed location in the restart vector location specified by the micro-
    /// processor programmer that the reset line in the microprocessor is primarily used.
    fn reset(&mut self) {
        // There are two major facts to remember about initialization.  One, the only automatic operations of the microprocessor during reset are to turn
        // on the interrupt disable bit and to force the program counter to the vector location specified in locations
        // FFFC and FFFD and to load the first instruction from that location.
        // force the program counter to the vector location specified in locations FFFC and FFFD
        self.addr_bus = vectors::RESET;
        self.pc = self.load_u16();
        self.p = 0b00110100;

        // just to be sure
        self.a = 0x00;
        self.x = 0x00;
        self.y = 0x00;

        // comeback. number of cycles should be 8, byt should include
    }

    
}


