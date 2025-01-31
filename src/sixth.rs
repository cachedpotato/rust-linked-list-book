use std::{marker::PhantomData, ptr::NonNull};

//WHAT THE FUCK HAPPENED TO MY BEAUTIFUL CODE
//WHY GOD WHY IS IT GONE
pub struct LinkedList<T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    // The linked list here doesn't really "Store" a value of type T, but
    // Only NonNull type pointers.
    // PhantomData<T> is used here to make the compiler THINK we do in fact
    // have some data of type T when in fact we don't
    _phantom: PhantomData<T>,
}

type Link<T> = Option<NonNull<Node<T>>>;

struct Node<T> {
    front: Link<T>,
    back: Link<T>,
    elem: T,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self {
            front: None,
            back: None,
            len: 0,
            _phantom: PhantomData,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        unsafe {
            let new = NonNull::new_unchecked(Box::into_raw(Box::new(Node{
                front: None,
                back: None,
                elem,
            })));

            //if let clause to access the NonNull inside Option<T>
            if let Some(old_front) = self.front {
                // rewiring
                // use as_ptr(), not as_ref()/as_mut()
                // try not to use safe references when we already are
                // in the territory of unsafe rust with raw ptrs (it will only make things worse)
                (*new.as_ptr()).back = Some(old_front);
                (*old_front.as_ptr()).front = Some(new);
            } else {
                self.back = Some(new);
            }

            self.len += 1;
            self.front = Some(new);
        }
    }

    pub fn push_back(&mut self, elem: T) {
        unsafe {
            let new = NonNull::new_unchecked(Box::into_raw(Box::new(Node{
                front: None,
                back: None,
                elem,
            })));

            //if let clause to access the NonNull inside Option<T>
            if let Some(old_back) = self.back {
                // rewiring
                // use as_ptr(), not as_ref()/as_mut()
                // try not to use safe references when we already are
                // in the territory of unsafe rust with raw ptrs (it will only make things worse)
                (*new.as_ptr()).front = Some(old_back);
                (*old_back.as_ptr()).back = Some(new);
            } else {
                self.front = Some(new);
            }

            self.len += 1;
            self.back = Some(new);
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            self.front.map(|old_front| {
                // get hold of current head
                // Use Box<T> to move the value inside and Drop
                let front = Box::from_raw(old_front.as_ptr());
                let result = front.elem;

                //rewiring
                self.front = front.back;
                if let Some(new) = self.front {
                    (*new.as_ptr()).front = None;
                } else {
                    self.back = None;
                }
                self.len -= 1;
                result
            })

        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        unsafe {
            self.back.map(|old_back| {
                //get hold of current head
                let back = Box::from_raw(old_back.as_ptr());
                let result = back.elem;

                //rewiring
                self.back = back.front;
                if let Some(new) = self.back {
                    (*new.as_ptr()).back = None;
                } else {
                    self.front = None;
                }
                self.len -= 1;
                result
            })

        }
    }

    pub fn front(&self) -> Option<&T> {
        unsafe {
            self.front.map(|front| {
                &(*front.as_ptr()).elem
            })

            // We can use the ? operator for a simpler code like so:
            // Some(&(*self.front?.as_ptr()).elem)
            // However the ? operator forces an early return in case of None
            // Which can be risky especially when we're dealing with unsafe Rust
        }
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        unsafe {
            self.front.map(|front| {
                &mut (*front.as_ptr()).elem
            })
        }
    }

    pub fn back(&self) -> Option<&T> {
        self.back.map(|back| unsafe {
            &(*back.as_ptr()).elem
        })
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.back.map(|back| unsafe {
            &mut (*back.as_ptr()).elem
        })
    }
    
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            front: self.front,
            back: self.back,
            len: self.len,
            _phantom: PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            front: self.front,
            back: self.back,
            len: self.len,
            _phantom: PhantomData,
        }
    }

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            list: self
        }
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop_front() {}
    }
}

// ITERATORS
/*
 THIS IS A LONG ONE.
 So... we are creating 3 new Iterable types that stems from our LinkedList
 a) Iter<'a, T>, which takes a reference of our List (&'a LinkedList<T>)
 b) IterMut<'a, T>, which takes a mutable reference of our List (&'a mut LinkedList<T>)
 c) IntoIter<T>, which CONSUMES our list to create an iterator

 for each iterator type, we need to implement the following:
 i)     Iterator trait
 ii)    DoubleEndedIterator trait
 iii)   ExactSizeIterator trait
 iv)    IntoIterator trait

 the first three is self explanatory, but the fourth one has a bit of explaining
 IntoIterator is primarily used for for loops.
 for loops, when desugared, is just a huge match statement with loops.
 Said match statement will have to CONVERT our struct INTO and ITERATOR.
 Hence the name, IntoIterator.

 This means we're not really implementing IntoIterator for our Iterator types,
 as well, they're already iterators, but actually for the following:
 I)    IntoIterator for &'a LinkedList<T>      -> will use a REFERENCE of self         -> convert to Iter<'a, T>
 II)   IntoIterator for &'a mut LinkedList<T>  -> will use a MUTABLE REFERENCE of self -> convert to IterMut<'a, T>
 III)  IntoIterator for LinkedList<T>          -> will CONSUME self                    -> convert to IntoIter<T>
 ^ 
 Don't confuse IntoIter<T> the type with IntoIterator the trait!

 to convert them into respective iterator types, we will create methods for our LinkedList<T>
 1) pub fn iter(&self)          -> Iter<'_, T>
 2) pub fn iter_mut(&mut self)  -> IterMut<'_, T>
 3) pub fn into_iter(self)      -> IntoIter<T>
 */

// 1) Iter
pub struct Iter<'a, T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.len > 0 {
                self.front.map(|old_front| {
                    self.len -= 1;
                    self.front = (*old_front.as_ptr()).back;
                    &(*old_front.as_ptr()).elem
                })
            } else {
                None
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.len > 0 {
                self.back.map(|old_back| {
                    self.len -= 1;
                    self.back = (*old_back.as_ptr()).front;
                    &(*old_back.as_ptr()).elem
                })

            } else {
                None
            }
        }
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// 2) IterMut
pub struct IterMut<'a, T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _phantom: PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.len > 0 {
                self.front.map(|old_head| {
                    self.front = (*old_head.as_ptr()).back;
                    self.len -= 1;
                    &mut (*old_head.as_ptr()).elem
                })
            } else {
                None
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.len > 0 {
                self.back.map(|old_back| {
                    self.back = (*old_back.as_ptr()).front;
                    self.len -= 1;
                    &mut (*old_back.as_ptr()).elem
                })
            } else {
                None
            }
        }
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T> IntoIterator for &'a mut LinkedList<T> {
    type IntoIter = IterMut<'a, T>;
    type Item = &'a mut T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

// 3) IntoIter
// we're consuming our list here
// no need for front/back/elem/phantom split
// just get the fucking list
pub struct IntoIter<T> {
    list: LinkedList<T>
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.list.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.list.pop_back()
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.list.len
    }
}

impl<T> IntoIterator for LinkedList<T> {
    type IntoIter = IntoIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

#[cfg(test)]
mod test {
    use super::LinkedList;

    #[test]
    fn test_basic() {
        let mut list = LinkedList::new();
        // Try to break an empty list
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        // Try to break a one item list
        list.push_front(10);
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        // Mess around
        list.push_front(10);
        assert_eq!(list.len(), 1);
        list.push_front(20);
        assert_eq!(list.len(), 2);
        list.push_front(30);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(30));
        assert_eq!(list.len(), 2);
        list.push_front(40);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(40));
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop_front(), Some(20));
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_iter() {
        let mut list = LinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);

        //IterMut testing with for loops
        for mut_iter in &mut list {
            *mut_iter += 1;
        }
        assert_eq!(list.front(), Some(&2));

        //IntoIter testing
        let mut into_iter = list.into_iter();
        assert_eq!(into_iter.len(), 3);
        assert_eq!(into_iter.next(), Some(2));
        assert_eq!(into_iter.next(), Some(3));
        assert_eq!(into_iter.next(), Some(4));
        assert_eq!(into_iter.next(), None);

        //backwards iteration testing
        let mut list = LinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        
        let mut iter = list.iter();

        assert_eq!(iter.next_back(), Some(&3));
        assert_eq!(iter.next_back(), Some(&2));
        assert_eq!(iter.next_back(), Some(&1));
        assert_eq!(iter.next_back(), None);

    }
}