#![allow(dead_code)]

trait FSpec: Copy {
    fn unit() -> Self;
    fn accumulate(self, other: Self) -> Self;
}

trait TSpec: FSpec {
    type F: FSpec;
    fn apply(self, op: Self::F, cnt: usize) -> Self;
}

struct SegTree<T: TSpec> {
    vals: Vec<T>,
    tags: Vec<T::F>,
    n: usize,
    leaf_cnt: usize,
}

impl<T: TSpec> SegTree<T> {
    fn new(n: usize) -> Self {
        assert!(n > 0);
        let leaf_cnt = n.next_power_of_two();
        let vals = vec![T::unit(); leaf_cnt << 1];
        let tags = vec![T::F::unit(); leaf_cnt];
        Self {
            vals,
            tags,
            n,
            leaf_cnt,
        }
    }

    fn build_from(&mut self, arr: &[T]) {
        assert!(arr.len() == self.n);
        self.tags.fill(T::F::unit());
        self.vals[self.leaf_cnt..self.leaf_cnt + self.n].copy_from_slice(arr);
        for i in (1..self.leaf_cnt).rev() {
            self._merge(i);
        }
    }

    fn update(&mut self, ql: usize, qr: usize, x: T::F) {
        assert!(ql <= qr && qr < self.n);
        let (l0, r0) = (self.leaf_cnt + ql, self.leaf_cnt + qr + 1);
        self._push_path(l0);
        self._push_path(r0 - 1);
        let (mut l, mut r) = (l0, r0);
        let h = self.leaf_cnt.trailing_zeros();
        for shift in 0..=h {
            let cnt = 1 << shift;
            if shift > 0 {
                if ((l0 >> shift) << shift) != l0 {
                    // When the condition is true, there exists any bit 1 be eaten by shift,
                    // which indicates that an update occurs on child nodes of l0's `shift`-th
                    // ancestor. So we should update it by `_merge`.
                    self._merge(l0 >> shift);
                }
                if ((r0 >> shift) << shift) != r0 {
                    // The same condition as above.
                    self._merge((r0 - 1) >> shift);
                }
            }
            if l < r {
                // In `query`, we can safely break the loop when `l == r` since the query range is
                // fully handled. But in `update`, we should continue ascend to root to update
                // values of ancestors.
                // So we need this `l < r` guard to avoid updating incorrectly when `l == r`.
                if (l & 1) == 1 {
                    // If l is odd, it is a right child and its left sibling is outside of the update,
                    // so update it before ascending.
                    self._apply(l, x, cnt);
                    l += 1;
                }
                if (r & 1) == 1 {
                    // If r is odd, its left sibling should be updated,
                    // so update it before ascending.
                    r -= 1;
                    self._apply(r, x, cnt);
                }
            }
            (l, r) = (l >> 1, r >> 1);
        }
    }

    fn query(&mut self, ql: usize, qr: usize) -> T {
        assert!(ql <= qr && qr < self.n);
        let (mut l, mut r) = (self.leaf_cnt + ql, self.leaf_cnt + qr + 1);
        self._push_path(l);
        self._push_path(r - 1);
        let (mut l_res, mut r_res) = (T::unit(), T::unit());
        while l < r {
            if (l & 1) == 1 {
                // If l is odd, it is a right child and its left sibling is outside of the query,
                // so add it to result before ascending.
                l_res = l_res.accumulate(self.vals[l]);
                l += 1;
            }
            if (r & 1) == 1 {
                // If r is odd, its left sibling should be added to result,
                // so add it to result before ascending.
                r -= 1;
                r_res = self.vals[r].accumulate(r_res);
            }
            (l, r) = (l >> 1, r >> 1);
        }
        l_res.accumulate(r_res)
    }

    #[inline]
    fn _apply(&mut self, i: usize, x: T::F, cnt: usize) {
        self.vals[i] = self.vals[i].apply(x, cnt);
        if i < self.leaf_cnt {
            self.tags[i] = self.tags[i].accumulate(x);
        }
    }

    #[inline]
    fn _push(&mut self, i: usize, cnt: usize) {
        let x = self.tags[i];
        self.tags[i] = T::F::unit();
        self._apply(i << 1, x, cnt >> 1);
        self._apply(i << 1 | 1, x, cnt >> 1);
    }

    // propagate tags in ancestors of *leaf* i
    #[inline]
    fn _push_path(&mut self, i: usize) {
        let h = self.leaf_cnt.trailing_zeros();
        for shift in (1..=h).rev() {
            self._push(i >> shift, 1 << shift);
        }
    }

    #[inline]
    fn _merge(&mut self, i: usize) {
        self.vals[i] = self.vals[i << 1].accumulate(self.vals[i << 1 | 1]);
    }
}

#[cfg(test)]
mod tests {
    use super::{FSpec, SegTree, TSpec};

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct AffineUpdate {
        multiplier: i64,
        offset: i64,
    }

    impl FSpec for AffineUpdate {
        fn unit() -> Self {
            Self {
                multiplier: 1,
                offset: 0,
            }
        }

        fn accumulate(self, newer: Self) -> Self {
            Self {
                multiplier: newer.multiplier * self.multiplier,
                offset: newer.multiplier * self.offset + newer.offset,
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Sum(i64);

    impl FSpec for Sum {
        fn unit() -> Self {
            Self(0)
        }

        fn accumulate(self, other: Self) -> Self {
            Self(self.0 + other.0)
        }
    }

    impl TSpec for Sum {
        type F = AffineUpdate;

        fn apply(self, operation: Self::F, element_count: usize) -> Self {
            Self(operation.multiplier * self.0 + operation.offset * element_count as i64)
        }
    }

    #[derive(Clone, Copy)]
    struct Identity;

    impl FSpec for Identity {
        fn unit() -> Self {
            Self
        }

        fn accumulate(self, _other: Self) -> Self {
            Self
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Composition {
        multiplier: u64,
        offset: u64,
    }

    impl FSpec for Composition {
        fn unit() -> Self {
            Self {
                multiplier: 1,
                offset: 0,
            }
        }

        fn accumulate(self, other: Self) -> Self {
            // Compose self(other(x)) modulo 97 to keep products bounded.
            Self {
                multiplier: self.multiplier * other.multiplier % 97,
                offset: (self.multiplier * other.offset + self.offset) % 97,
            }
        }
    }

    impl TSpec for Composition {
        type F = Identity;

        fn apply(self, _operation: Self::F, _element_count: usize) -> Self {
            self
        }
    }

    fn assert_all_sums(tree: &mut SegTree<Sum>, values: &[i64]) {
        assert_eq!(tree.query(0, values.len() - 1), Sum(values.iter().sum()));

        for left in 0..values.len() {
            let mut expected_sum = 0;
            for (right, value) in values.iter().enumerate().skip(left) {
                expected_sum += value;
                assert_eq!(
                    tree.query(left, right),
                    Sum(expected_sum),
                    "incorrect range [{left}, {right}] for length {}",
                    values.len()
                );
            }
        }
    }

    fn update_reference(values: &mut [i64], left: usize, right: usize, operation: AffineUpdate) {
        for value in &mut values[left..=right] {
            *value = operation.multiplier * *value + operation.offset;
        }
    }

    #[test]
    fn new_trees_have_identity_ranges() {
        for length in [1, 2, 3, 7, 8, 9, 31, 32, 33] {
            let mut tree = SegTree::<Sum>::new(length);
            assert_all_sums(&mut tree, &vec![0; length]);
        }
    }

    #[test]
    fn build_matches_every_range() {
        for length in [1, 2, 3, 7, 8, 9, 31, 32, 33] {
            let values: Vec<i64> = (0..length).map(|index| index as i64 - 4).collect();
            let mut tree = SegTree::<Sum>::new(length);
            tree.build_from(&values.iter().copied().map(Sum).collect::<Vec<_>>());
            assert_all_sums(&mut tree, &values);
        }
    }

    #[test]
    fn singleton_supports_affine_updates() {
        let mut tree = SegTree::<Sum>::new(1);
        tree.build_from(&[Sum(5)]);

        for (multiplier, offset, expected_sum) in
            [(1, 3, 8), (2, -1, 15), (0, 0, 0), (1, -4, -4), (-1, 2, 6)]
        {
            tree.update(0, 0, AffineUpdate { multiplier, offset });
            assert_eq!(tree.query(0, 0), Sum(expected_sum));
        }
    }

    #[test]
    fn point_range_update_rebuilds_all_ancestors() {
        for length in [2, 3, 4, 8, 9, 32, 33] {
            for position in [0, length / 2, length - 1] {
                let mut tree = SegTree::<Sum>::new(length);
                let mut values = vec![0; length];
                let operation = AffineUpdate {
                    multiplier: 1,
                    offset: 1,
                };

                tree.update(position, position, operation);
                values[position] = 1;
                assert_all_sums(&mut tree, &values);
            }
        }
    }

    #[test]
    fn full_range_update_reaches_deep_leaf_queries() {
        for length in [8, 16, 32, 64] {
            let mut tree = SegTree::<Sum>::new(length);
            tree.update(
                0,
                length - 1,
                AffineUpdate {
                    multiplier: 1,
                    offset: 3,
                },
            );

            // Query a leaf before any other query can propagate the pending tags.
            assert_eq!(tree.query(0, 0), Sum(3));
            assert_all_sums(&mut tree, &vec![3; length]);
        }
    }

    #[test]
    fn overlapping_updates_preserve_composition_order() {
        let mut values = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let mut tree = SegTree::<Sum>::new(values.len());
        tree.build_from(&values.iter().copied().map(Sum).collect::<Vec<_>>());

        // Keep tags pending across assignments, additions, and multiplication.
        for (left, right, multiplier, offset) in [
            (0, 7, 1, 3),
            (0, 3, 0, 2),
            (0, 7, 2, 1),
            (2, 6, -1, 0),
            (3, 4, 0, 0),
            (1, 5, 1, -4),
        ] {
            let operation = AffineUpdate { multiplier, offset };
            tree.update(left, right, operation);
            update_reference(&mut values, left, right, operation);
        }

        assert_eq!(values, [5, 1, -9, -4, -4, -23, -21, 23]);
        assert_all_sums(&mut tree, &values);
        assert_all_sums(&mut tree, &values);
    }

    #[test]
    fn mixed_updates_match_a_reference_vector_after_batches() {
        let mut random_state = 42_u64;
        for length in (1..=33).chain([63, 64, 65]) {
            let mut values: Vec<i64> = (0..length).map(|index| index as i64 - 20).collect();
            let mut tree = SegTree::<Sum>::new(length);
            tree.build_from(&values.iter().copied().map(Sum).collect::<Vec<_>>());

            for batch in 0..8 {
                for _ in 0..16 {
                    random_state = random_state
                        .wrapping_mul(6_364_136_223_846_793_005)
                        .wrapping_add(1);
                    let first = (random_state >> 32) as usize % length;
                    random_state = random_state
                        .wrapping_mul(6_364_136_223_846_793_005)
                        .wrapping_add(1);
                    let second = (random_state >> 32) as usize % length;
                    let (left, right) = (first.min(second), first.max(second));
                    let operation = AffineUpdate {
                        multiplier: ((random_state >> 16) % 3) as i64 - 1,
                        offset: ((random_state >> 24) % 21) as i64 - 10,
                    };

                    tree.update(left, right, operation);
                    update_reference(&mut values, left, right, operation);
                }

                // Exercise identity updates without flushing all tags between updates.
                tree.update(0, length - 1, AffineUpdate::unit());
                assert_all_sums(&mut tree, &values);

                if batch == 3 {
                    values.reverse();
                    tree.build_from(&values.iter().copied().map(Sum).collect::<Vec<_>>());
                }
            }
        }
    }

    #[test]
    fn rebuilding_discards_pending_updates() {
        for length in [1, 3, 5, 8, 9, 16, 17] {
            let mut tree = SegTree::<Sum>::new(length);
            for (left, right) in [(0, length - 1), (length / 2, length - 1)] {
                tree.update(
                    left,
                    right,
                    AffineUpdate {
                        multiplier: 0,
                        offset: 100,
                    },
                );
            }

            let values: Vec<i64> = (0..length).map(|index| index as i64 - 3).collect();
            tree.build_from(&values.iter().copied().map(Sum).collect::<Vec<_>>());
            assert_all_sums(&mut tree, &values);

            let operation = AffineUpdate {
                multiplier: -1,
                offset: 2,
            };
            tree.update(0, length - 1, operation);
            let mut expected = values;
            update_reference(&mut expected, 0, length - 1, operation);
            assert_all_sums(&mut tree, &expected);
        }
    }

    #[test]
    fn noncommutative_aggregation_preserves_order() {
        for length in [5, 8, 9] {
            let values: Vec<Composition> = (0..length)
                .map(|index| Composition {
                    multiplier: index as u64 + 2,
                    offset: index as u64 * 3 + 1,
                })
                .collect();
            let mut tree = SegTree::<Composition>::new(length);
            tree.build_from(&values);
            assert_eq!(
                tree.query(0, 1),
                Composition {
                    multiplier: 6,
                    offset: 9,
                }
            );

            tree.update(1, length - 2, Identity);
            for left in 0..length {
                for right in left..length {
                    let expected = values[left..=right]
                        .iter()
                        .copied()
                        .fold(Composition::unit(), Composition::accumulate);
                    assert_eq!(tree.query(left, right), expected);
                }
            }
        }
    }

    #[test]
    #[should_panic]
    fn empty_constructor_is_rejected() {
        SegTree::<Sum>::new(0);
    }

    #[test]
    #[should_panic]
    fn build_with_too_few_values_is_rejected() {
        SegTree::<Sum>::new(2).build_from(&[Sum(1)]);
    }

    #[test]
    #[should_panic]
    fn build_with_too_many_values_is_rejected() {
        SegTree::<Sum>::new(2).build_from(&[Sum(1), Sum(2), Sum(3)]);
    }

    #[test]
    #[should_panic]
    fn reversed_update_is_rejected() {
        SegTree::<Sum>::new(3).update(2, 1, AffineUpdate::unit());
    }

    #[test]
    #[should_panic]
    fn update_outside_the_array_is_rejected() {
        SegTree::<Sum>::new(3).update(0, 3, AffineUpdate::unit());
    }

    #[test]
    #[should_panic]
    fn reversed_query_is_rejected() {
        SegTree::<Sum>::new(3).query(2, 1);
    }

    #[test]
    #[should_panic]
    fn query_outside_the_array_is_rejected() {
        SegTree::<Sum>::new(3).query(0, 3);
    }
}
