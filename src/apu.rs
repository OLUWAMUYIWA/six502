// APU Registers
// [source](https://8bitworkshop.com/blog/platforms/nes/)
// ---------------------------------------------------------------------------------------------|
// | Address |    Name          |   Bits      |  Description                                     |
// | $4000   | SQ1_VOL          | ddlcvvvv    | Square wave 1, duty and volume                  |
// | $4001   | SQ1_SWEEP        | epppnsss    | Square wave 1, sweep                            |
// | $4002   | SQ1_LO           | pppppppp    | Square wave 1, period (LSB)                     |
// | $4003   | SQ1_HI           | xxxxxppp    | Square wave 1, period (MSB) and counter load    |
// | $4004   | SQ2_VOL          | dd..vvvv    | Square wave 2, duty and volume                  |
// | $4005   | SQ2_SWEEP        | epppnsss    | Square wave 2, sweep                            |
// | $4006   | SQ2_LO           | pppppppp    | Square wave 2, period (LSB)                     |
// | $4007   | SQ2_HI           | xxxxxppp    | Square wave 2, period (MSB) and counter load    |
// | $4008   | TRI_LINEAR       | crrrrrrr    | Triangle wave, control and counter load         |
// | $400A   | TRI_LO           | pppppppp    | Triangle wave, period (LSB)                     |
// | $400B   | TRI_HI           | xxxxxppp    | Triangle wave, period (MSB) and counter load    |
// | $400C   | NOISE_VOL        | ..lcvvvv    | Noise generator, flags and volume               |
// | $400E   | NOISE_CTRL       | t...pppp    | Noise generator, tone and period                |
// | $400F   | NOISE_LEN        | lllll...    | Noise generator, counter load                   |
// | $4010   | DMC_FREQ         | il..rrrr    | DMC: IRQ, flags, and rate                       |
// | $4011   | DMC_RAW          | .xxxxxxx    | DMC: direct load                                |
// | $4012   | DMC_START        | aaaaaaaa    | DMC, waveform start address                     |
// | $4013   | DMC_LEN          | llllllll    | DMC, waveform length                            |
// | $4015   | SND_CHN          | ...dnt21    | Sound channel enable                            |
// | $4017   | JOY2             | mi......    | Frame counter mode and IRQ                      |
// | $4015   | SND_CHN          | if.dnt21    | DMC/frame interrupt and status (read)           |
// | $4016   | JOY1             | ...xxxxd    | Joystick 1 (read)                               |
// | $4017   | JOY2             | ...xxxxd    | Joystick 2 (read)                               |
// ---------------------------------------------------------------------------------------------|

#[derive(Debug)]
pub(crate) struct Apu {}

impl Default for Apu {
    fn default() -> Self {
        todo!()
    }
}
impl Apu {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn load_u8(&self, addr: u16) -> u8 {
        todo!()
    }

    pub(crate) fn store_u8(&self, addr: u16) {
        todo!()
    }
}
