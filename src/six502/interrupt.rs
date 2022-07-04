/// https://www.nesdev.org/wiki/CPU_interrupts
/// Tthe concept of interrupt is used to signal the microprocessor that an external event has occurred and the
/// microprocessor should devote attention to it immediately.  
/// This technique accomplishes processing in which the microprocessor's program is interrupted and the event that caused the interrupt is serviced.
#[derive(Debug)]
pub(crate) struct Interrupt {
    schedule: Option<u8>,
}

/// When an interrupt occurs, the microprocessor uses the stack to save the reentrant or recovery code and then uses the interrupt vectors
/// FFFE and FFFF, (or FFFA and FFFB), depending on whether or not an interrupt request or a non maskable interrupt request had occurred.  
///It should he noted that the interrupt disable is turned on at this point by the microprocessor automatically.
impl Interrupt {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for Interrupt {
    fn default() -> Self {
        Self { schedule: None }
    }
}
