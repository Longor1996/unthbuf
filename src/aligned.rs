//! Layout that stores integers in groups *within* word boundaries.
use crate::{UnthBuf, CellLayout, Bits, BITS_PER_CELL};

/// Layout that stores integers in groups *within* word boundaries.
#[derive(Clone, Copy)]
pub struct AlignedLayout;

impl CellLayout for AlignedLayout {
    type Location = AlignedLocation;
    
    #[inline(always)]
    fn get_cell_count(capacity: usize, bits: Bits) -> usize {
        let elements_per_cell = get_aligned_elements_per_cell(bits.get());
        (capacity + elements_per_cell as usize) / elements_per_cell as usize
    }
    
    fn get_exact_bit_count(buf: &UnthBuf<Self>) -> usize {
        let elements_per_cell = get_aligned_elements_per_cell(buf.bits.get());
        buf.data.len()
        * elements_per_cell as usize
        * buf.bits.get() as usize
    }
    
    #[inline(always)]
    fn location_of(buf: &UnthBuf<Self>, index: usize) -> Self::Location {
        //if !self.is_index(index) {eprintln!("index {index} is out of bounds; {:?}", self)}
        let cell = index / (buf.elpc as usize);
        //if !self.is_cell(cell) {eprintln!("index-{index} / cell-{cell} (E={elements_per_cell}) is outside the bounds of {:?}", self)}
        let offset = (index % buf.elpc as usize) as u8 * buf.bits.get();
        let mask = buf.mask << offset;
        AlignedLocation {
            cell, offset, mask
        }
    }

    #[inline(always)]
    unsafe fn set_unchecked(buf: &mut UnthBuf<Self>, index: usize, value: usize) {
        let loc = Self::location_of(buf, index);
        
        let mut cell = *buf.data.get_unchecked(loc.cell);
        
        cell &= !loc.mask; // unset the bits of the old value
        cell |= value << loc.offset; // set bits for new value
        
        *buf.data.get_unchecked_mut(loc.cell) = cell;
    }

    #[inline(always)]
    unsafe fn get_unchecked(buf: &UnthBuf<Self>, index: usize) -> usize {
        let loc = Self::location_of(buf, index);
        
        //if !self.is_cell(loc.cell) {panic!("aligned cell @{index} -> {:?} out of bounds; {:?}", loc, self)}
        
        (buf.data.get_unchecked(loc.cell) & loc.mask) >> loc.offset
    }
}

/// An usize-aligned location of an element within an [`UnthBuf`].
#[derive(Clone, Copy)]
pub struct AlignedLocation {
    pub(crate) cell: usize,
    pub(crate) offset: u8,
    pub(crate) mask: usize,
}

impl core::fmt::Debug for AlignedLocation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[#{} <<{} &{:b}]", self.cell, self.offset, self.mask)
    }
}

#[inline(always)]
pub(crate) fn get_aligned_elements_per_cell(bits: u8) -> u8 {
    BITS_PER_CELL / bits
}

// #[inline(always)]
// pub(crate) fn get_aligned_cellindex(index: usize, elements_per_cell: u8) -> usize {
//     index / (elements_per_cell as usize)
// }

// #[inline(always)]
// pub(crate) fn get_aligned_element_offset(index: usize, elements_per_cell: u8, bits: u8) -> u8 {
//     (index % elements_per_cell as usize) as u8 * bits
// }

// #[inline(always)]
// pub(crate) fn get_aligned_element_mask(mask: usize, offset: u8) -> usize {
//     mask << offset
// }
