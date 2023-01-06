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
