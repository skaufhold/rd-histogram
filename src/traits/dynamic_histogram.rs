/// Base trait for histogram types independent of
/// implementation details
pub trait DynamicHistogram<T, C> {
    /// Type of a bin in this histogram
    type Bin;//: HistogramBin;
    //type BinIter: Iterator<Item=Self::Bin>;

    /// Instantiate a histogram with the given number of maximum bins
    fn new(n_bins: usize) -> Self;

    /// Insert a new data point into this histogram
    fn insert(&mut self, value: T, count: C);

    fn insert_iter<'a>(&mut self, values: impl IntoIterator<Item = &'a (T, C)>)
    where
        T: 'a + Copy,
        C: 'a + Copy,
    {
        for (val, count) in values {
            self.insert(*val, *count);
        }
    }

    /// Count the total number of data points in this histogram (over all bins)
    fn count(&self) -> C;

    //fn iter_bins() -> Self::BinIter;
}
