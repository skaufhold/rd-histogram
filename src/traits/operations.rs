pub trait EmptyClone {
    /// Return an empty clone of the item that has otherwise
    /// identical attributes (e.g. number of maximum bins)
    fn empty_clone(&self) -> Self;
}

pub trait MergeIter<H> {
    /// Merge all histograms in an iterator into
    /// one histogram
    fn merge(self) -> Option<H>;
}

impl<'b, H, Iter> MergeIter<H> for Iter
where
    H: 'b + MergeRef + EmptyClone,
    Iter: Iterator<Item = &'b H>,
{
    fn merge(self) -> Option<H> {
        let mut peekable = self.peekable();
        let seed = peekable.peek()?.empty_clone();
        Some(peekable.fold(seed, |mut agg, item| {
            agg.merge_ref(item);
            agg
        }))
    }
}

pub trait Merge {
    /// Merge another instance of this type into this histogram
    fn merge(&mut self, other: Self);
}

pub trait MergeRef {
    /// Merge another instance of this type into this histogram
    fn merge_ref(&mut self, other: &Self);
}

pub trait Median<T> {
    /// Estimate the median value of the data points in this histogram
    fn median(&self) -> Option<T>;
}
