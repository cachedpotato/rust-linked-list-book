use std::fmt::Debug;

use crate::sixth::LinkedList;

pub struct LRUCache<T> {
    capacity: usize,
    cache: LinkedList<(usize, T)>,
}

impl<T> LRUCache<T> {
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: LinkedList::new(),
        }
    }

    pub fn get(&mut self, key: usize) -> Option<&T> {
        let mut m = self.cache.cursor_mut();
        m.move_next();
        while let Some((k, _v)) = m.current() {
            if k != &key {m.move_next(); continue;}

            let a = m.remove_current(); //<- YOU FUCKING PIECE OF SHIT
            self.cache.push_back(a.unwrap());
            return self.cache.back()
                .map(|(_k, v)| v);
        }
        None
        //m.remove_current();
    }

    pub fn put(&mut self, new: (usize, T)) {
        if self.cache.len() == self.capacity {
            self.cache.pop_front();
        }
        self.cache.push_back(new);
    }
}

impl<T> Drop for LRUCache<T> {
    fn drop(&mut self) {
        self.cache.clear();
        self.capacity = 0;
    }
}

#[cfg(test)]
mod test {
    use super::LRUCache;
    #[test]
    fn basics() {
        let mut lru: LRUCache<i32> = LRUCache::new(2);
        assert_eq!(lru.get(1), None);
        lru.put((1, 10));
        assert_eq!(lru.len(), 1);
        assert_eq!(lru.get(1), Some(&10));
        lru.put((2, 20));
        lru.put((3, 30));
        assert_eq!(lru.get(2), Some(&20));
        assert_eq!(lru.get(1), None);
    }
}