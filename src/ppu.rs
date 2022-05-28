use crate::bus::{ByteAccess, WordAccess};
use std::ops::{Deref, DerefMut};

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

struct PpuCtrl {
    v: u8,
}

struct PpuMask {
    v: u8,
}

struct PpuStatus {
    v: u8,
}

struct OamAddr {
    v: u8,
}

struct OamData {
    v: u8,
}

struct PpuScroll {
    v: u8,
}

struct PpuAddr {
    v: u8,
}

struct PpuData {
    v: u8,
}

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

pub(crate) struct Ppu {}

impl Ppu {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn load_u8(&self, addr: u16) -> u8 {
        todo!()
    }

    pub(crate) fn store_u8(&mut self, addr: u16, v: u8) {
        todo!()
    }
}

impl ByteAccess for Ppu {
    fn load_u8(&self, addr: u16) -> u8 {
        todo!()
    }

    fn store_u8(&mut self, addr: u16, v: u8) {
        todo!()
    }
}
