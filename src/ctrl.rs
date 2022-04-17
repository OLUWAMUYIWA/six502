//check nesdev: https://www.nesdev.org/wiki/Controller_reading_code
// bit 	    7     	    6     	    5     	    4     	    3     	    2     	    1     	    0
// button 	A 			B 		 Select	      Start 		Up 		   Down 	   Left	    	Right
bitflags::bitflags! {
    pub(crate) struct Btns: u8 {
        const RIGHT             = 0b10000000;
        const LEFT              = 0b01000000;
        const DOWN              = 0b00100000;
        const UP                = 0b00010000;
        const START             = 0b00001000;
        const SELECT            = 0b00000100;
        const BUTTON_B          = 0b00000010;
        const BUTTON_A          = 0b00000001;
    }
}

pub(crate) struct Joypad {}
