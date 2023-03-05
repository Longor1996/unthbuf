//! Formatting for [`UnthBuf`]
use super::{UnthBuf, CellLayout};

impl<CL: CellLayout> core::fmt::Debug for UnthBuf<CL> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // let max: Box<dyn std::fmt::Debug> = match ALIGNED {
        //     true => Box::new(self.aligned_location_of(self.capacity-1)),
        //     false => Box::new(self.unaligned_location_of(self.capacity-1))
        // };
        
        f.debug_struct("UnthBuf")
            .field("layout", &std::any::type_name::<CL>())
            .field("capacity", &self.capacity)
            .field("data_len", &self.data.len())
            .field("bits", &self.bits)
            .field("mask", &self.mask)
            //.field("data", &self.data)
            .finish()
    }
}

impl<CL: CellLayout + 'static> core::fmt::Display for UnthBuf<CL> {
    /// Prints the [`UnthBuf`] as a list of numbers, with the bit-size and capacity at the start...
    /// 
    /// ...thus taking the form: `[uBITS; CAPACITY; ELEMENT, ... ELEMENT]`
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[")?;
        write!(f, "u{}", self.bits)?;
        write!(f, "; ")?;
        write!(f, "{}", self.capacity)?;
        write!(f, "; ")?;
        let mut comma = false;
        for element in self {
            if comma {
                write!(f, ", {element}")?;
            } else {
                write!(f, "{element}")?;
                comma = true;
            }
        }
        write!(f, "]")
    }
}
