use num::traits::NumAssign;
use ord_subset::{OrdSubset, OrdSubsetIterExt, OrdSubsetSliceExt};
use std::cmp::Ordering;
use crate::traits::{DynamicHistogram, EmptyClone, Merge, MergeRef};

#[derive(Clone, Debug, PartialEq)]
pub struct SimpleVecHistogram<V, C> {
    bins: Vec<Bin<V, C>>,
    bins_cap: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bin<V, C> {
    left: V,
    right: V,
    count: C,
    sum: V,
}

impl<V: PartialOrd + OrdSubset + NumAssign + Copy, C: Clone + NumAssign + Copy>
SimpleVecHistogram<V, C>
{
    fn search_bins(&self, value: V) -> Result<usize, usize> {
        self.bins.binary_search_by(|probe| {
            if probe.left <= value && probe.right > value {
                Ordering::Equal
            } else if probe.left > value {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        })
    }

    fn shrink_to_fit(&mut self) {
        while self.bins.len() > self.bins_cap {
            let merge_result = self
                .bins
                .iter()
                .zip(self.bins.iter().skip(1))
                .enumerate()
                .map(|(i, (bin, next_bin))| {
                    // calculate distances between bins
                    (i, bin, next_bin, next_bin.left - bin.right)
                })
                .ord_subset_min_by_key(|(_i, _bin, _next_bin, distance)| *distance)
                .map(|(i, bin, next_bin, _)| {
                    let merged_bin = Bin {
                        left: bin.left,
                        right: next_bin.right,
                        sum: bin.sum + next_bin.sum,
                        count: bin.count + next_bin.count,
                    };
                    (i, merged_bin)
                });

            if let Some((i, merged_bin)) = merge_result {
                self.bins[i] = merged_bin;
                self.bins.remove(i + 1);
            }
        }
    }

    fn bins(&self) -> &[Bin<V, C>] {
        self.bins.as_slice()
    }
}

impl<V, C> DynamicHistogram<V, C> for SimpleVecHistogram<V, C>
    where V: PartialOrd + OrdSubset + Copy + NumAssign, C: Copy + NumAssign + Into<V> {
    /// Type of a bin in this histogram
    type Bin = Bin<V, C>;

    /// Instantiate a histogram with the given number of maximum bins
    fn new(n_bins: usize) -> Self {
        SimpleVecHistogram {
            bins: Vec::with_capacity(n_bins),
            bins_cap: n_bins,
        }
    }

    /// Insert a new data point into this histogram
    fn insert(&mut self, value: V, count: C) {
        let search_result = self.search_bins(value);

        match search_result {
            Ok(found) => {
                self.bins[found].count += count;
                self.bins[found].sum += value * count.into();
            }
            Err(insert_at) => self.bins.insert(
                insert_at,
                Bin {
                    left: value,
                    right: value,
                    count,
                    sum: value * count.into(),
                },
            ),
        }

        self.shrink_to_fit();
    }

    /// Count the total number of data points in this histogram (over all bins)
    fn count(&self) -> C {
        unimplemented!()
    }
}

impl<V, C> EmptyClone for SimpleVecHistogram<V, C>
    where V: PartialOrd + OrdSubset + Copy + NumAssign, C: Copy + NumAssign + Into<V> {
    fn empty_clone(&self) -> Self {
        SimpleVecHistogram::new(self.bins_cap)
    }
}

impl<V, C> MergeRef for SimpleVecHistogram<V, C>
    where V: PartialOrd + OrdSubset + Copy + NumAssign, C: Copy + NumAssign + Into<V> {
    fn merge_ref(&mut self, other: &Self) {
        self.bins.extend_from_slice(other.bins());
        self.bins.ord_subset_sort_by_key(|b| b.left);
        self.shrink_to_fit();
    }
}

impl<V, C> Merge for SimpleVecHistogram<V, C>
    where V: PartialOrd + OrdSubset + Copy + NumAssign, C: Copy + NumAssign + Into<V> {
    fn merge(&mut self, other: Self) {
        self.bins.extend(other.bins.into_iter());
        self.bins.ord_subset_sort_by_key(|b| b.left);
        self.shrink_to_fit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        // fill with the maximum bin number
        let mut h = SimpleVecHistogram::new(5);
        let samples = &[(1_f64, 1_u32), (2., 1), (3., 1), (4., 1), (5., 1)];
        h.insert_iter(samples);

        for (bin, s) in h.bins().iter().zip(samples.iter()) {
            assert_eq!(s.0, bin.left);
            assert_eq!(s.0, bin.right);
        }
        // checks merging the closest two bins
        h.insert(5.5, 2);
        assert_eq!(
            h.bins()[4],
            Bin {
                left: 5.,
                right: 5.5,
                count: 3,
                sum: 16.,
            }
        );
        // checks inserting into an existing bin
        h.insert(5.2, 1);
        assert_eq!(
            h.bins()[4],
            Bin {
                left: 5.,
                right: 5.5,
                count: 4,
                sum: 21.2,
            }
        );
    }

    #[test]
    fn merge() {
        // fill with the maximum bin number
        let samples1 = &[(1., 1), (2., 1), (3., 1), (4., 1), (5., 1)];
        let samples2 = &[(1.1, 1), (2.1, 1), (3.1, 1), (4.1, 1), (5.1, 1)];

        let mut h = SimpleVecHistogram::new(5);
        h.insert_iter(samples1);
        println!("{:?}", h);

        let mut h2 = SimpleVecHistogram::new(5);
        h2.insert_iter(samples2);
        println!("{:?}", h2);

        h.merge(h2);
        println!("{:?}", h);
        assert_eq!(h.bins().len(), 5);
        assert_eq!(
            h.bins()[2],
            Bin {
                left: 3.,
                right: 3.1,
                count: 2,
                sum: 6.1,
            }
        );
        assert_eq!(
            h.bins()[4],
            Bin {
                left: 5.,
                right: 5.1,
                count: 2,
                sum: 10.1,
            }
        );
    }
}
