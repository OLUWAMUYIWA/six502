use std::io::{self, Read};

pub struct Common;

impl Common {
    pub fn read_full(b: &mut [u8], r: impl Read) -> Result<(), io::Error> {
        let len = b.len();
        let mut n = 0;
        while n < len {
            let count = r.read(&mut b[n..])?;
            match count {
                0 => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "should be up to count",
                    ))
                }
                _ => n += count,
            }
        }
        Ok(())
    }
}
