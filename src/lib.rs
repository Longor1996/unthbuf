#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::integer_arithmetic)]
#![allow(clippy::integer_division)]
#![allow(clippy::implicit_return)]

mod iter;
mod fmt;

// cell layouts
pub mod aligned;
pub mod packed;


/// A [`UnthBuf`] using the [`aligned::AlignedLayout`].
pub type AlignedUnthBuf = UnthBuf<aligned::AlignedLayout>;

/// A [`UnthBuf`] using the [`packed::PackedLayout`].
pub type PackedUnthBuf = UnthBuf<packed::PackedLayout>;

pub use iter::UnthBufIter;

mod tests;

/// The amount of bits a single cell can hold.
pub(crate) const BITS_PER_CELL: u8 = usize::BITS as u8;

/// Bit-size of individual elements in an [`UnthBuf`].
pub type Bits = core::num::NonZeroU8;

/// A structure that holds a fixed buffer of `bits`-sized unsigned integer elements.
/// 
/// Note: If the given bit-size is `0`, the internal buffer won't be allocated, and all operations are no-ops.
/// 
/// Internally, the [`UnthBuf`] is a boxed slice of **cells**, each holding a set amount of elements.
#[derive(Clone)]
pub struct UnthBuf<CL: CellLayout> {
    /// Capacity of the buffer.
    pub(crate) capacity: usize,
    
    /// Buffer of cells, containing [`Self::bits`]-sized unsigned integer elements.
    pub(crate) data: Box<[usize]>,
    
    /// Bit-size of an individual element in [`Self::data`].
    pub(crate) bits: Bits,
    
    /// Mask of bits covering a single element.
    pub(crate) mask: usize,
    
    /// Elements per cell.
    /// 
    /// - When   aligned, this is an exact number.
    /// - When unaligned, this number is inexact.
    pub(crate) elpc: u8,
    
    /// Marker for cell layout.
    pub(crate) cell_layout: core::marker::PhantomData<CL>
}

/// The internal layout of the cells held by an [`UnthBuf`].
pub trait CellLayout: Sized + Clone + Copy {
    /// Type representing an elements location.
    type Location;
    
    /// Returns the amount of [`usize`]-cells needed to fit the given `capacity` × `bits` in.
    fn get_cell_count(capacity: usize, bits: Bits) -> usize;
    
    /// Returns the exact amount of bits that are stored, excluding any padding.
    fn get_exact_bit_count(buf: &UnthBuf<Self>) -> usize;
    
    /// Calculates the exact location of the given index.
    /// 
    /// The index is not required to be valid for this operation.
    fn location_of(buf: &UnthBuf<Self>, index: usize) -> Self::Location;
    
    /// Stores the given value at the given UNCHECKED index in the buffer.
    /// 
    /// # Safety
    /// This function is safe if the provided index was tested with [`UnthBuf::is_index`]
    unsafe fn set_unchecked(buf: &mut UnthBuf<Self>, index: usize, value: usize);
    
    /// Retrieves the value at the given UNCHECKED index from the buffer.
    /// 
    /// # Safety
    /// This function is safe if the provided index was tested with [`UnthBuf::is_index`]
    unsafe fn get_unchecked(buf: &UnthBuf<Self>, index: usize) -> usize;
}

impl<CL: CellLayout> UnthBuf<CL> {
    
    /// Creates a new [`UnthBuf`] from the given `ExactSizeIterator` and `bits`-size,
    /// filling it with all elements from the provided iterator.
    /// 
    /// # Panic
    /// - Panics if the given iterators `len()` returns `0`.
    pub fn new_from_sized_iter<I>(bits: Bits, iter: I) -> Self
        where I: Iterator<Item = usize> + core::iter::ExactSizeIterator
    {
        let mut new = Self::new(bits, iter.len());
        new.fill_from(iter);
        new
    }
    
    /// Creates a new [`UnthBuf`] with the given `capacity` and `bits`-size,
    /// filling it with as many elements from the provided iterator as can fit.
    /// 
    /// # Panic
    /// - Panics if the given `capacity` is `0`.
    pub fn new_from_capacity_and_iter<I>(bits: Bits, capacity: usize, iter: I) -> Self
        where I: Iterator<Item = usize>
    {
        let mut new = Self::new(bits, capacity);
        new.fill_from(iter);
        new
    }
    
    /// Creates a new [`UnthBuf`] with the given `capacity` and `bits`-size,
    /// filling it with the provided `default_value`.
    /// 
    /// # Panic
    /// - Panics if the given `capacity` is `0`.
    /// - Panics if the given `default_value` does not fit in `bits`.
    pub fn new_with_default(bits: Bits, capacity: usize, default_value: usize) -> Self {
        let mut new = Self::new(bits, capacity);
        assert!(new.can_element_fit(default_value), "given default value (0x{default_value:X}) does not fit into {bits} bits");
        
        // The buffer is already zero-filled at creation.
        if default_value != 0 {
            new.fill_with(default_value);
        }
        new
    }
    
    /// Creates a new [`UnthBuf`] with the given `capacity` and `bits`-size, filled with `0`.
    /// 
    /// # Panic
    /// - Panics if the given `capacity` is `0`.
    pub fn new(bits: Bits, capacity: usize) -> Self {
        assert!(capacity != 0, "cannot create buffer of 0 capacity");
        
        let size = CL::get_cell_count(capacity, bits);
        let data = vec![0; size].into_boxed_slice();
        let mask = Self::mask_from_bits(bits.get());
        let elpc = BITS_PER_CELL.checked_div(bits.get()).unwrap_or(0);
        
        Self {
            capacity,
            data,
            bits,
            mask,
            elpc,
            cell_layout: core::marker::PhantomData,
        }
    }
    
    /// Gets the capacity of this buffer / how many elements are stored within.
    #[inline(always)]
    pub fn get_capacity(&self) -> usize {
        self.capacity
    }
    
    /// Gets a range of all valid indices of this buffer; the same as `0 .. get_capacity()`.
    #[inline(always)]
    pub fn get_indices(&self) -> core::ops::Range<usize> {
        0..self.capacity
    }
    
    /// Returns the total amount of bits stored in this buffer, including all padding.
    #[inline(always)]
    pub fn get_total_bit_count(&self) -> usize {
        self.data.len() * BITS_PER_CELL as usize
    }
    
    /// Returns the exact amount of bits that are stored, excluding any padding.
    #[inline(always)]
    pub fn get_exact_bit_count(&self) -> usize {
        CL::get_exact_bit_count(self)
    }
    
    /// Returns the amount of bits that are padding.
    #[inline(always)]
    pub fn get_padding_bit_count(&self) -> usize {
        self.get_total_bit_count() - CL::get_exact_bit_count(self)
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
    
    /// Gets the length of the backing buffer, in cells.
    #[inline(always)]
    pub fn raw_len(&self) -> usize {
        self.data.len()
    }
    
    /// Gets the length of the backing buffer, in bytes.
    #[inline(always)]
    pub fn raw_byte_len(&self) -> usize {
        self.data.len() * (usize::BITS/8) as usize
    }
    
    /// Checks if the given value can be stored in this buffer.
    #[inline(always)]
    pub fn can_element_fit(&self, value: usize) -> bool {
        (value & self.mask) == value
    }
    
    /// Returns the bitmask that is used to check if elements can fit in this buffer; see [`Self::can_element_fit`].
    #[inline(always)]
    pub fn get_element_mask(&self) -> usize {
        self.mask
    }
    
    /// Returns the bit-size of the individual elements in this buffer.
    #[inline(always)]
    pub fn get_element_bits(&self) -> Bits {
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
    
    /// Returns the location of the element at the given `index`.
    pub fn location_of(&self, index: usize) -> CL::Location {
        CL::location_of(self, index)
    }
    
    /// Fills the buffer with the given value.
    /// 
    /// Filling with `0` is a memset, which is *very* fast.
    pub fn fill_with(&mut self, value: usize) {
        assert!(self.can_element_fit(value), "given value does not fit");
        
        if value == 0 {
            return self.fill_with_default()
        }
        
        for index in self.get_indices() {
            unsafe {self.set_unchecked(index, value)};
        }
    }
    
    /// Fills the buffer with `0`, clearing everything.
    /// 
    /// Filling with `0` is a memset, which is *very* fast.
    pub fn fill_with_default(&mut self) {
        self.data.fill(0);
    }
    
    /// Fills the buffer with as many values from the given iterator as possible.
    pub fn fill_from(&mut self, iter: impl Iterator<Item = usize>) {
        for (index, value) in self.get_indices().zip(iter).fuse() {
            unsafe {self.set_unchecked(index, value)};
        }
    }
    
    /// Tries to set the element at the given `index` to the provided `value`.
    /// 
    /// # Errors
    /// - If the value does not fit; check with [`Self::can_element_fit`].
    /// - If the index is out of bounds; check with [`Self::is_index`].
    #[inline]
    pub fn set(&mut self, index: usize, value: usize) -> Result<(),&'static str> {
        if !self.can_element_fit(value) {return Err("value does not fit")}
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
        CL::set_unchecked(self, index, value);
    }
    
    /// Returns the element at the given `index`.
    /// 
    /// Out-of-bounds access will return [`Option::None`].
    #[inline]
    pub fn get(&self, index: usize) -> Option<usize> {
        if !self.is_index(index) {return None}
        Some(unsafe {self.get_unchecked(index)})
    }
    
    /// Returns the element at the given `index`, *without* checking bounds.
    /// 
    /// # Safety
    /// If the index is not within `0..self.capacity`, this function will cause undefined behaviour.
    #[inline(always)]
    pub unsafe fn get_unchecked(&self, index: usize) -> usize {
        CL::get_unchecked(self, index)
    }
    
    /// Returns a LSB-first bitmask of the given bit-length.
    /// 
    /// Special case: Zero bits will return an empty mask.
    pub(crate) fn mask_from_bits(bits: u8) -> usize {
        if bits == 0 {return 0}
        if bits as u32 == usize::BITS {return usize::MAX}
        if bits as u32 > usize::BITS {
            panic!("cannot store {bits} bits in usize ({})", usize::BITS)
        }
        
        2_usize.pow(bits.into()) - 1
    }
}

// TODO: Implement index-operator access; blocked on rust internals.
// impl<const ALIGNED: bool> std::ops::Index<usize> for UnthBuf<ALIGNED> {
//     type Output = usize;
//     fn index(&self, index: usize) -> &Self::Output {
//         self.get(index)
//     }
// }
