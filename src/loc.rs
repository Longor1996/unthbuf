//! Location helpers for [`UnthBuf`]
use super::*;

impl<const ALIGNED: bool> UnthBuf<ALIGNED> {
    /// Returns the *location* of the cell, the *bit-offset* within it and a *mask*,
    /// for the element at the given index.
    #[inline(always)]
    pub fn aligned_location_of(&self, index: usize) -> UnthBufAlignedLocation {
        //if !self.is_index(index) {eprintln!("index {index} is out of bounds; {:?}", self)}
        let cell = index / (self.elpc as usize);
        //if !self.is_cell(cell) {eprintln!("index-{index} / cell-{cell} (E={elements_per_cell}) is outside the bounds of {:?}", self)}
        let offset = (index % self.elpc as usize) as u8 * self.bits;
        let mask = self.mask << offset;
        UnthBufAlignedLocation {
            cell, offset, mask
        }
    }
    
    /// Returns the *location* of the starting cell, the *bit-offsets* within it and *two masks*,
    /// for the element at the given index.
    #[inline(always)]
    pub fn unaligned_location_of(&self, index: usize) -> UnthBufUnalignedLocation {
        //if !self.is_index(index) {eprintln!("index {index} is out of bounds; {:?}", self)}
        let bitindex = get_unaligned_bitindex(index, self.bits);
        let cell = get_unaligned_cellindex_low(bitindex);
        //if !self.is_cell(cell) {eprintln!("cell {cell} is out of bounds; {:?}", self)}
        let offset_low = get_unaligned_element_offset_low(bitindex);
        let offset_high = get_unaligned_element_offset_high(bitindex, self.bits);
        let mask0 = get_unaligned_element_mask_low(self.mask, offset_low);
        let mask1 = get_unaligned_element_mask_high(self.mask, offset_low, self.bits);
        
        UnthBufUnalignedLocation {
            cell,
            offset0: offset_low,
            offset1: offset_high,
            mask0,
            mask1
        }
    }
}

impl UnthBuf<true> {
    /// Returns the location of the element at the given `index`.
    pub fn location_of(&self, index: usize) -> UnthBufAlignedLocation {
        self.aligned_location_of(index)
    }
}

impl UnthBuf<false> {
    /// Returns the location of the element at the given `index`.
    pub fn location_of(&self, index: usize) -> UnthBufUnalignedLocation {
        self.unaligned_location_of(index)
    }
}

/// An usize-aligned location of an element within an [`UnthBuf`].
#[derive(Clone, Copy)]
pub struct UnthBufAlignedLocation {
    pub(crate) cell: usize,
    pub(crate) offset: u8,
    pub(crate) mask: usize,
}

impl std::fmt::Debug for UnthBufAlignedLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[#{} <<{} &{:b}]", self.cell, self.offset, self.mask)
    }
}

/// An unaligned location of an element within an [`UnthBuf`].
#[derive(Clone, Copy)]
pub struct UnthBufUnalignedLocation {
    pub(crate) cell: usize,
    pub(crate) offset0: u8,
    pub(crate) offset1: u8,
    pub(crate) mask0: usize,
    pub(crate) mask1: usize,
}

impl std::fmt::Debug for UnthBufUnalignedLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[#{} <<h{}l{} &h{:b}l{:b}]", self.cell, self.offset1, self.offset0, self.mask1, self.mask0)
    }
}

#[inline(always)]
pub(crate) fn get_aligned_elements_per_cell(bits: u8) -> u8 {
    UnthBuf::BITS_PER_CELL / bits
}

#[inline(always)]
pub(crate) fn get_aligned_cellindex(index: usize, elements_per_cell: u8) -> usize {
    index / (elements_per_cell as usize)
}

#[inline(always)]
pub(crate) fn get_aligned_element_offset(index: usize, elements_per_cell: u8, bits: u8) -> u8 {
    (index % elements_per_cell as usize) as u8 * bits
}

#[inline(always)]
pub(crate) fn get_aligned_element_mask(mask: usize, offset: u8) -> usize {
    mask << offset
}

#[inline(always)]
pub(crate) fn get_unaligned_bitindex(index: usize, bits: u8) -> usize {
    index * bits as usize
}

#[inline(always)]
pub(crate) fn get_unaligned_cellindex_low(bitindex: usize) -> usize {
    bitindex / UnthBuf::BITS_PER_CELL as usize
}

#[inline(always)]
#[allow(dead_code)]
pub(crate) fn get_unaligned_cellindex_high(bitindex: usize) -> usize {
    (bitindex / UnthBuf::BITS_PER_CELL as usize) + 1
}

#[inline(always)]
pub(crate) fn get_unaligned_element_offset_low(bitindex: usize) -> u8 {
    (bitindex % UnthBuf::BITS_PER_CELL as usize) as u8
}

#[inline(always)]
pub(crate) fn get_unaligned_element_offset_high(bitindex: usize, bits: u8) -> u8 {
    ((bitindex + bits as usize) % UnthBuf::BITS_PER_CELL as usize) as u8
}

#[inline(always)]
pub(crate) fn get_unaligned_element_mask_low(mask: usize, offset_low: u8) -> usize {
    mask << offset_low as usize
}

#[inline(always)]
pub(crate) fn get_unaligned_element_mask_high(mask: usize, offset_low: u8, bits: u8) -> usize {
    if (offset_low + bits) <= usize::BITS as u8 {
        0
    } else {
        mask >> (usize::BITS - offset_low as u32)
    }
}

// #[inline(always)]
// pub(crate) fn get_unaligned_higher_offset(bitindex: usize, bits: u8) -> u32 {
//     let offset = get_unaligned_element_offset_low(bitindex);
//     let limit = offset + (bits as u32);
//     if limit > usize::BITS {
//         limit - usize::BITS
//     } else {
//         0
//     }
// }
