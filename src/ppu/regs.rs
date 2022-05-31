use crate::bus::{ByteAccess, WordAccess};
use std::ops::{Deref, DerefMut};

// The PPU exposes eight memory-mapped registers to the CPU. These nominally sit at $2000 through $2007 in the CPU's address space, but because they're incompletely decoded,
// they're mirrored in every 8 bytes from $2008 through $3FFF, so a write to $3456 is the same as a write to $2006.
// ----------------------------------------------------------------------------------------------|
// | Address | Name       |  R/W           | Description                                         |
// ----------------------------------------------------------------------------------------------|
// | $2000   | PPU_CTRL   | write          | PPU Control 1                                       |
// | $2001   | PPU_MASK   | write          | PPU Control 2                                       |
// | $2002   | PPU_STATUS | read           | PPU Status                                          |
// | $2003   | OAM_ADDR   | write          | OAM Address                                         |
// | $2004   | OAM_DATA   | read/write     | OAM Data                                            |
// | $2005   | PPU_SCROLL | write 2x       | Background Scroll Position \newline (write X then Y)|
// | $2006   | PPU_ADDR   | write 2x       | PPU Address \newline (write upper then lower)       |
// | $2007   | PPU_DATA   | read/write     | PPU Data                                            |
// | $4014   | OAM_DMA    | write          | Sprite Page DMA Transfer                            |
// -----------------------------------------------------------------------------------------------

/// [Details](https://www.nesdev.org/wiki/PPU_registers)
/// PPU Registers
pub(crate) struct Registers {

}
/// ```
/// 0x2000
/// 7654 3210
/// VPHB SINN
/// ```
/// NMI enable (V), PPU master/slave (P), sprite height (H),
/// background tile select (B), sprite tile select (S), increment mode (I), nametable select (NN)
/// Access: write
struct PpuCtrl {
    v: u8,
}

/// ```
/// 0x2001
/// 7654 3210
/// BGRs bMmG
/// ```
/// color emphasis (BGR), sprite enable (s), background enable (b),
/// sprite left column enable (M), background left column enable (m), greyscale (G)
struct PpuMask {
    v: u8,
}

/// ```
/// 0x2002
/// 7654 3210
/// VSO- ----
/// ```
/// vblank (V), sprite 0 hit (S), sprite overflow (O); read resets write pair for $2005/$2006
struct PpuStatus {
    v: u8,
}

/// ```
/// 0x2003
/// 7654 3210
/// aaaa aaaa
/// ```
/// OAM read/write address
struct OamAddr {
    v: u8,
}

/// ```
/// 0x2004
/// 7654 3210
/// dddd dddd
/// ```
/// OAM data read/write
struct OamData {
    v: u8,
}

/// ```
/// 0x2005
/// 7654 3210
/// xxxx xxxx
/// ```
/// fine scroll position (two writes: X scroll, Y scroll)
struct PpuScroll {
    v: u8,
}

/// ```
/// 0x2006
/// 7654 3210
/// aaaa aaaa
/// ```
/// PPU read/write address (two writes: most significant byte, least significant byte)
struct PpuAddr {
    v: u8,
}
/// ```
/// 0x2007
/// 7654 3210
/// dddd dddd
/// ```
/// PPU data read/write 
struct PpuData {
    v: u8,
}
/// ```
/// 0x4014
/// 7654 3210
/// aaaa aaaa
/// ```
/// OAM DMA high address
struct OamDma {
    v: u8,
}

crate::impl_deref_mut!(
    PpuCtrl { v },
    PpuStatus { v },
    OamAddr { v },
    OamData { v },
    PpuScroll { v },
    PpuAddr { v },
    PpuData { v },
    OamDma { v }
);

