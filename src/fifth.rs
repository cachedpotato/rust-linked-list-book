use std::ptr;

pub struct List<T> {
    head: Link<T>,
    tail: *mut Node<T>,
}

type Link<T> = *mut Node<T>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> List<T> {
        List {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
        }
    }

    pub fn push(&mut self, elem: T) {
        unsafe {
            let new_tail = Box::into_raw(Box::new(Node {
                elem,
                next: ptr::null_mut(),
            }));

            if !self.tail.is_null() {
                (*self.tail).next = new_tail;
            }
            else {
                self.head = new_tail;
            }
            self.tail = new_tail;
        }

    }

    pub fn pop(&mut self) -> Option<T> {
        // previously...
        // type Link<T> = Option<Box<Node<T>>>;
        // self.head.take().map(|head| {
        //     let head = *head;
        //     self.head = head.next;
        //     if self.head.is_none() {self.tail = ptr::null_mut();}
        //     head.elem})
        // 
        unsafe {
            if self.head.is_null() {
                None
            } else {
                let head = Box::from_raw(self.head);
                self.head = head.next;

                if self.head.is_null() {
                    self.tail = ptr::null_mut();
                }
                Some(head.elem)
            }
        }
    }

    pub fn peek(&self) -> Option<&T> {
        unsafe {
            self.head.as_ref()
                .map(|node| {
                    &node.elem
                })
        }
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        unsafe {
            self.head.as_mut()
                .map(|mref| {
                    &mut mref.elem
                })
        }
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() {
        }
    }
}

pub struct IntoIter<T>(List<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            self.next
                .map(|node| {
                    self.next = node.next.as_ref();
                    &node.elem
                })
        }
    }
}

pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            self.next.take()
                .map(|mut_node| {
                    self.next = mut_node.next.as_mut();
                    &mut mut_node.elem
                })
        }
    }
}

impl<T> List<T> {
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }

    pub fn iter(&self) -> Iter<'_, T> {
        unsafe {
            Iter {
                next: self.head.as_ref(),
            }
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        unsafe {
            IterMut {
                next: self.head.as_mut(),
            }
        }
    }
}



#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();
        assert_eq!(list.pop(), None);

        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        list.push(4);
        list.push(5);

        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);

        list.push(6);
        assert_eq!(list.pop(), Some(6));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn iterators() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut into_iter_list = list.into_iter();
        assert_eq!(into_iter_list.next(), Some(1));
        assert_eq!(into_iter_list.next(), Some(2));
        assert_eq!(into_iter_list.next(), Some(3));
        assert_eq!(into_iter_list.next(), None);


        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter_list = list.iter();
        assert_eq!(iter_list.next(), Some(&1));
        assert_eq!(iter_list.next(), Some(&2));
        assert_eq!(iter_list.next(), Some(&3));
        assert_eq!(iter_list.next(), None);

        let iter_mut_list = list.iter_mut();
        for mref in iter_mut_list {
            *mref += 1;
        }
        assert_eq!(list.pop(), Some(2));
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn test_peek() {
        let mut list = List::new();

        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.peek(), Some(&1));
        list.pop();
        assert_eq!(list.peek(), Some(&2));
        list.peek_mut()
            .map(|elem| *elem = 42);
        assert_eq!(list.peek(), Some(&42));
        list.pop();
        assert_eq!(list.peek(), Some(&3));
        list.pop();
        assert_eq!(list.peek(), None);
    }
}
