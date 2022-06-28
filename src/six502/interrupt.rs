/// https://www.nesdev.org/wiki/CPU_interrupts
#[derive(Debug)]
pub(crate) struct Interrupt {
    schedule: Option<u8>,
}

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
