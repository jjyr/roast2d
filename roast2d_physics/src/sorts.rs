pub fn insertion_sort<T: Ord>(list: &mut [T]) {
    insertion_sort_by(list, |a, b| a < b)
}

pub fn insertion_sort_by_key<T, K: Ord, F: Fn(&T) -> K>(list: &mut [T], f: F) {
    insertion_sort_by(list, |a, b| f(a) < f(b))
}

pub fn insertion_sort_by<T, F: Fn(&T, &T) -> bool>(list: &mut [T], is_less: F) {
    for i in 1..list.len() {
        for j in (0..i).rev() {
            if is_less(&list[j], &list[j + 1]) {
                break;
            }
            list.swap(j, j + 1);
        }
    }
}

#[cfg(test)]
mod tests {

    use rand::{thread_rng, Rng};

    use crate::sorts::insertion_sort;

    #[test]
    fn test_sort() {
        let mut list = [2, 0, 2, 4, 8, 17];
        insertion_sort(&mut list);
        assert_eq!(list, [0, 2, 2, 4, 8, 17]);
    }

    #[test]
    fn test_rand_sort() {
        let mut rng = thread_rng();
        let list: [u8; 16] = rng.r#gen();
        let mut l1 = list;
        let mut l2 = list;
        insertion_sort(&mut l1);
        l2.sort();
        assert_eq!(l1, l2);
    }
}
