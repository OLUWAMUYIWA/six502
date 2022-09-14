use super::addressing::AddressingMode::*;
use super::{Op, CYCLES};
use crate::bus::{ByteAccess, DataBus, WordAccess};
use crate::{AddressingMode, Cpu};

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


    pub(crate) addr_bus: u16,
    pub(crate) data: u8,
}


impl ByteAccess for Six502 {
    fn load_u8(&mut self, addr: u16) -> u8 {
        self.bus.load_u8(addr)
    }

    fn store_u8(&mut self, addr: u16, v: u8) {
        self.bus.store_u8(addr, v);
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

    fn load_u8_bump_pc(&mut self) -> u8 {
        let addr = self.pc;
        self.pc = self.pc.wrapping_add(1);
        self.data = self.load_u8(addr);
        self.tick();
        self.data
    }
    
    fn load_u16_bump_pc(&mut self) -> u16 {
        let mut addr = self.pc;
        self.pc = self.pc.wrapping_add(1);
        self.data = self.load_u8(addr);
        self.tick();
        let lo = self.data;

        addr = self.pc;
        self.data = self.load_u8(addr);
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
    
    // fetch uses the address bus to fetch an opcode. It gets the u8 and bumps the pc
    fn fetch_op(&mut self, op: &mut Op) {
        let op_num = self.load_u8_bump_pc();
        op.curr_op_num = op_num;
    }

    /// The first byte of an instruction is called the OP CODE and is coded to contain the basic operation such as LDA
    /// then it has the data necessary to allow the microprocessor to interpret the address of the data on which the operation will occur
    /// the pc will increment after picking up the opcode (executing). it will then pick up the address of data the opcode is to act on
    /// and incrementing again after. for a full operation, it may incr 1,2,3 or more times
    /// an instance is LDA absolute addressing. three increments. one for opcode. one for low addr byte. one for high addr byte
    fn exec(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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
            0xaa => self.tax(Implied),
            0xa8 => self.tay(Implied),
            0x8a => self.txa(Implied),
            0x98 => self.tya(Implied),
            0x9a => self.txs(Implied),
            0xba => self.tsx(Implied),

            // stack ops
            0x08 => self.php(Implied), //implied addressing
            0x28 => self.plp(Implied), //implied addressing
            0x48 => self.pha(Implied), //implied addressing
            0x68 => self.pla(Implied), //implied addressing

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

            0xe8 => self.inx(Implied),
            0xca => self.dex(Implied),
            0xc8 => self.iny(Implied),
            0x88 => self.dey(Implied),

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
            0x4c => self.jmp(Implied),          // absolute
            0x6c => self.jmp_indirect(Implied), // indirect

            0x20 => self.jsr(Implied), // absolute
            0x60 => self.rts(Implied), // implied. In an implied instruction, the data and/or destination is mandatory for the instruction
            0x00 => self.brk(Implied), // implied
            0x40 => self.rti(Implied), // implied

            // branches
            0x10 => self.bpl(Implied), // relative The byte after the opcode is the branch offset.
            0x30 => self.bmi(Implied), // relative
            0x50 => self.bvc(Implied), // relative
            0x70 => self.bvs(Implied), // relative
            0x90 => self.bcc(Implied), // relative
            0xb0 => self.bcs(Implied), // relative
            0xd0 => self.bne(Implied), // relative
            0xf0 => self.beq(Implied), // relative

            // status flag changes
            0x18 => self.clc(Implied), // implied. In an implied instruction, the data and/or destination is mandatory for the instruction
            0x38 => self.sec(Implied), // implied
            0x58 => self.cli(Implied), // implied
            0x78 => self.sei(Implied), // implied
            0xb8 => self.clv(Implied), // implied
            0xd8 => self.cld(Implied), // implied
            0xf8 => self.sed(Implied), // implied

            // no-op
            0xea => self.nop(Implied),

            _ => unimplemented!("op not unimplemented: {}", op),
        };
        self.cy = self
            .cy
            .wrapping_add(CYCLES[op as usize] as u64)
            .wrapping_add(page_cross as u64);

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
        self.pc = self.load_u16(vectors::RESET);
        self.p = 0b00110100;

        // just to be sure
        self.a = 0x00;
        self.x = 0x00;
        self.y = 0x00;

        // comeback. number of cycles should be 8, byt should include
    }
}

pub trait Addressing {
    fn dispatch_load(&mut self, mode: AddressingMode) -> u8;
    fn dispatch_store(&mut self, v: u8, mode: AddressingMode);
}
