#![doc = include_str!("../README.md")]
#[deny(missing_docs)]

mod iter;
mod fmt;
mod loc;

pub use iter::*;
pub use fmt::*;
pub use loc::*;

mod tests;

/// A structure that holds a fixed buffer of `bits`-sized unsigned integer elements.
/// 
/// Note: If the given bit-size is `0`, the internal buffer won't be allocated, and all operations are no-ops.
/// 
/// Internally, the [`UnthBuf`] is a boxed slice of **cells**, each holding a set amount of elements.
#[derive(Clone)]
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
    
    /// Creates a new [`UnthBuf`] from the given `ExactSizeIterator` and `bits`-size,
    /// filling it with all elements from the provided iterator.
    /// 
    /// # Panic
    /// - Panics if the given iterators `len()` returns `0`.
    pub fn new_from_sized_iter(bits: u8, iter: impl Iterator<Item = usize> + std::iter::ExactSizeIterator) -> Self {
        let mut new = Self::new(iter.len(), bits);
        for (idx, val) in new.get_indices().zip(iter) {
            new.set(idx, val).ok();
        }
        
        new
    }
    
    /// Creates a new [`UnthBuf`] with the given `capacity` and `bits`-size,
    /// filling it with as many elements from the provided iterator as can fit.
    /// 
    /// # Panic
    /// - Panics if the given `capacity` is `0`.
    pub fn new_from_capacity_and_iter(capacity: usize, bits: u8, iter: impl Iterator<Item = usize>) -> Self {
        let mut new = Self::new(capacity, bits);
        for (idx, val) in new.get_indices().zip(iter) {
            new.set(idx, val).ok();
        }
        
        new
    }
    
    /// Creates a new [`UnthBuf`] with the given `capacity` and `bits`-size,
    /// filling it with the provided `default_value`.
    /// 
    /// # Panic
    /// - Panics if the given `capacity` is `0`.
    /// - Panics if the given `default_value` does not fit in `bits`.
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
    /// 
    /// # Panic
    /// - Panics if the given `capacity` is `0`.
    pub fn new(capacity: usize, bits: u8) -> Self {
        if capacity == 0 {
            panic!("cannot create buffer of 0 capacity")
        }
        
        let size = UnthBuf::size_from_capacity_and_bits(capacity, bits, ALIGNED);
        let data = vec![0; size].into_boxed_slice();
        let mask = UnthBuf::mask_from_bits(bits);
        let elpc = UnthBuf::BITS_PER_CELL.checked_div(bits).unwrap_or(0);
        
        Self {
            capacity,
            data,
            bits,
            mask,
            elpc,
        }
    }
    
    /// Gets the capacity of this buffer / how many elements are stored within.
    #[inline(always)]
    pub fn get_capacity(&self) -> usize {
        self.capacity
    }
    
    /// Gets a range of all valid indices of this buffer.
    #[inline(always)]
    pub fn get_indices(&self) -> std::ops::Range<usize> {
        0..self.capacity
    }
    
    /// Gets the total amount of bits stored in this buffer.
    #[inline(always)]
    pub fn get_stored_bit_count(&self) -> usize {
        self.capacity * self.bits as usize
    }
    
    // /// Gets the total amount of stored bytes in this buffer.
    // pub fn get_stored_byte_count(&self) -> usize {
    //     (self.capacity * self.bits as usize).div_ceil(8)
    // }
    
    /// Returns a reference to the raw backing buffer of cells.
    #[inline(always)]
    pub fn raw(&self) -> &[usize] {
        &self.data
    }
    
    /// Returns a mutable reference to the raw backing buffer of cells.
    #[inline(always)]
    pub fn raw_mut(&mut self) -> &mut [usize] {
        &mut self.data
    }
    
    /// Gets the length of the backing buffer in bytes.
    #[inline(always)]
    pub fn raw_byte_len(&self) -> usize {
        self.data.len() * (usize::BITS/8) as usize
    }
    
    /// Checks if the given value can be stored in this buffer.
    #[inline(always)]
    pub fn fits(&self, value: usize) -> bool {
        (value & self.mask) == value
    }
    
    /// Returns the bitmask that is used to check if elements can fit in this buffer; see [`Self::fits`].
    #[inline(always)]
    pub fn get_mask(&self) -> usize {
        self.mask
    }
    
    /// Returns the bit-size of the individual elements in this buffer.
    #[inline(always)]
    pub fn get_bits(&self) -> u8 {
        self.bits
    }
    
    /// Is the given index (`0..self.capacity`) valid for this buffer?
    #[inline(always)]
    pub fn is_index(&self, index: usize) -> bool {
        index < self.capacity
    }
    
    /// Is the given cell-id (`0..self.data.len()`) valid for this buffer?
    /// 
    /// i.e.: Does this point inside the *backing buffer*?
    #[inline(always)]
    pub fn is_cell(&self, cell: usize) -> bool {
        cell < self.data.len()
    }
    
    /// Fills the buffer with the given value.
    /// 
    /// Filling with `0` calls [`core::slice::fill`]/memset, which is *very* fast.
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
    #[inline]
    pub fn set(&mut self, index: usize, value: usize) -> Result<(),&'static str> {
        if self.bits == 0 {
            return Ok(());
        }
        
        if !self.fits(value) {return Err("value does not fit")}
        if !self.is_index(index) {return Err("index is out-of-bounds")}
        unsafe {self.set_unchecked(index, value);}
        Ok(())
    }
    
    /// Set the element at the given `index` to the provided `value`, *without* checking bounds.
    /// 
    /// # Safety
    /// If the index is not within `0..self.capacity`, testable via [`Self::is_index`], this function will cause undefined behaviour.
    #[inline(always)]
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
    /// 
    /// Out-of-bounds access will return [`Option::None`].
    #[inline]
    pub fn get(&self, index: usize) -> Option<usize> {
        if self.bits == 0 {
            return Some(0);
        }
        
        if !self.is_index(index) {return None}
        Some(unsafe {self.get_unchecked(index)})
    }
    
    /// Returns the element at the given `index`, *without* checking bounds.
    /// 
    /// # Safety
    /// If the index is not within `0..self.capacity`, this function will cause undefined behaviour.
    #[inline(always)]
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
}

// TODO: Implement index-operator access; blocked on rust internals.
// impl<const ALIGNED: bool> std::ops::Index<usize> for UnthBuf<ALIGNED> {
//     type Output = usize;
//     fn index(&self, index: usize) -> &Self::Output {
//         self.get(index)
//     }
// }

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
