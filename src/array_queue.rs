use arrayvec::ArrayVec;

pub trait ArrayVecExt<T, const N: usize> {
    fn insert_by_key<K: Ord>(&mut self, elem: T, key: impl FnMut(&T) -> K) -> Option<T>;
}

impl<T, const N: usize> ArrayVecExt<T, N> for ArrayVec<T, N> {
    fn insert_by_key<K: Ord>(&mut self, elem: T, mut key: impl FnMut(&T) -> K) -> Option<T> {
        let index = match self.binary_search_by_key(&key(&elem), key) {
            Ok(i) => i,
            Err(i) => i,
        };
        if index == N {
            Some(elem)
        } else {
            let poped = if self.len() == N { self.pop() } else { None };
            self.insert(index, elem);
            poped
        }
    }
}
