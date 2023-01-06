#![doc = include_str!("../README.md")]
#[deny(missing_docs)]

mod fmt;
pub use fmt::*;
/// A structure that holds a fixed buffer of `bits`-sized unsigned integer elements.
/// 
/// Note: If the given bit-size is `0`, the internal buffer won't be allocated, and all operations are no-ops.
pub struct UnthBuf<const ALIGNED: bool = true> {
    /// Capacity of the buffer.
    pub(crate) capacity: usize,
    
    /// Buffer of cells, containing [`Self::bits`]-sized unsigned integer elements.
    pub(crate) data: Box<[usize]>,
    
    /// Bit-size of an individual element in [`Self::data`].
    pub(crate) bits: u8,
    
    /// Mask of bits covering a single element.
    pub(crate) mask: usize,
    
    /// Elements per cell.
    /// 
    /// - When   aligned, this is an exact number.
    /// - When unaligned, this number is inexact.
    pub(crate) elpc: u8,
}

impl<const ALIGNED: bool> UnthBuf<ALIGNED> {
    /// Creates a new [`UnthBuf`] with the given `capacity` and `bits`-size,
    /// filling it with as many elements from the provided as can fit.
    pub fn new_from_capacity_and_iter(capacity: usize, bits: u8, iter: impl Iterator<Item = usize>) -> Self {
        let mut new = Self::new(capacity, bits);
        for (idx, val) in new.get_indices().zip(iter) {
            new.set(idx, val).ok();
        }
        
        new
    }
    
    /// Creates a new [`UnthBuf`] with the given `capacity` and `bits`-size,
    /// filling it with the provided `default_value`.
    pub fn new_with_default(capacity: usize, bits: u8, default_value: usize) -> Self {
        let mut new = Self::new(capacity, bits);
        if !new.fits(default_value) {
            panic!("given default value (0x{default_value:X}) does not fit into {bits} bits");
        }
        
        // The buffer is already zero-filled at creation.
        if default_value != 0 {
            new.fill(default_value);
        }
        new
    }
    
    /// Creates a new [`UnthBuf`] with the given `capacity` and `bits`-size, filled with `0`.
    pub fn new(capacity: usize, bits: u8) -> Self {
        if capacity == 0 {
            panic!("cannot create buffer of 0 capacity")
        }
        
        let size = UnthBuf::size_from_capacity_and_bits(capacity, bits, ALIGNED);
        let data = vec![0; size].into_boxed_slice();
        let mask = UnthBuf::mask_from_bits(bits);
        let elpc = UnthBuf::BITS_PER_CELL / bits;
        
        Self {
            capacity,
            data,
            bits,
            mask,
            elpc,
        }
    }
    
    /// Gets the capacity of this buffer / how many elements are stored within.
    pub fn get_capacity(&self) -> usize {
        self.capacity
    }
    
    /// Gets a range of all valid indices of this buffer.
    pub fn get_indices(&self) -> std::ops::Range<usize> {
        0..self.capacity
    }
    
    /// Gets the byte-length of this buffer.
    pub fn get_byte_len(&self) -> usize {
        self.data.len() * (usize::BITS/8) as usize
    }
    
    /// Returns the raw backing buffer.
    pub fn raw(&self) -> &[usize] {
        &self.data
    }
    
    /// Returns the raw backing buffer.
    pub fn raw_mut(&mut self) -> &mut [usize] {
        &mut self.data
    }
    
    /// Returns an iterator that yields all elements contained in this buffer.
    pub fn iter(&self) -> impl std::iter::Iterator<Item = usize> + '_ {
        // Safety Notice: Since `get_indices` will always be within bounds, this is safe.
        self.get_indices().map(|index| unsafe { self.get_unchecked(index) })
    }
    
    /// Checks if the given value can be stored in this buffer.
    pub fn fits(&self, value: usize) -> bool {
        (value & self.mask) == value
    }
    
    /// Is the given index valid for this buffer?
    pub fn is_index(&self, index: usize) -> bool {
        index < self.capacity
    }
    
    /// Is the given cell-id valid for this buffer?
    pub fn is_cell(&self, cell: usize) -> bool {
        cell < self.data.len()
    }
    
    /// Fills the buffer with the given value.
    pub fn fill(&mut self, value: usize) {
        if self.bits == 0 {
            return;
        }
        
        if !self.fits(value) {panic!("given value does not fit")}
        if value == 0 {
            self.data.fill(0);
        } else {
            for index in 0..self.capacity {
                unsafe {self.set_unchecked(index, value);}
            }
        }
    }
    
    /// Tries to set the element at the given `index` to the provided `value`.
    pub fn set(&mut self, index: usize, value: usize) -> Result<(),String> {
        if self.bits == 0 {
            return Ok(());
        }
        
        if !self.fits(value) {return Err(format!("value '{value}' does not fit within {} bit/s", self.bits))}
        if !self.is_index(index) {return Err(format!("index {index} is out of bounds ({})", self.capacity))}
        unsafe {self.set_unchecked(index, value);}
        Ok(())
    }
    
    /// Set the element at the given `index` to the provided `value`, *without* checking bounds.
    /// 
    /// # Unsafety
    /// If the index is not within `0..self.capacity`, this function will cause undefined behaviour.
    pub unsafe fn set_unchecked(&mut self, index: usize, value: usize) {
        if self.bits == 0 {
            return;
        }
        
        match ALIGNED {
            true => {
                let loc = self.aligned_location_of(index);
                let mut cell = *self.data.get_unchecked(loc.cell);
                cell &= !loc.mask; // unset the bits of the old value
                cell |= value << loc.offset; // set bits for new value
                *self.data.get_unchecked_mut(loc.cell) = cell;
            },
            false => {
                let loc = self.unaligned_location_of(index);
                if loc.mask0 != 0 {
                    let mut lcell = *self.data.get_unchecked(loc.cell);
                    lcell &= !loc.mask0; // unset the bits of the old value
                    lcell |= value << loc.offset0; // set bits for new value
                    *self.data.get_unchecked_mut(loc.cell) = lcell;
                }
                if loc.mask1 != 0 {
                    let mut hcell = *self.data.get_unchecked(loc.cell + 1);
                    hcell &= !loc.mask1; // unset the bits of the old value
                    hcell |= value >> (self.bits - loc.offset1); // set bits for new value
                    *self.data.get_unchecked_mut(loc.cell + 1) = hcell;
                }
            }
        }
    }
    
    /// Returns the element at the given `index`.
    pub fn get(&self, index: usize) -> Option<usize> {
        if self.bits == 0 {
            return Some(0);
        }
        
        if !self.is_index(index) {return None}
        return Some(unsafe {self.get_unchecked(index)})
    }
    
    /// Returns the element at the given `index`, *without* checking bounds.
    /// 
    /// # Unsafety
    /// If the index is not within `0..self.capacity`, this function will cause undefined behaviour.
    pub unsafe fn get_unchecked(&self, index: usize) -> usize {
        match ALIGNED {
            true => {
                let loc = self.aligned_location_of(index);
                //if !self.is_cell(loc.cell) {panic!("aligned cell @{index} -> {:?} out of bounds; {:?}", loc, self)}
                (self.data.get_unchecked(loc.cell) & loc.mask) >> loc.offset
            },
            false => {
                let loc = self.unaligned_location_of(index);
                //if !self.is_cell(loc.cell) {panic!("unaligned cell @{index} -> {:?} out of bounds; {:?}", loc, self)}
                let mut low = *self.data.get_unchecked(loc.cell);
                low &= loc.mask0;
                low >>= loc.offset0;
                if loc.mask1 != 0 {
                    let mut high = *self.data.get_unchecked(loc.cell + 1);
                    high &= loc.mask1;
                    high <<= self.bits - loc.offset1;
                    low |= high;
                }
                low
            }
        }
    }
    
    /// Returns the *location* of the cell, the *bit-offset* within it and a *mask*,
    /// for the element at the given index.
    #[inline(always)]
    pub(crate) fn aligned_location_of(&self, index: usize) -> UnthBufAlignedLocation {
        //if !self.is_index(index) {eprintln!("index {index} is out of bounds; {:?}", self)}
        let cell = index / (self.elpc as usize);
        //if !self.is_cell(cell) {eprintln!("index-{index} / cell-{cell} (E={elements_per_cell}) is outside the bounds of {:?}", self)}
        let offset = (index % self.elpc as usize) as u8 * self.bits;
        let mask = self.mask << offset;
        UnthBufAlignedLocation {
            cell, offset, mask
        }
    }
    
    /// Returns the *location* of the first cell, the *bit-offsets* within it and *two masks*,
    /// for the element at the given index.
    #[inline(always)]
    pub(crate) fn unaligned_location_of(&self, index: usize) -> UnthBufUnalignedLocation {
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

// TODO: Implement index-operator access; blocked on rust internals.
// impl<const ALIGNED: bool> std::ops::Index<usize> for UnthBuf<ALIGNED> {
//     type Output = usize;
//     fn index(&self, index: usize) -> &Self::Output {
//         self.get(index)
//     }
// }

impl<const ALIGNED: bool> IntoIterator for UnthBuf<ALIGNED> {
    type Item = usize;
    
    type IntoIter = std::iter::Map<std::ops::Range<usize>, Box<dyn FnMut(usize) -> usize>>;
    
    fn into_iter(self) -> Self::IntoIter {
        let indices = self.get_indices();
        let iter = Box::new(move |index| unsafe {self.get_unchecked(index)});
        indices.map(iter)
    }
}
impl UnthBuf {
    const BITS_PER_CELL: u8 = usize::BITS as u8;
    
    /// Returns a LSB-first bitmask of the given bit-length.
    /// 
    /// Special case: Zero bits will return an empty mask.
    pub(crate) fn mask_from_bits(bits: u8) -> usize {
        if bits == 0 {return 0}
        if bits as u32 == usize::BITS {return usize::MAX}
        if bits as u32 > usize::BITS {panic!("cannot store {bits} bits in usize ({})", usize::BITS)}
        2usize.pow(bits.into()) - 1
    }
    
    /// Returns the amount of [`usize`]-cells needed to fit the given `capacity` × `bits` in.
    /// 
    /// Special case: Zero bits will always return 0.
    pub(crate) fn size_from_capacity_and_bits(capacity: usize, bits: u8, aligned: bool) -> usize {
        if bits == 0 {return 0}
        match aligned {
            true  => {
                let elements_per_cell = get_aligned_elements_per_cell(bits);
                (capacity + elements_per_cell as usize) / elements_per_cell as usize
            },
            false => (capacity * bits as usize) / 8
        }
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

/// An usize-aligned location of an element.
pub(crate) struct UnthBufAlignedLocation {
    cell: usize,
    offset: u8,
    mask: usize,
}

impl std::fmt::Debug for UnthBufAlignedLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[#{} <<{} &{:b}]", self.cell, self.offset, self.mask)
    }
}

/// An unaligned location of an element.
pub(crate) struct UnthBufUnalignedLocation {
    cell: usize,
    offset0: u8,
    offset1: u8,
    mask0: usize,
    mask1: usize,
}

impl std::fmt::Debug for UnthBufUnalignedLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[#{} <<h{}l{} &h{:b}l{:b}]", self.cell, self.offset1, self.offset0, self.mask1, self.mask0)
    }
}

mod tests;
