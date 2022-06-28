use std::ops::BitAnd;

//check nesdev: https://www.nesdev.org/wiki/Controller_reading_code
// JOYPAD1 = $4016
// JOYPAD2 = $4017

// bit         7           6           5           4           3           2           1           0
// button      A           B        Select       Start         Up         Down        Left       Right
bitflags::bitflags! {
        pub struct JoypadButton: u8 {
            const A          = 0b00000001;
            const B          = 0b00000010;
            const SELECT     = 0b00000100;
            const START      = 0b00001000;
            const UP         = 0b00010000;
            const DOWN       = 0b00100000;
            const LEFT       = 0b01000000;
            const RIGHT      = 0b10000000;
        }
}

#[derive(Debug)]
pub struct Joypad {
    strobe: bool,
    index: u8,
    curr: JoypadButton,
}

impl  Default for Joypad {
    fn default() -> Self {
        todo!()
    }
}
impl Joypad {
    pub fn new() -> Self {
        Joypad {
            strobe: false,
            index: 0,
            curr: JoypadButton::from_bits_truncate(0),
        }
    }

    pub fn store_u8(&mut self, data: u8) {
        self.strobe = data.bitand(1) == 1;
        if self.strobe {
            self.index = 0
        }
    }

    pub(crate) fn load_u8(&mut self) -> u8 {
        if self.index > 7 {
            return 1;
        }
        let response = (self.curr.bits & (1 << self.index)) >> self.index;
        if !self.strobe && self.index <= 7 {
            self.index += 1;
        }
        response
    }

    pub fn set_button_pressed_status(&mut self, button: JoypadButton, pressed: bool) {
        self.curr.set(button, pressed);
    }
}
