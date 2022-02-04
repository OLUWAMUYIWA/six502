use crate::apu::Apu;
use crate::ppu::Ppu;
use crate::rom::Rom;
use crate::six502::Six502;

pub(crate) struct Nes {
    cpu: Six502,
    rom: Rom,
    ppu: Ppu,
    apu: Apu,
}
