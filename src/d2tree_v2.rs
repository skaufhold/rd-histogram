use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::rc::Rc;

use id_vec::Id;
use id_vec::IdVec;
use num::traits::NumAssign;
use ord_subset::OrdVar;

use crate::traits::DynamicHistogram;
use std::fmt::Debug;
use ord_subset::OrdSubset;

#[derive(Clone, Debug)]
pub struct Histogram<V: PartialOrd, C> {
    bins: IdVec<Bin<V, C>>,
    bins_ascending: Vec<Id<Bin<V, C>>>,
    distances: BTreeSet<Rc<BinDistance<V, C>>>,
    n_bins: usize,
}

#[derive(Clone, Debug)]
pub struct Bin<V: PartialOrd, C> {
    left: V,
    right: V,
    count: C,
    sum: V,
    neighbor_distance_right: Option<Rc<BinDistance<V, C>>>,
}

#[derive(Clone, Debug)]
pub struct BinDistance<V: PartialOrd, C> {
    left: Id<Bin<V, C>>,
    right: Id<Bin<V, C>>,
    distance: OrdVar<V>,
}

impl<V: PartialOrd, C> PartialEq for BinDistance<V, C> {
    fn eq(&self, other: &Self) -> bool {
        self.distance.eq(&other.distance)
            && self.left == other.left
            && self.right == other.right
    }
}

impl<V: PartialOrd, C> Eq for BinDistance<V, C> {}

impl<V: PartialOrd, C> PartialOrd for BinDistance<V, C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let o = self.distance.partial_cmp(&other.distance);
        if let Some(Ordering::Equal) = o {
            if self.left == other.left && self.right == other.right {
                Some(Ordering::Equal)
            } else { Some(Ordering::Greater) }
        } else {
            o
        }
    }
}

impl<V: PartialOrd, C> Ord for BinDistance<V, C> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<V: PartialOrd + Debug + OrdSubset, C: Debug> DynamicHistogram<V, C> for Histogram<V, C>
    where
        V: Copy + NumAssign,
        C: Copy + NumAssign + Into<V>,
{
    /// Type of a bin in this histogram
    type Bin = Bin<V, C>;

    /// Instantiate a histogram with the given number of maximum bins
    fn new(n_bins: usize) -> Self {
        Histogram {
            bins: IdVec::new(),
            bins_ascending: vec![],
            distances: BTreeSet::new(),
            n_bins,
        }
    }

    /// Insert a new data point into this histogram
    fn insert(&mut self, value: V, count: C) {
        let search_result = self.search_bins(value);

        match search_result {
            Ok(found) => {
                let found_id = self.bins_ascending[found];
                let found_bin = &mut self.bins[found_id];
                found_bin.count += count;
                found_bin.sum += value * count.into();
            }
            Err(insert_at) => {
                let new_bin_id = self.bins.insert(Bin {
                    left: value,
                    right: value,
                    count,
                    sum: count.into() * value,
                    neighbor_distance_right: None,
                });

                let opt_bin_before_id = if insert_at > 0 {
                    self.bins_ascending.get(insert_at - 1)
                } else { None };
                let opt_bin_after_id = self.bins_ascending.get(insert_at);

                // remove obsolete distance when inserting somewhere in the middle
                if let (Some(bin_before_id), Some(_bin_after_id)) = (opt_bin_before_id, opt_bin_after_id) {
                    let bin_before = &mut self.bins[*bin_before_id];
                    self.distances.remove(bin_before.neighbor_distance_right.as_ref().unwrap());
                }

                // add distance before the new bin
                if let Some(bin_before_id) = opt_bin_before_id {
                    let distance_before_new_bin = Rc::new(BinDistance {
                        left: *bin_before_id,
                        right: new_bin_id,
                        distance: OrdVar::new(self.bins[new_bin_id].left - self.bins[*bin_before_id].right),
                    });
                    self.bins[*bin_before_id].neighbor_distance_right = Some(distance_before_new_bin.clone());
                    self.distances.insert(distance_before_new_bin);
                }

                // add distance after the new bin
                if let Some(bin_after_id) = opt_bin_after_id {
                    let distance_after_new_bin = Rc::new(BinDistance {
                        left: new_bin_id,
                        right: *bin_after_id,
                        distance: OrdVar::new(self.bins[*bin_after_id].left - self.bins[new_bin_id].right),
                    });
                    self.distances.insert(distance_after_new_bin.clone());
                    dbg!(&distance_after_new_bin);
                    self.bins[new_bin_id].neighbor_distance_right = Some(distance_after_new_bin)
                }

                self.bins_ascending.insert(insert_at, new_bin_id);
                println!("Inserted bin at {}", insert_at);
                dbg!(&self.bins_ascending);
                dbg!(&self.bins);
                dbg!(&self.distances);
                // insert new bin into the sorted list

                self.shrink_to_fit();
            }
        }
    }

    /// Count the total number of data points in this histogram (over all bins)
    fn count(&self) -> C {
        unimplemented!()
    }
}

impl<V: PartialOrd, C> Histogram<V, C> where V: Copy + Debug + NumAssign, C: NumAssign + Clone + Debug {
    fn shrink_to_fit(&mut self) {
        if self.n_bins >= self.bins_ascending.len() { return; }
        let overfill_amount = self.bins_ascending.len() - self.n_bins;
        println!("Shrink from {} to {}, merge {} time(s)", self.bins.len(), self.n_bins, overfill_amount);
        let mut shortest_distances = self.distances
            .iter()
            .take(overfill_amount + 1)
            .cloned()
            .collect::<Vec<_>>();
        self.distances = self.distances.split_off(&shortest_distances[1]);
        shortest_distances.remove(shortest_distances.len() - 1);
        dbg!(&shortest_distances);
        dbg!(&self.distances);
        for shortest_distance in shortest_distances {
            let right_bin = self.bins[shortest_distance.right].clone();
            let left_bin = &mut self.bins[shortest_distance.left];
            left_bin.right = right_bin.right;
            left_bin.neighbor_distance_right = right_bin.neighbor_distance_right.clone();
            left_bin.count += right_bin.count;
            left_bin.sum += right_bin.sum;

            self.bins.remove(shortest_distance.right);
        }
        let bins = &self.bins;
        self.bins_ascending.retain(|id| bins.contains_id(*id));
        dbg!(&self.bins_ascending);
        dbg!(&self.bins);
    }

    fn search_bins(&mut self, value: V) -> Result<usize, usize> {
        self.bins_ascending.binary_search_by(|probe| {
            let bin = &self.bins[*probe];
            if bin.left <= value && bin.right > value {
                Ordering::Equal
            } else if bin.left > value {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        })
    }

    pub fn bin(&self, n: usize) -> &Bin<V, C> {
        &self.bins[self.bins_ascending[n]]
    }
}

mod tests {
    use super::*;

    #[test]
    fn insert() {
        // fill with the maximum bin number
        let mut h = Histogram::new(5);
        let samples = &[(1_f64, 1_u32), (2., 1), (3., 1), (4., 1), (5., 1)];
        h.insert_iter(samples);

        //for (bin, s) in h.bins().iter().zip(samples.iter()) {
        //    assert_eq!(s.0, bin.left);
        //    assert_eq!(s.0, bin.right);
        //}

        // checks merging the closest two bins
        h.insert(5.5, 2);
        {
            let bin = h.bin(4);
            assert_eq!(bin.left, 5.);
            assert_eq!(bin.right, 5.5);
            assert_eq!(bin.count, 3);
            assert_eq!(bin.sum, 16.);
        }

        // checks inserting into an existing bin
        h.insert(5.2, 1);
        {
            let bin = h.bin(4);
            assert_eq!(bin.left, 5.);
            assert_eq!(bin.right, 5.5);
            assert_eq!(bin.count, 4);
            assert_eq!(bin.sum, 21.2);
        }
    }

    /*#[test]
    fn merge() {
        // fill with the maximum bin number
        let samples1: &[(f64, u32)] = &[(1., 1), (2., 1), (3., 1), (4., 1), (5., 1)];
        let samples2: &[(f64, u32)] = &[(1.1, 1), (2.1, 1), (3.1, 1), (4.1, 1), (5.1, 1)];

        let mut h = Histogram::new(5);
        h.insert_iter(samples1);
        println!("{:?}", h);

        let mut h2 = Histogram::new(5);
        h2.insert_iter(samples2);
        println!("{:?}", h2);

        h.merge(h2);
        println!("{:?}", h);
        assert_eq!(h.bins().len(), 5);

        let bin2 = h.bin(2);
        assert_eq!(bin2.left, 3);
        assert_eq!(bin2.right, 3.1);
        assert_eq!(bin2.count, 2);
        assert_eq!(bin2.sum, 6.1);

        let bin4 = h.bin(4);
        assert_eq!(bin4.left, 5);
        assert_eq!(bin4.right, 5.1);
        assert_eq!(bin4.count, 2);
        assert_eq!(bin4.sum, 10.1);
    }*/
}
