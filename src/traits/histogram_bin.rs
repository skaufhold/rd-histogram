
/// A bin in a dynamic histogram with the value type V and count type C
pub trait HistogramBin<V, C> {
    /// central value of the bin
    fn center() -> V;

    /// number of items in the bin
    fn count() -> C;

    /// sum of all items in the bin
    fn sum() -> V;
}