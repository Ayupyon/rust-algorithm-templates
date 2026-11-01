#![allow(dead_code)]

trait FSpec: Copy {
    fn unit() -> Self;
    fn accumulate(self, other: Self) -> Self;
}

trait TSpec: FSpec {
    type F: FSpec;
    fn apply(self, op: Self::F, cnt: u64) -> Self;
}

struct Node<T: TSpec> {
    val: T,
    tag: T::F,
    children: Option<(usize, usize)>,
}

impl<T: TSpec> Default for Node<T> {
    fn default() -> Self {
        Self {
            val: T::unit(),
            tag: T::F::unit(),
            children: None,
        }
    }
}

enum Work {
    Visit { i: usize, l: u64, r: u64 },
    Merge { i: usize },
}

struct SegTree<T: TSpec> {
    nodes: Vec<Node<T>>,
    n: u64,
}

impl<T: TSpec> SegTree<T> {
    fn new(n: u64) -> Self {
        assert!(n > 0);
        let mut res = Self {
            nodes: Vec::new(),
            n,
        };
        res._alloc();
        res
    }

    fn update(&mut self, ql: u64, qr: u64, x: T::F) {
        assert!(ql <= qr && qr < self.n);
        let qr = qr + 1;
        let mut worklist = vec![Work::Visit {
            i: 0,
            l: 0,
            r: self.n,
        }];
        while let Some(item) = worklist.pop() {
            match item {
                Work::Visit { i, l, r } => {
                    if qr <= l || r <= ql {
                        continue;
                    }
                    if ql <= l && r <= qr {
                        self._apply(i, x, r - l);
                        continue;
                    }
                    let m = l + (r - l) / 2;
                    let (lc, rc) = self._push(i, l, r);
                    worklist.push(Work::Merge { i });
                    worklist.push(Work::Visit { i: rc, l: m, r });
                    worklist.push(Work::Visit { i: lc, l, r: m });
                }
                Work::Merge { i } => self._merge(i),
            }
        }
    }

    fn query(&mut self, ql: u64, qr: u64) -> T {
        assert!(ql <= qr && qr < self.n);
        let qr = qr + 1;
        let mut res = T::unit();
        let mut worklist = vec![(0, 0, self.n)]; // (i, l, r)
        while let Some((i, l, r)) = worklist.pop() {
            if qr <= l || r <= ql {
                continue;
            }
            if ql <= l && r <= qr {
                res = res.accumulate(self.nodes[i].val);
                continue;
            }
            let m = l + (r - l) / 2;
            let (lc, rc) = self._push(i, l, r);
            // Process left child before right child to preserve aggregation order.
            worklist.push((rc, m, r));
            worklist.push((lc, l, m));
        }
        res
    }

    #[inline]
    fn _alloc(&mut self) -> usize {
        let i = self.nodes.len();
        self.nodes.push(Node::default());
        i
    }

    #[inline]
    fn _children(&mut self, i: usize) -> (usize, usize) {
        if let Some(children) = self.nodes[i].children {
            return children;
        }
        let (lc, rc) = (self._alloc(), self._alloc());
        self.nodes[i].children = Some((lc, rc));
        (lc, rc)
    }

    #[inline]
    fn _apply(&mut self, i: usize, x: T::F, cnt: u64) {
        self.nodes[i].val = self.nodes[i].val.apply(x, cnt);
        if cnt > 1 {
            self.nodes[i].tag = self.nodes[i].tag.accumulate(x);
        }
    }

    #[inline]
    fn _push(&mut self, i: usize, l: u64, r: u64) -> (usize, usize) {
        let m = l + ((r - l) >> 1);
        let (lc, rc) = self._children(i);
        let x = self.nodes[i].tag;
        self.nodes[i].tag = T::F::unit();
        // In static version tree, we always use 2^n leaf nodes, so non-leaf nodes always hold even
        // range. But in dynamic version which cannot be ensured, so we should handle odd range nodes.
        self._apply(lc, x, m - l);
        self._apply(rc, x, r - m);
        (lc, rc)
    }

    #[inline]
    fn _merge(&mut self, i: usize) {
        let (lc, rc) = self.nodes[i]
            .children
            .expect("internal node must have children");
        self.nodes[i].val = self.nodes[lc].val.accumulate(self.nodes[rc].val);
    }
}

#[cfg(test)]
mod tests {
    use super::{FSpec, SegTree, TSpec};

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct AffineUpdate {
        multiplier: i128,
        offset: i128,
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
    struct Sum(i128);

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

        fn apply(self, operation: Self::F, element_count: u64) -> Self {
            // Fail promptly if traversal incorrectly descends below a leaf.
            assert!(element_count > 0);
            Self(operation.multiplier * self.0 + operation.offset * i128::from(element_count))
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

    #[derive(Clone, Copy)]
    struct Assign(Option<Composition>);

    impl FSpec for Assign {
        fn unit() -> Self {
            Self(None)
        }

        fn accumulate(self, newer: Self) -> Self {
            Self(newer.0.or(self.0))
        }
    }

    impl TSpec for Composition {
        type F = Assign;

        fn apply(self, operation: Self::F, element_count: u64) -> Self {
            assert!(element_count > 0);
            let Some(mut value) = operation.0 else {
                return self;
            };

            // A uniform assignment aggregates repeated copies of the function.
            let mut count = element_count;
            let mut result = Self::unit();
            while count > 0 {
                if count & 1 == 1 {
                    result = result.accumulate(value);
                }
                value = value.accumulate(value);
                count >>= 1;
            }
            result
        }
    }

    fn assert_all_sums(tree: &mut SegTree<Sum>, values: &[i128]) {
        assert_eq!(
            tree.query(0, values.len() as u64 - 1),
            Sum(values.iter().sum())
        );

        for left in 0..values.len() {
            let mut expected_sum = 0;
            for (right, value) in values.iter().enumerate().skip(left) {
                expected_sum += value;
                assert_eq!(
                    tree.query(left as u64, right as u64),
                    Sum(expected_sum),
                    "incorrect range [{left}, {right}] for length {}",
                    values.len()
                );
            }
        }
    }

    fn update_reference(values: &mut [i128], left: usize, right: usize, operation: AffineUpdate) {
        for value in &mut values[left..=right] {
            *value = operation.multiplier * *value + operation.offset;
        }
    }

    #[test]
    fn new_trees_have_identity_ranges() {
        for length in [1, 2, 3, 7, 8, 9, 31, 32, 33] {
            let mut tree = SegTree::<Sum>::new(length as u64);
            assert_eq!(tree.nodes.len(), 1);
            assert_all_sums(&mut tree, &vec![0; length]);
        }
    }

    #[test]
    fn singleton_queries_terminate_and_return_updated_values() {
        let mut tree = SegTree::<Sum>::new(1);
        assert_eq!(tree.query(0, 0), Sum(0));

        for (multiplier, offset, expected_sum) in
            [(1, 3, 3), (2, -1, 5), (0, 0, 0), (1, -4, -4), (-1, 2, 6)]
        {
            tree.update(0, 0, AffineUpdate { multiplier, offset });
            assert_eq!(tree.query(0, 0), Sum(expected_sum));
        }
        assert_eq!(tree.nodes.len(), 1);
    }

    #[test]
    fn full_range_operations_do_not_allocate_children() {
        let length = 1_000_000_000_000_000_000;
        let mut tree = SegTree::<Sum>::new(length);
        let mut expected_value = 0;

        for (multiplier, offset) in [(1, 3), (2, 1), (0, 4), (-1, 2)] {
            tree.update(0, length - 1, AffineUpdate { multiplier, offset });
            expected_value = multiplier * expected_value + offset;
            assert_eq!(
                tree.query(0, length - 1),
                Sum(expected_value * i128::from(length))
            );
            assert_eq!(tree.nodes.len(), 1);
        }
    }

    #[test]
    fn point_updates_change_only_the_selected_element() {
        for length in [2, 3, 7, 8, 9, 32, 33] {
            for position in [0, length / 2, length - 1] {
                let mut tree = SegTree::<Sum>::new(length as u64);
                let mut values = vec![0; length];
                tree.update(
                    position as u64,
                    position as u64,
                    AffineUpdate {
                        multiplier: 1,
                        offset: 5,
                    },
                );
                values[position] = 5;
                assert_all_sums(&mut tree, &values);
            }
        }
    }

    #[test]
    fn partial_updates_push_tags_through_unequal_children() {
        for length in [3, 5, 7, 9, 17, 65] {
            let mut tree = SegTree::<Sum>::new(length as u64);
            let mut values = vec![0; length];

            for (left, right, multiplier, offset) in [
                (0, length - 1, 0, 5),
                (0, 0, 1, -3),
                (length / 2, length - 1, -1, 2),
                (length - 1, length - 1, 0, 0),
            ] {
                let operation = AffineUpdate { multiplier, offset };
                tree.update(left as u64, right as u64, operation);
                update_reference(&mut values, left, right, operation);
            }

            assert_all_sums(&mut tree, &values);
        }
    }

    #[test]
    fn overlapping_updates_preserve_composition_order() {
        let mut tree = SegTree::<Sum>::new(8);
        let mut values = vec![0; 8];

        for (left, right, multiplier, offset) in [
            (0, 7, 1, 3),
            (0, 3, 0, 2),
            (0, 7, 2, 1),
            (2, 6, -1, 0),
            (3, 4, 0, 0),
            (1, 5, 1, -4),
        ] {
            let operation = AffineUpdate { multiplier, offset };
            tree.update(left as u64, right as u64, operation);
            update_reference(&mut values, left, right, operation);
        }

        assert_eq!(values, [5, 1, -9, -4, -4, -11, -7, 7]);
        assert_all_sums(&mut tree, &values);
        assert_all_sums(&mut tree, &values);
    }

    #[test]
    fn mixed_updates_match_a_reference_vector_after_batches() {
        let mut random_state = 42_u64;
        for length in (1..=33).chain([63, 64, 65]) {
            let mut values = vec![0; length];
            let mut tree = SegTree::<Sum>::new(length as u64);

            for _ in 0..8 {
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
                        multiplier: i128::from((random_state >> 16) % 3) - 1,
                        offset: i128::from((random_state >> 24) % 21) - 10,
                    };

                    tree.update(left as u64, right as u64, operation);
                    update_reference(&mut values, left, right, operation);
                }

                tree.update(0, length as u64 - 1, AffineUpdate::unit());
                assert_all_sums(&mut tree, &values);
            }
        }
    }

    #[test]
    fn sparse_point_updates_on_a_huge_domain_use_bounded_storage() {
        let length = 1_u64 << 60;
        let mut tree = SegTree::<Sum>::new(length);

        for (position, value) in [(0, 5), (length / 2, -2), (length - 1, 11)] {
            tree.update(
                position,
                position,
                AffineUpdate {
                    multiplier: 1,
                    offset: value,
                },
            );
            assert_eq!(tree.query(position, position), Sum(value));
        }

        // Each point update creates at most two children per level.
        assert!(tree.nodes.len() <= 1 + 2 * 60 * 3);
        assert_eq!(tree.query(0, length - 1), Sum(14));
        assert_eq!(tree.query(1, length / 2 - 1), Sum(0));
        assert_eq!(tree.query(length / 2 + 1, length - 2), Sum(0));
    }

    #[test]
    fn maximum_domain_length_handles_right_endpoint_without_overflow() {
        let length = u64::MAX;
        let mut tree = SegTree::<Sum>::new(length);
        tree.update(
            0,
            length - 1,
            AffineUpdate {
                multiplier: 1,
                offset: 3,
            },
        );
        tree.update(
            length - 2,
            length - 1,
            AffineUpdate {
                multiplier: 1,
                offset: 2,
            },
        );

        assert_eq!(tree.query(0, length - 1), Sum(i128::from(length) * 3 + 4));
        assert_eq!(tree.query(length - 1, length - 1), Sum(5));
        assert_eq!(tree.query(length - 2, length - 1), Sum(10));
        assert_eq!(tree.query(0, 0), Sum(3));

        tree.update(
            length - 1,
            length - 1,
            AffineUpdate {
                multiplier: 0,
                offset: 0,
            },
        );
        assert_eq!(tree.query(0, length - 1), Sum(i128::from(length) * 3 - 1));
    }

    #[test]
    fn repeated_queries_reuse_existing_nodes() {
        let mut tree = SegTree::<Sum>::new(1_000_000_000_000_000_000);
        tree.update(
            0,
            tree.n - 1,
            AffineUpdate {
                multiplier: 1,
                offset: 7,
            },
        );

        assert_eq!(tree.query(100, 200), Sum(707));
        let allocated_nodes = tree.nodes.len();
        for _ in 0..8 {
            assert_eq!(tree.query(100, 200), Sum(707));
            assert_eq!(tree.nodes.len(), allocated_nodes);
        }
    }

    #[test]
    fn noncommutative_aggregation_preserves_order_after_assignments() {
        for length in [5, 8, 9] {
            let mut values: Vec<Composition> = (0..length)
                .map(|index| Composition {
                    multiplier: index as u64 + 2,
                    offset: index as u64 * 3 + 1,
                })
                .collect();
            let mut tree = SegTree::<Composition>::new(length as u64);
            for (position, value) in values.iter().copied().enumerate() {
                tree.update(position as u64, position as u64, Assign(Some(value)));
            }
            assert_eq!(
                tree.query(0, 1),
                Composition {
                    multiplier: 6,
                    offset: 9,
                }
            );

            for (left, right, operation) in [
                (0, length - 1, Assign::unit()),
                (
                    1,
                    length - 2,
                    Assign(Some(Composition {
                        multiplier: 3,
                        offset: 7,
                    })),
                ),
                (0, length - 1, Assign(Some(Composition::unit()))),
                (
                    length / 2,
                    length / 2,
                    Assign(Some(Composition {
                        multiplier: 5,
                        offset: 2,
                    })),
                ),
            ] {
                tree.update(left as u64, right as u64, operation);
                if let Some(value) = operation.0 {
                    values[left..=right].fill(value);
                }

                for left in 0..length {
                    for right in left..length {
                        let expected = values[left..=right]
                            .iter()
                            .copied()
                            .fold(Composition::unit(), Composition::accumulate);
                        assert_eq!(tree.query(left as u64, right as u64), expected);
                    }
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
