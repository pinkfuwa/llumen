use std::ops::Range;

/// Trait for types that can be merged together and sliced.
///
/// This is the core trait that enables efficient streaming with merge operations.
/// Items in the channel buffer can be merged when consecutive items can be combined.
pub trait Mergeable
where
    Self: Sized + Clone,
{
    /// Attempt to merge another item into this one.
    ///
    /// Returns:
    /// - `None` if the merge was successful (other was absorbed into self)
    /// - `Some(other)` if the merge failed (other should be kept separate)
    fn merge(&mut self, other: Self) -> Option<Self>;

    /// Get the length of this item in logical units.
    fn len(&self) -> usize;

    /// Check if this item is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Slice this item to a specific range.
    ///
    /// Returns `None` if the slice operation is invalid (e.g., invalid UTF-8 boundary).
    fn slice(&self, r: Range<usize>) -> Option<Self>;
}
