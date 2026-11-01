#![allow(dead_code)]

struct Lca {
    ancestor: Vec<Vec<usize>>,
    depth: Vec<i32>,
}

impl Lca {
    fn new(ancestor: Vec<Vec<usize>>, depth: Vec<i32>) -> Self {
        Self { ancestor, depth }
    }

    fn build(&mut self) {
        for i in 1..self.ancestor.len() {
            for u in 0..self.ancestor[0].len() {
                let v = self.ancestor[i - 1][u];
                if v != usize::MAX {
                    self.ancestor[i][u] = self.ancestor[i - 1][v];
                }
            }
        }
    }

    fn get_kth_ancestor(&self, mut u: usize, mut k: i32) -> usize {
        assert!(k >= 0 && k <= self.depth[u]);
        while k > 0 {
            let t = k.trailing_zeros() as usize;
            u = self.ancestor[t][u];
            k ^= 1 << t;
        }
        u
    }

    fn query(&self, mut u: usize, mut v: usize) -> usize {
        if self.depth[u] > self.depth[v] {
            (u, v) = (v, u);
        }
        v = self.get_kth_ancestor(v, self.depth[v] - self.depth[u]);
        if u == v {
            return u;
        }

        for k in (0..self.ancestor.len()).rev() {
            let (pu, pv) = (self.ancestor[k][u], self.ancestor[k][v]);
            if pu != pv {
                (u, v) = (pu, pv);
            }
        }
        self.ancestor[0][u]
    }
}

#[cfg(test)]
mod tests {
    use super::Lca;

    fn ancestor_path(parents: &[usize], mut node: usize) -> Vec<usize> {
        let mut path = vec![node];
        while parents[node] != usize::MAX && parents[node] != node {
            node = parents[node];
            path.push(node);
        }
        path
    }

    fn build_tree(parents: &[usize]) -> Lca {
        let level_count = (usize::BITS - parents.len().leading_zeros()) as usize;
        let mut ancestor = vec![vec![usize::MAX; parents.len()]; level_count];
        ancestor[0].copy_from_slice(parents);
        let depth = (0..parents.len())
            .map(|node| (ancestor_path(parents, node).len() - 1) as i32)
            .collect();
        let mut tree = Lca::new(ancestor, depth);
        tree.build();
        tree
    }

    fn assert_matches_parent_walks(parents: &[usize]) {
        let tree = build_tree(parents);
        let paths: Vec<_> = (0..parents.len())
            .map(|node| ancestor_path(parents, node))
            .collect();

        for (node, path) in paths.iter().enumerate() {
            for (distance, &expected_ancestor) in path.iter().enumerate() {
                assert_eq!(
                    tree.get_kth_ancestor(node, distance as i32),
                    expected_ancestor,
                    "incorrect ancestor of node {node} at distance {distance}"
                );
            }
            for (other_node, other_path) in paths.iter().enumerate() {
                let expected_lca = path
                    .iter()
                    .copied()
                    .find(|ancestor| other_path.contains(ancestor))
                    .unwrap();
                assert_eq!(
                    tree.query(node, other_node),
                    expected_lca,
                    "incorrect LCA of nodes {node} and {other_node} with parents {parents:?}"
                );
            }
        }
    }

    #[test]
    fn build_computes_power_of_two_ancestors_and_preserves_missing_ancestors() {
        let missing = usize::MAX;
        let mut tree = Lca::new(
            vec![
                vec![missing, 0, 1, 2, 3, 1],
                vec![missing; 6],
                vec![missing; 6],
                vec![missing; 6],
            ],
            vec![0, 1, 2, 3, 4, 2],
        );
        let expected = vec![
            vec![missing, 0, 1, 2, 3, 1],
            vec![missing, missing, 0, 1, 2, 0],
            vec![missing, missing, missing, missing, 0, missing],
            vec![missing; 6],
        ];

        tree.build();
        assert_eq!(tree.ancestor, expected);

        tree.build();
        assert_eq!(tree.ancestor, expected);
    }

    #[test]
    fn singleton_supports_both_root_parent_conventions() {
        for root_parent in [usize::MAX, 0] {
            assert_matches_parent_walks(&[root_parent]);
        }
    }

    #[test]
    fn balanced_tree_queries_cover_siblings_cousins_and_ancestors() {
        for root_parent in [usize::MAX, 0] {
            assert_matches_parent_walks(&[root_parent, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4]);
        }
    }

    #[test]
    fn star_queries_return_the_root_for_distinct_leaves() {
        for root_parent in [usize::MAX, 0] {
            let mut parents = vec![0; 33];
            parents[0] = root_parent;
            assert_matches_parent_walks(&parents);
        }
    }

    #[test]
    fn chain_queries_cover_power_of_two_boundaries_and_mixed_bit_distances() {
        for length in [2, 3, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 65] {
            for root_parent in [usize::MAX, 0] {
                let mut parents = vec![root_parent];
                parents.extend(0..length - 1);
                assert_matches_parent_walks(&parents);
            }
        }
    }

    #[test]
    fn queries_support_a_nonzero_root_and_arbitrary_node_order() {
        for root_parent in [usize::MAX, 4] {
            assert_matches_parent_walks(&[3, 4, 5, 1, root_parent, 1, 3, 2]);
        }
    }

    #[test]
    fn generated_trees_match_parent_walks_for_every_pair() {
        let mut random_state = 42_u64;
        for length in (2..=65).chain([127, 128, 129]) {
            let mut parents = vec![usize::MAX];
            for node in 1..length {
                random_state = random_state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                parents.push((random_state >> 32) as usize % node);
            }
            assert_matches_parent_walks(&parents);

            parents[0] = 0;
            assert_matches_parent_walks(&parents);
        }
    }
}
