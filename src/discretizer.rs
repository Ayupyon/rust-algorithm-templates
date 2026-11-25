#![allow(dead_code)]

struct Discretizer<T: Ord + Clone> {
    arr: Vec<T>,
}

impl<T: Ord + Clone> Discretizer<T> {
    pub fn from(arr: &[T]) -> Self {
        let mut arr = arr.to_vec();
        arr.sort_unstable();
        arr.dedup();
        Self { arr }
    }

    pub fn len(&self) -> usize {
        self.arr.len()
    }

    pub fn get_index(&self, x: &T) -> Option<usize> {
        Some(self.arr.binary_search(x).ok()?)
    }

    pub fn get_element(&self, i: usize) -> &T {
        assert!(i < self.arr.len());
        &self.arr[i]
    }
}

#[cfg(test)]
mod tests {
    use super::Discretizer;

    #[test]
    fn empty_input_has_no_indices() {
        let discretizer = Discretizer::<i32>::from(&[]);

        assert_eq!(discretizer.len(), 0);
        assert_eq!(discretizer.get_index(&0), None);
    }

    #[test]
    fn unsorted_duplicates_map_to_sorted_unique_indices() {
        let input = [7, -3, 0, 7, i32::MAX, -3, i32::MIN, 0];
        let discretizer = Discretizer::from(&input);
        let expected = [i32::MIN, -3, 0, 7, i32::MAX];

        assert_eq!(discretizer.len(), expected.len());
        for (index, value) in expected.iter().enumerate() {
            assert_eq!(discretizer.get_index(value), Some(index));
            assert_eq!(discretizer.get_element(index), value);
        }
        for value in &input {
            let index = discretizer.get_index(value).unwrap();
            assert_eq!(discretizer.get_element(index), value);
        }
    }

    #[test]
    fn singleton_and_repeated_values_have_one_index() {
        for input in [&[42][..], &[42, 42, 42][..]] {
            let discretizer = Discretizer::from(input);

            assert_eq!(discretizer.len(), 1);
            assert_eq!(discretizer.get_index(&42), Some(0));
            assert_eq!(discretizer.get_element(0), &42);
        }
    }

    #[test]
    fn missing_values_before_between_and_after_elements_have_no_index() {
        let discretizer = Discretizer::from(&[-10, 0, 10]);

        for value in [-11, -5, 5, 11] {
            assert_eq!(discretizer.get_index(&value), None);
        }
    }

    #[test]
    fn owns_clones_of_non_copy_elements_without_changing_input() {
        let mut input = vec![
            String::from("pear"),
            String::from("apple"),
            String::from("pear"),
        ];
        let discretizer = Discretizer::from(&input);

        assert_eq!(input, ["pear", "apple", "pear"]);
        input[1].clear();
        drop(input);

        assert_eq!(discretizer.len(), 2);
        assert_eq!(discretizer.get_index(&String::from("apple")), Some(0));
        assert_eq!(discretizer.get_index(&String::from("pear")), Some(1));
        assert_eq!(discretizer.get_element(0), "apple");
        assert_eq!(discretizer.get_element(1), "pear");
    }

    #[test]
    #[should_panic]
    fn accessing_an_element_of_empty_input_panics() {
        Discretizer::<i32>::from(&[]).get_element(0);
    }

    #[test]
    #[should_panic]
    fn accessing_index_equal_to_unique_length_panics() {
        let discretizer = Discretizer::from(&[5, 5, 1]);

        discretizer.get_element(discretizer.len());
    }
}
