#![allow(dead_code)]

trait FSpec: Copy {
    fn unit() -> Self;
    fn accumulate(self, other: Self) -> Self;
}

trait TSpec: FSpec {
    type F: FSpec;
    fn apply(self, op: Self::F) -> Self;
}

// Simple implementation of segment tree.
// Support single update and range query.
struct SegTree<T: TSpec> {
    vals: Vec<T>,
    n: usize,
    leaf_cnt: usize,
}

impl<T: TSpec> SegTree<T> {
    fn new(n: usize) -> Self {
        assert!(n > 0);
        let leaf_cnt = n.next_power_of_two();
        let vals = vec![T::unit(); leaf_cnt << 1];
        Self { vals, n, leaf_cnt }
    }

    fn build_from(&mut self, arr: &[T]) {
        assert!(arr.len() == self.n);
        self.vals[self.leaf_cnt..self.leaf_cnt + self.n].copy_from_slice(arr);
        for i in (1..self.leaf_cnt).rev() {
            self._merge(i);
        }
    }

    fn update(&mut self, p: usize, x: T::F) {
        assert!(p < self.n);
        let mut i = self.leaf_cnt + p;
        self.vals[i] = self.vals[i].apply(x);
        while i > 1 {
            i >>= 1;
            self._merge(i);
        }
    }

    fn query(&self, ql: usize, qr: usize) -> T {
        assert!(ql <= qr && qr < self.n);
        let (mut l, mut r) = (self.leaf_cnt + ql, self.leaf_cnt + qr + 1);
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
    fn _merge(&mut self, i: usize) {
        self.vals[i] = self.vals[i << 1].accumulate(self.vals[i << 1 | 1]);
    }
}

#[cfg(test)]
mod tests {
    use super::SegTree;
    use super::{FSpec, TSpec};

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
        type F = Self;

        fn apply(self, operation: Self) -> Self {
            Self(self.0 + operation.0)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Minimum(i64);

    impl FSpec for Minimum {
        fn unit() -> Self {
            Self(i64::MAX)
        }

        fn accumulate(self, other: Self) -> Self {
            Self(self.0.min(other.0))
        }
    }

    impl TSpec for Minimum {
        type F = Self;

        fn apply(self, operation: Self) -> Self {
            self.accumulate(operation)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Affine {
        multiplier: u64,
        offset: u64,
    }

    impl FSpec for Affine {
        fn unit() -> Self {
            Self {
                multiplier: 1,
                offset: 0,
            }
        }

        fn accumulate(self, other: Self) -> Self {
            // Compose self(other(x)) modulo 97 to keep repeated products bounded.
            Self {
                multiplier: self.multiplier * other.multiplier % 97,
                offset: (self.multiplier * other.offset + self.offset) % 97,
            }
        }
    }

    impl TSpec for Affine {
        type F = Self;

        fn apply(self, operation: Self) -> Self {
            self.accumulate(operation)
        }
    }

    fn assert_all_sums(tree: &SegTree<Sum>, values: &[i64]) {
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

    #[test]
    fn new_trees_have_identity_ranges() {
        for length in [1, 2, 3, 7, 8, 9, 31, 32, 33] {
            let tree = SegTree::<Sum>::new(length);
            assert_all_sums(&tree, &vec![0; length]);
        }
    }

    #[test]
    fn build_and_rebuild_replace_all_values() {
        for length in [1, 2, 3, 7, 8, 9, 31, 32, 33] {
            let mut tree = SegTree::<Sum>::new(length);
            let mut values: Vec<i64> = (0..length).map(|index| index as i64 - 4).collect();
            tree.build_from(&values.iter().copied().map(Sum).collect::<Vec<_>>());
            assert_all_sums(&tree, &values);

            tree.update(0, Sum(100));
            values.iter_mut().for_each(|value| *value = -*value - 3);
            tree.build_from(&values.iter().copied().map(Sum).collect::<Vec<_>>());
            assert_all_sums(&tree, &values);
        }
    }

    #[test]
    fn singleton_accumulates_repeated_updates() {
        let mut tree = SegTree::<Sum>::new(1);
        tree.build_from(&[Sum(5)]);

        for (delta, expected_sum) in [(3, 8), (-10, -2), (0, -2), (2, 0)] {
            tree.update(0, Sum(delta));
            assert_eq!(tree.query(0, 0), Sum(expected_sum));
        }
    }

    #[test]
    fn first_and_last_updates_affect_only_containing_ranges() {
        for length in [2, 3, 8, 9, 32, 33] {
            let mut tree = SegTree::<Sum>::new(length);
            let mut values = vec![0; length];

            tree.update(length - 1, Sum(11));
            values[length - 1] += 11;
            assert_all_sums(&tree, &values);

            tree.update(0, Sum(-4));
            values[0] -= 4;
            assert_all_sums(&tree, &values);
        }
    }

    #[test]
    fn mixed_updates_match_a_reference_vector_for_every_range() {
        let mut random_state = 42_u64;
        for length in (1..=33).chain([63, 64, 65]) {
            let mut values: Vec<i64> = (0..length).map(|index| index as i64 - 20).collect();
            let mut tree = SegTree::<Sum>::new(length);
            tree.build_from(&values.iter().copied().map(Sum).collect::<Vec<_>>());
            assert_all_sums(&tree, &values);

            for _ in 0..32 {
                random_state = random_state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                let position = (random_state >> 32) as usize % length;
                random_state = random_state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                let delta = ((random_state >> 32) % 101) as i64 - 50;

                values[position] += delta;
                tree.update(position, Sum(delta));
                assert_all_sums(&tree, &values);
            }
        }
    }

    #[test]
    fn noncommutative_ranges_preserve_order_after_updates() {
        for length in [5, 8, 9] {
            let mut values: Vec<Affine> = (0..length)
                .map(|index| Affine {
                    multiplier: index as u64 + 2,
                    offset: index as u64 * 3 + 1,
                })
                .collect();
            let mut tree = SegTree::<Affine>::new(length);
            tree.build_from(&values);
            assert_eq!(
                tree.query(0, 1),
                Affine {
                    multiplier: 6,
                    offset: 9,
                }
            );

            for position in [0, length / 2, length - 1] {
                for left in 0..length {
                    for right in left..length {
                        let expected = values[left..=right]
                            .iter()
                            .copied()
                            .fold(Affine::unit(), Affine::accumulate);
                        assert_eq!(tree.query(left, right), expected);
                    }
                }

                let operation = Affine {
                    multiplier: 3,
                    offset: 7,
                };
                tree.update(position, operation);
                values[position] = values[position].apply(operation);
            }

            for left in 0..length {
                for right in left..length {
                    let expected = values[left..=right]
                        .iter()
                        .copied()
                        .fold(Affine::unit(), Affine::accumulate);
                    assert_eq!(tree.query(left, right), expected);
                }
            }
        }
    }

    #[test]
    fn minimum_supports_a_nonzero_identity() {
        let mut values = [9, 6, 8, 3, 7];
        let mut tree = SegTree::<Minimum>::new(values.len());
        assert_eq!(tree.query(0, 4), Minimum(i64::MAX));
        tree.build_from(&values.map(Minimum));

        for (position, value) in [(2, 12), (1, -2), (4, -5)] {
            tree.update(position, Minimum(value));
            values[position] = values[position].min(value);

            for left in 0..values.len() {
                for right in left..values.len() {
                    assert_eq!(
                        tree.query(left, right),
                        Minimum(*values[left..=right].iter().min().unwrap())
                    );
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
    fn update_outside_the_array_is_rejected() {
        SegTree::<Sum>::new(3).update(3, Sum(1));
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
