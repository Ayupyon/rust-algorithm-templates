#![allow(dead_code)]

trait TSpec: Copy {
    fn unit() -> Self;
    fn accumulate(self, other: Self) -> Self;
}

struct FenwickTree<T: TSpec> {
    tree: Vec<T>,
}

impl<T: TSpec> FenwickTree<T> {
    fn new(n: usize) -> Self {
        Self {
            tree: vec![T::unit(); n + 1],
        }
    }

    fn update(&mut self, mut i: usize, x: T) {
        let n = self.tree.len();
        i += 1;
        while i < n {
            self.tree[i] = self.tree[i].accumulate(x);
            i += 1 << i.trailing_zeros();
        }
    }

    fn query(&self, mut i: usize) -> T {
        let mut res = T::unit();
        i += 1;
        while i > 0 {
            res = res.accumulate(self.tree[i]);
            i ^= 1 << i.trailing_zeros();
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::{FenwickTree, TSpec};

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Sum(i64);

    impl TSpec for Sum {
        fn unit() -> Self {
            Self(0)
        }

        fn accumulate(self, other: Self) -> Self {
            Self(self.0 + other.0)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Xor(u64);

    impl TSpec for Xor {
        fn unit() -> Self {
            Self(0)
        }

        fn accumulate(self, other: Self) -> Self {
            Self(self.0 ^ other.0)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Minimum(i64);

    impl TSpec for Minimum {
        fn unit() -> Self {
            Self(i64::MAX)
        }

        fn accumulate(self, other: Self) -> Self {
            Self(self.0.min(other.0))
        }
    }

    fn assert_prefix_sums(tree: &FenwickTree<Sum>, values: &[i64]) {
        let mut expected_sum = 0;
        for (index, value) in values.iter().enumerate() {
            expected_sum += value;
            assert_eq!(
                tree.query(index),
                Sum(expected_sum),
                "incorrect prefix at index {index} for length {}",
                values.len()
            );
        }
    }

    #[test]
    fn new_trees_have_identity_prefixes() {
        for length in [0, 1, 2, 3, 7, 8, 9, 31, 32, 33] {
            let tree = FenwickTree::<Sum>::new(length);
            assert_prefix_sums(&tree, &vec![0; length]);
        }
    }

    #[test]
    fn singleton_accumulates_repeated_updates() {
        let mut tree = FenwickTree::<Sum>::new(1);
        for (delta, expected_sum) in [(7, 7), (-10, -3), (0, -3), (3, 0)] {
            tree.update(0, Sum(delta));
            assert_eq!(tree.query(0), Sum(expected_sum));
        }
    }

    #[test]
    fn first_and_last_updates_affect_only_containing_prefixes() {
        for length in [2, 3, 8, 9, 32, 33] {
            let mut tree = FenwickTree::<Sum>::new(length);
            let mut values = vec![0; length];

            tree.update(length - 1, Sum(11));
            values[length - 1] += 11;
            assert_prefix_sums(&tree, &values);

            tree.update(0, Sum(-4));
            values[0] -= 4;
            assert_prefix_sums(&tree, &values);
        }
    }

    #[test]
    fn mixed_updates_match_a_reference_vector() {
        let mut random_state = 42_u64;
        for length in (1..=65).chain([127, 128, 129]) {
            let mut tree = FenwickTree::<Sum>::new(length);
            let mut values = vec![0; length];

            for _ in 0..128 {
                random_state = random_state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                let index = (random_state >> 32) as usize % length;
                random_state = random_state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                let delta = ((random_state >> 32) % 101) as i64 - 50;

                tree.update(index, Sum(delta));
                values[index] += delta;
                assert_prefix_sums(&tree, &values);
            }
        }
    }

    #[test]
    fn xor_updates_match_reference_prefixes_and_cancel_when_repeated() {
        let values = [0b1010, 0b1100, 0b0011, 0, 0b1111];
        let mut tree = FenwickTree::<Xor>::new(values.len());
        for (index, value) in values.iter().copied().enumerate() {
            tree.update(index, Xor(value));
        }

        let mut expected_xor = 0;
        for (index, value) in values.iter().copied().enumerate() {
            expected_xor ^= value;
            assert_eq!(tree.query(index), Xor(expected_xor));
        }

        tree.update(2, Xor(values[2]));
        expected_xor = 0;
        for (index, value) in values.iter().copied().enumerate() {
            if index != 2 {
                expected_xor ^= value;
            }
            assert_eq!(tree.query(index), Xor(expected_xor));
        }
    }

    #[test]
    fn minimum_uses_its_identity_and_accumulates_updates() {
        let mut values = [9, 6, 8, 3, 7];
        let mut tree = FenwickTree::<Minimum>::new(values.len());
        for index in 0..values.len() {
            assert_eq!(tree.query(index), Minimum(i64::MAX));
        }

        for (index, value) in values.iter().copied().enumerate() {
            tree.update(index, Minimum(value));
        }

        for (index, value) in [(2, 12), (1, -2), (4, -5)] {
            tree.update(index, Minimum(value));
            values[index] = values[index].min(value);

            let mut expected_minimum = i64::MAX;
            for (prefix_index, value) in values.iter().copied().enumerate() {
                expected_minimum = expected_minimum.min(value);
                assert_eq!(tree.query(prefix_index), Minimum(expected_minimum));
            }
        }
    }
}
