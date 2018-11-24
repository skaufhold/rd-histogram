use super::traits::DynamicHistogram;
use num::traits::NumAssign;
use ord_subset::{OrdSubset, OrdSubsetIterExt};
use std::cmp::Ordering;

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
        if self.bins.len() > self.bins_cap {
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

impl<V: PartialOrd + OrdSubset + Copy + NumAssign, C: Copy + NumAssign + Into<V>>
    DynamicHistogram<V, C> for SimpleVecHistogram<V, C>
{
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn insert() {
        // fill with the maximum bin number
        let mut h = SimpleVecHistogram::new(5);
        let samples: &[f64] = &[1., 2., 3., 4., 5.];
        for i in samples {
            h.insert(*i, 1_u32);
        }
        for (bin, s) in h.bins().iter().zip(samples.iter()) {
            assert_eq!(*s, bin.left);
            assert_eq!(*s, bin.right);
        }
        // checks merging the closest two bins
        h.insert(5.5, 2);
        assert_eq!(
            h.bins()[4],
            Bin {
                left: 5.,
                right: 5.5,
                count: 3,
                sum: 16.
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
                sum: 21.2
            }
        );
    }
}
