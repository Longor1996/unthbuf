//! Iteration for [`UnthBuf`]
use super::*;

impl<const ALIGNED: bool> UnthBuf<ALIGNED> {
    /// Returns an iterator that yields all elements contained in this buffer.
    pub fn iter(&self) -> UnthBufIter<ALIGNED> {
        UnthBufIter {
            buf: std::borrow::Cow::Borrowed(self),
            idx: 0
        }
    }
}

impl<const ALIGNED: bool> IntoIterator for UnthBuf<ALIGNED> {
    type Item = usize;
    type IntoIter = UnthBufIter<'static, ALIGNED>;
    
    fn into_iter(self) -> Self::IntoIter {
        UnthBufIter {
            buf: std::borrow::Cow::Owned(self),
            idx: 0,
        }
    }
}

/// Iterator over an UnthBuf
pub struct UnthBufIter<'b, const ALIGNED: bool> {
    /// The [`UnthBuf`] to iterate over.
    pub(crate) buf: std::borrow::Cow<'b, UnthBuf<ALIGNED>>,
    
    /// The current index.
    pub(crate) idx: usize
}

impl<const ALIGNED: bool> std::iter::Iterator for UnthBufIter<'_, {ALIGNED}> {
    type Item = usize;
    
    fn next(&mut self) -> Option<Self::Item> {
        if ! self.buf.is_index(self.idx) {
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

impl<const ALIGNED: bool> std::iter::ExactSizeIterator for UnthBufIter<'_, {ALIGNED}> {
    fn len(&self) -> usize {
        self.buf.capacity - self.idx
    }
}

impl<const ALIGNED: bool> std::iter::FusedIterator for UnthBufIter<'_, {ALIGNED}> {}
