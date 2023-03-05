//! Iteration for [`UnthBuf`]
use crate::{UnthBuf, CellLayout};
use std::borrow::Cow;

impl<CL: CellLayout> UnthBuf<CL> {
    /// Returns an iterator that yields all elements contained in this buffer.
    pub fn iter(&self) -> UnthBufIter<CL> {
        UnthBufIter {
            idx: 0,
            cap: self.capacity,
            buf: Cow::Borrowed(self)
        }
    }
}

impl<'buf, CL: CellLayout + 'static> IntoIterator for &'buf UnthBuf<CL> {
    type Item = usize;
    type IntoIter = UnthBufIter<'buf, CL>;
    
    /// Creates an iterator from a reference to an [`UnthBuf`].
    fn into_iter(self) -> Self::IntoIter {
        UnthBufIter {
            idx: 0,
            cap: self.capacity,
            buf: Cow::Borrowed(self)
        }
    }
}

impl<CL: CellLayout + 'static> IntoIterator for UnthBuf<CL> {
    type Item = usize;
    type IntoIter = UnthBufIter<'static, CL>;
    
    /// Creates an iterator from an [`UnthBuf`], consuming it.
    fn into_iter(self) -> Self::IntoIter {
        UnthBufIter {
            idx: 0,
            cap: self.capacity,
            buf: Cow::Owned(self)
        }
    }
}

/// Iterator over an [`UnthBuf`]
pub struct UnthBufIter<'buf, CL: CellLayout + 'static> {
    /// The [`UnthBuf`] to iterate over.
    pub(crate) buf: Cow<'buf, UnthBuf<CL>>,
    
    /// The current index.
    pub(crate) idx: usize,
    
    /// The maximum index.
    pub(crate) cap: usize
}

impl<CL: CellLayout + 'static> core::iter::Iterator for UnthBufIter<'_, CL> {
    type Item = usize;
    
    fn next(&mut self) -> Option<Self::Item> {
        if ! self.idx < self.cap {
            return None;
        }
        
        // This is safe due to the above range-check.
        let item = unsafe {
            self.buf.get_unchecked(self.idx)
        };
        
        // On to the next item!
        self.idx += 1;
        
        Some(item)
    }
}

impl<CL: CellLayout + 'static> core::iter::ExactSizeIterator for UnthBufIter<'_, CL> {
    fn len(&self) -> usize {
        self.buf.capacity - self.idx
    }
    
    // fn is_empty(&self) -> bool {
    //     self.len() == 0
    // }
}

impl<CL: CellLayout + 'static> core::iter::FusedIterator for UnthBufIter<'_, CL> {}
