//! Formatting for [`UnthBuf`]
use super::*;

impl<const ALIGNED: bool> std::fmt::Debug for UnthBuf<ALIGNED> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let max: Box<dyn std::fmt::Debug> = match ALIGNED {
        //     true => Box::new(self.aligned_location_of(self.capacity-1)),
        //     false => Box::new(self.unaligned_location_of(self.capacity-1))
        // };
        f.debug_struct("UnthBuf")
            .field("aligned", &ALIGNED)
            .field("capacity", &self.capacity)
            .field("data_len", &self.data.len())
            .field("bits", &self.bits)
            .field("mask", &self.mask)
            //.field("last", &max)
            .finish()
    }
}

impl<const ALIGNED: bool> std::fmt::Display for UnthBuf<ALIGNED> {
    /// Prints the [`UnthBuf`] as a list of numbers, with the bit-size and capacity at the start...
    /// 
    /// ...thus taking the form: `[uBITS; CAPACITY; ELEMENT, ... ELEMENT]`
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        write!(f, "u{}", self.bits)?;
        write!(f, "; ")?;
        write!(f, "{}", self.capacity)?;
        write!(f, "; ")?;
        let mut comma = false;
        for element in self.iter() {
            if comma {
                write!(f, ", {}", element)?;
            } else {
                write!(f, "{}", element)?;
                comma = true;
            }
        }
        write!(f, "]")
    }
}
