use super::traits::DynamicHistogram;
use binary_heap_plus::{BinaryHeap, MinComparator};
use std::cmp::Ordering;
use std::ops::AddAssign;
use std::rc::{Rc, Weak};

#[derive(Clone)]
pub struct Histogram<V: Ord, C> {
    bins: Vec<Rc<Bin<V, C>>>,
    distances: BinaryHeap<BinDistance<V, C>, MinComparator>,
}

#[derive(Clone)]
pub struct Bin<V, C> {
    left: V,
    right: V,
    count: C,
    sum: V,
}

#[derive(Clone)]
pub struct BinDistance<V, C> {
    left: Weak<Bin<V, C>>,
    right: Weak<Bin<V, C>>,
    distance: V,
}

impl<V: PartialEq, C> PartialEq for BinDistance<V, C> {
    fn eq(&self, other: &Self) -> bool {
        self.distance.eq(&other.distance)
    }
}

impl<V: Eq, C> Eq for BinDistance<V, C> {}

impl<V: PartialOrd, C> PartialOrd for BinDistance<V, C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.distance.partial_cmp(&other.distance)
    }
}

impl<V: Ord, C> Ord for BinDistance<V, C> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance.cmp(&other.distance)
    }
}

impl<V: Ord + Clone, C: Clone + AddAssign> DynamicHistogram<V, C> for Histogram<V, C> {
    /// Type of a bin in this histogram
    type Bin = Bin<V, C>;

    /// Instantiate a histogram with the given number of maximum bins
    fn new(n_bins: usize) -> Self {
        Histogram {
            bins: vec![],
            distances: BinaryHeap::new_min(),
        }
    }

    /// Insert a new data point into this histogram
    fn insert(&mut self, value: V, count: C) {
        let search_result = self.bins.binary_search_by(|probe| {
            if probe.left <= value && probe.right > value {
                Ordering::Equal
            } else if probe.left > value {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        });

        match search_result {
            Ok(found) => {
                unimplemented!();
            }
            Err(insert_at) => unimplemented!(),
        }
    }

    /// Count the total number of data points in this histogram (over all bins)
    fn count(&self) -> C {
        unimplemented!()
    }
}
