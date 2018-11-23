/// Base trait for histogram types independent of
/// implementation details
pub trait DynamicHistogram<T, C> {
    /// Type of a bin in this histogram
    type Bin;

    /// Instantiate a histogram with the given number of maximum bins
    fn new(n_bins: usize) -> Self;

    /// Insert a new data point into this histogram
    fn insert(&mut self, value: T, count: C);

    /// Count the total number of data points in this histogram (over all bins)
    fn count(&self) -> C;
}
