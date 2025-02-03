use crate::sixth::LinkedList;

pub struct LRUCache<T> {
    capacity: usize,
    cache: LinkedList<(usize, T)>,
}

impl<T: Clone> LRUCache<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: LinkedList::new(),
        }
    }

    pub fn get(&mut self, key: usize) -> Option<&T> {
        let mut m = self.cache.cursor_mut();
        m.move_next();
        while m.current().is_some() {
            if m.current().unwrap().0 != key {m.move_next(); continue;}
            let res= m.current().unwrap().clone();
            m.remove_current();
            m.move_to_back();
            m.splice_after(LinkedList::from_iter([res]));

            return self.cache.back().map(|(_k, v)| v)
        }
        None
    }

    pub fn put(&mut self, new: (usize, T)) {
        if self.cache.len() == self.capacity {
            self.cache.pop_front();
        }
        self.cache.push_back(new);
        self.capacity += 1;
    }
}
