#![allow(dead_code)]

use std::{collections::HashMap, hash::Hash};

/// Each value may have at most `i32::MAX` live copies.
struct IndexedHeap<T: Copy + Ord + Hash> {
    buf: Vec<T>,
    cnt: Vec<i32>,
    pos: HashMap<T, usize>,
    n: usize,
}

impl<T: Copy + Ord + Hash> IndexedHeap<T> {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            cnt: Vec::new(),
            pos: HashMap::new(),
            n: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.n
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    pub fn push(&mut self, x: T) {
        self.n += 1;
        if let Some(&p) = self.pos.get(&x) {
            self.cnt[p] += 1;
            return;
        }
        let i = self.buf.len();
        self.buf.push(x);
        self.cnt.push(1);
        self.pos.insert(x, i);
        self.sift_up(i);
    }

    pub fn remove(&mut self, x: &T) -> bool {
        if let Some(&p) = self.pos.get(x) {
            self.remove_at(p);
            true
        } else {
            false
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            Some(self.remove_at(0))
        }
    }

    pub fn peek(&self) -> Option<&T> {
        self.buf.first()
    }

    fn remove_at(&mut self, i: usize) -> T {
        self.n -= 1;
        let res = self.buf[i];
        if self.cnt[i] == 1 {
            self.swap(i, self.buf.len() - 1);
            self.buf.pop();
            self.cnt.pop();
            self.pos.remove(&res);
            if i < self.buf.len() {
                self.fix(i);
            }
        } else {
            self.cnt[i] -= 1;
        }
        res
    }

    fn swap(&mut self, i: usize, j: usize) {
        self.buf.swap(i, j);
        self.cnt.swap(i, j);
        *self.pos.get_mut(&self.buf[i]).unwrap() = i;
        *self.pos.get_mut(&self.buf[j]).unwrap() = j;
    }

    fn sift_up(&mut self, mut i: usize) {
        while i > 0 {
            let p = (i - 1) >> 1;
            if self.buf[i] <= self.buf[p] {
                break;
            }
            self.swap(i, p);
            i = p;
        }
    }

    fn sift_down(&mut self, mut i: usize) {
        let n = self.buf.len();
        loop {
            let mut mx = i;
            for c in [(i << 1) + 1, (i << 1) + 2] {
                if c < n && self.buf[c] > self.buf[mx] {
                    mx = c;
                }
            }
            if mx == i {
                break;
            }
            self.swap(i, mx);
            i = mx;
        }
    }

    fn fix(&mut self, i: usize) {
        if i > 0 && self.buf[i] > self.buf[(i - 1) >> 1] {
            self.sift_up(i);
        } else {
            self.sift_down(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IndexedHeap;

    #[test]
    fn empty_heap_can_be_reused_after_removing_its_only_value() {
        let mut heap = IndexedHeap::new();
        assert!(heap.is_empty());
        assert_eq!(heap.len(), 0);
        assert_eq!(heap.peek(), None);
        assert_eq!(heap.pop(), None);
        assert!(!heap.remove(&7));

        heap.push(7);
        assert!(heap.remove(&7));
        assert!(heap.is_empty());
        assert_eq!(heap.peek(), None);

        heap.push(7);
        assert_eq!(heap.pop(), Some(7));
        assert_eq!(heap.pop(), None);
    }

    #[test]
    fn remove_and_pop_each_consume_one_copy() {
        let mut heap = IndexedHeap::new();
        for value in [4, -2, 9, 9, 9, 4] {
            heap.push(value);
        }

        assert_eq!(heap.len(), 6);
        assert!(!heap.is_empty());
        assert!(heap.remove(&9));
        assert_eq!(heap.len(), 5);
        assert_eq!(heap.pop(), Some(9));
        assert_eq!(heap.len(), 4);
        assert_eq!(heap.peek(), Some(&9));

        for value in [9, 4, 4, -2] {
            assert_eq!(heap.pop(), Some(value));
        }
        assert!(heap.is_empty());
        assert_eq!(heap.pop(), None);
    }

    #[test]
    fn arbitrary_removals_preserve_heap_order_and_value_positions() {
        let mut heap = IndexedHeap::new();
        for value in [100, 60, 90, 10, 50, 80, 85, 85] {
            heap.push(value);
        }

        assert!(heap.remove(&10));
        assert!(heap.remove(&85));
        assert!(heap.remove(&60));
        assert!(!heap.remove(&60));
        assert!(heap.remove(&100));
        assert_eq!(heap.len(), 4);

        for value in [90, 85, 80, 50] {
            assert_eq!(heap.pop(), Some(value));
        }
        assert_eq!(heap.pop(), None);
    }
}
