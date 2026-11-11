#![allow(dead_code)]

use std::{
    collections::{BinaryHeap, HashMap},
    hash::Hash,
};

/// For each value, its live count and pending deletion count must each fit in `i32`.
struct LazyHeap<T: Copy + Ord + Hash> {
    heap: BinaryHeap<T>,
    cnt: HashMap<T, i32>,
    todo: HashMap<T, i32>,
    n: usize,
}

impl<T: Copy + Ord + Hash> LazyHeap<T> {
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            cnt: HashMap::new(),
            todo: HashMap::new(),
            n: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    pub fn len(&self) -> usize {
        self.n
    }

    pub fn push(&mut self, x: T) {
        self.n += 1;
        Self::map_add(&mut self.cnt, x);
        if !Self::map_remove(&mut self.todo, &x) {
            self.heap.push(x);
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        self.apply_remove();
        let res = self.heap.pop();
        if let Some(x) = res.clone() {
            Self::map_remove(&mut self.cnt, &x);
            self.n -= 1;
        }
        res
    }

    pub fn peek(&mut self) -> Option<&T> {
        self.apply_remove();
        self.heap.peek()
    }

    pub fn remove(&mut self, x: &T) -> bool {
        if !Self::map_remove(&mut self.cnt, x) {
            return false;
        }
        self.n -= 1;
        Self::map_add(&mut self.todo, *x);
        true
    }

    fn apply_remove(&mut self) {
        while let Some(x) = self.heap.peek()
            && Self::map_remove(&mut self.todo, x)
        {
            self.heap.pop();
        }
    }

    fn map_add(map: &mut HashMap<T, i32>, x: T) {
        *map.entry(x).or_default() += 1;
    }

    fn map_remove(map: &mut HashMap<T, i32>, x: &T) -> bool {
        if let Some(c) = map.get_mut(x) {
            *c -= 1;
            if *c == 0 {
                map.remove(x);
            }
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LazyHeap;

    #[test]
    fn empty_heap_can_be_reused_after_clearing_pending_deletions() {
        let mut heap = LazyHeap::new();
        assert!(heap.is_empty());
        assert_eq!(heap.len(), 0);
        assert_eq!(heap.peek(), None);
        assert_eq!(heap.pop(), None);
        assert!(!heap.remove(&7));

        heap.push(7);
        assert!(heap.remove(&7));
        assert!(heap.is_empty());
        assert_eq!(heap.len(), 0);
        assert_eq!(heap.peek(), None);

        heap.push(7);
        assert_eq!(heap.pop(), Some(7));
        assert_eq!(heap.pop(), None);
    }

    #[test]
    fn peek_keeps_the_remaining_copy_of_a_partially_removed_maximum() {
        let mut heap = LazyHeap::new();
        for value in [2, 9, 9] {
            heap.push(value);
        }

        assert!(heap.remove(&9));
        assert_eq!(heap.len(), 2);
        assert!(!heap.is_empty());
        assert_eq!(heap.peek(), Some(&9));
        assert_eq!(heap.peek(), Some(&9));
        assert_eq!(heap.len(), 2);
        assert_eq!(heap.pop(), Some(9));
        assert!(!heap.remove(&9));
        assert_eq!(heap.peek(), Some(&2));
    }

    #[test]
    fn pop_skips_deleted_values_as_they_reach_the_top() {
        let mut heap = LazyHeap::new();
        for value in [10, 10, 7, 4, -1] {
            heap.push(value);
        }

        assert!(heap.remove(&10));
        assert!(heap.remove(&10));
        assert!(!heap.remove(&10));
        assert!(heap.remove(&4));
        assert_eq!(heap.len(), 2);
        assert_eq!(heap.pop(), Some(7));
        assert_eq!(heap.pop(), Some(-1));
        assert!(heap.is_empty());
        assert_eq!(heap.pop(), None);
    }

    #[test]
    fn reinserting_a_value_restores_one_of_two_removed_copies() {
        let mut heap = LazyHeap::new();
        for value in [10, 3, 3] {
            heap.push(value);
        }

        assert!(heap.remove(&3));
        assert!(heap.remove(&3));
        heap.push(3);

        assert_eq!(heap.len(), 2);
        assert_eq!(heap.pop(), Some(10));
        assert_eq!(heap.pop(), Some(3));
        assert!(heap.is_empty());
        assert_eq!(heap.pop(), None);
    }
}
