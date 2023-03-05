//! Layout that stores integers tightly packed, *across* word boundaries.
use crate::{UnthBuf, CellLayout, Bits, BITS_PER_CELL};

/// Layout that stores integers tightly packed, *across* word boundaries.
#[derive(Clone, Copy)]
pub struct PackedLayout;

impl CellLayout for PackedLayout {
    type Location = PackedLocation;
    
    #[inline(always)]
    fn get_cell_count(capacity: usize, bits: Bits) -> usize {
        (capacity * bits.get() as usize) / 8
    }
    
    fn get_exact_bit_count(buf: &UnthBuf<Self>) -> usize {
        buf.capacity * buf.bits.get() as usize
    }
    
    #[inline(always)]
    fn location_of(buf: &UnthBuf<Self>, index: usize) -> Self::Location {
        //if !self.is_index(index) {eprintln!("index {index} is out of bounds; {:?}", self)}
        let bitindex = get_packed_bitindex(index, buf.bits.get());
        let cell = get_packed_cellindex_low(bitindex);
        //if !self.is_cell(cell) {eprintln!("cell {cell} is out of bounds; {:?}", self)}
        let offset_low = get_packed_element_offset_low(bitindex);
        let offset_high = get_packed_element_offset_high(bitindex, buf.bits.get());
        let mask0 = get_packed_element_mask_low(buf.mask, offset_low);
        let mask1 = get_packed_element_mask_high(buf.mask, offset_low, buf.bits.get());
        
        PackedLocation {
            cell,
            offset0: offset_low,
            offset1: offset_high,
            mask0,
            mask1
        }
    }

    #[inline(always)]
    unsafe fn set_unchecked(buf: &mut UnthBuf<Self>, index: usize, value: usize) {
        let location = Self::location_of(buf, index);
        
        if location.mask0 != 0 {
            let mut lcell = *buf.data.get_unchecked(location.cell);
            
            lcell &= !location.mask0; // unset the bits of the old value
            lcell |= value << location.offset0; // set bits for new value
            
            *buf.data.get_unchecked_mut(location.cell) = lcell;
        }
        
        if location.mask1 != 0 {
            let mut hcell = *buf.data.get_unchecked(location.cell + 1);
            
            hcell &= !location.mask1; // unset the bits of the old value
            hcell |= value >> (buf.bits.get() - location.offset1); // set bits for new value
            
            *buf.data.get_unchecked_mut(location.cell + 1) = hcell;
        }
    }

    #[inline(always)]
    unsafe fn get_unchecked(buf: &UnthBuf<Self>, index: usize) -> usize {
        let location = Self::location_of(buf, index);
        
        //if !buf.is_cell(loc.cell) {panic!("unaligned cell @{index} -> {:?} out of bounds; {:?}", loc, self)}
        
        let mut low = *buf.data.get_unchecked(location.cell);
        
        low &= location.mask0;
        low >>= location.offset0;
        
        if location.mask1 != 0 {
            let mut high = *buf.data.get_unchecked(location.cell + 1);
            
            high &= location.mask1;
            high <<= buf.bits.get() - location.offset1;
            low |= high;
        }
        
        low
    }
}

/// An unaligned location of an element within an [`UnthBuf`].
#[derive(Clone, Copy)]
pub struct PackedLocation {
    pub(crate) cell: usize,
    pub(crate) offset0: u8,
    pub(crate) offset1: u8,
    pub(crate) mask0: usize,
    pub(crate) mask1: usize,
}

impl core::fmt::Debug for PackedLocation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[#{} <<h{}l{} &h{:b}l{:b}]", self.cell, self.offset1, self.offset0, self.mask1, self.mask0)
    }
}

#[inline(always)]
pub(crate) fn get_packed_bitindex(index: usize, bits: u8) -> usize {
    index * bits as usize
}

#[inline(always)]
pub(crate) fn get_packed_cellindex_low(bitindex: usize) -> usize {
    bitindex / BITS_PER_CELL as usize
}

#[inline(always)]
#[allow(dead_code)]
pub(crate) fn get_packed_cellindex_high(bitindex: usize) -> usize {
    (bitindex / BITS_PER_CELL as usize) + 1
}

#[inline(always)]
pub(crate) fn get_packed_element_offset_low(bitindex: usize) -> u8 {
    (bitindex % BITS_PER_CELL as usize) as u8
}

#[inline(always)]
pub(crate) fn get_packed_element_offset_high(bitindex: usize, bits: u8) -> u8 {
    ((bitindex + bits as usize) % BITS_PER_CELL as usize) as u8
}

#[inline(always)]
pub(crate) fn get_packed_element_mask_low(mask: usize, offset_low: u8) -> usize {
    mask << offset_low as usize
}

#[inline(always)]
pub(crate) fn get_packed_element_mask_high(mask: usize, offset_low: u8, bits: u8) -> usize {
    if (offset_low + bits) <= usize::BITS as u8 {
        0
    } else {
        mask.overflowing_shr(usize::BITS - offset_low as u32).0
    }
}

// #[inline(always)]
// pub(crate) fn get_packed_higher_offset(bitindex: usize, bits: u8) -> u32 {
//     let offset = get_packed_element_offset_low(bitindex);
//     let limit = offset + (bits as u32);
//     if limit > usize::BITS {
//         limit - usize::BITS
//     } else {
//         0
//     }
// }
