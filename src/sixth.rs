use std::{fmt::Debug, hash::Hash, marker::PhantomData, mem, ptr::NonNull};

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

    //Cursor Creation
    pub fn cursor_mut(&mut self) -> CursorMut<'_, T> {
        CursorMut {
            list: self,
            cur: None,
            index: None,
        }
    }

    //useful methods
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
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

//MISC. (+ Drop cuz Drop is very important and definitely not misc.)
impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop_front() {}
    }
}

// 1. Default
// create a fallback default case of LinkedList, which we kinda..already..have?
impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

// 2. Clone
// Make a clone of self
impl<T: Clone> Clone for LinkedList<T> {
    fn clone(&self) -> Self {
        let mut new  = Self::new();
        for item in self {
            new.push_back(item.clone());
        }
        new
    }
}

// 3. Extend
//
impl<T> Extend<T> for LinkedList<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push_back(item);
        }
    }
}

// 4. FromIterator
// A reverse of IntoIterator, where we now convert an iterator back to LinkedList
// needed for collect()
impl<T> FromIterator<T> for LinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::new();
        list.extend(iter);
        list
    }
}

// 5. Debug
// Debug display option
impl<T: Debug> Debug for LinkedList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

// 6. PartialEq
// The items and the length should be the same
// The two lists need not necessarily be EXACT (as in they both point to the same point in memory)
impl<T: PartialEq> PartialEq for LinkedList<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other)
    }

    fn ne(&self, other: &Self) -> bool {
        self.len() != other.len() || self.iter().ne(other)
    }
}

// 7. Eq
// Partial Eq guarantees transitivity and symmetry
// Eq guarantees reflexivity on top of it
// Eq is a trait without method
impl<T: Eq> Eq for LinkedList<T> {}

// 8. PartialOrd
// for > < >= <=
impl <T: PartialOrd> PartialOrd for LinkedList<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.iter().partial_cmp(other)
    }
}

// 9. Ord
// for traits with total (linear) order where any of the two elements are comparable
impl <T: Ord> Ord for LinkedList<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.iter().cmp(other)
    }
}

// 10. Hash
// for, well, hashing
// make sure to hash everything that's in struct
impl<T: Hash> Hash for LinkedList<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        for item in self {
            item.hash(state);
        }
    }
}


// 11. Send and Sync
/* 
 raw pointers have two safeguards:
 First, it is invariant, which means subtyping on generic parameters is blocked.
 Second, it automatically opts out of Send and Sync
 This means we need to opt these back in.
 Reminder: these traits are UNSAFE!

 Send and Sync traits do not have methods
*/ 
unsafe impl<T: Send> Send for LinkedList<T> {}
unsafe impl<T: Sync> Sync for LinkedList<T> {}

unsafe impl<'a, T: Send> Send for Iter<'a, T> {}
unsafe impl<'a, T: Sync> Sync for Iter<'a, T> {}

unsafe impl<'a, T: Send> Send for IterMut<'a, T> {}
unsafe impl<'a, T: Sync> Sync for IterMut<'a, T> {}



// CURSOR
// cursors have "ghost" elements that contains None to indicate
// the start and end of the list.
// We can "walk over" this ghost elements to wrap over to the other side
pub struct CursorMut<'a, T> {
    cur: Link<T>,
    list: &'a mut LinkedList<T>,
    index: Option<usize>,
}

impl<'a, T> CursorMut<'a, T> {
    pub fn index(&self) -> Option<usize> {
        self.index
    }

    pub fn move_next(&mut self) {
        unsafe {
            if let Some(pos) = self.cur {
                //the cursor is on a real element
                self.cur = (*pos.as_ptr()).back;

                if self.cur.is_some() {
                    *self.index.as_mut().unwrap() += 1;
                } else {
                    //walked into ghost
                    self.index = None;
                }
            } else if !self.list.is_empty() {
                //we're at ghost element
                self.cur = self.list.front;
                self.index = Some(0);
            } else {
                //empty list
                //nothing to do
            }
        }
    }

    pub fn move_prev(&mut self) {
        if let Some(pos) = self.cur {
            unsafe {
                //move back
                self.cur = (*pos.as_ptr()).front;

                if self.cur.is_some() {
                    //real node
                    *self.index.as_mut().unwrap() -= 1;
                } else {
                    //we at ghost node
                    self.index = None
                }
            }
        } else if !self.list.is_empty() {
            //cursor at ghost node (front)
            //wrap back to end of list
            self.cur = self.list.back;
            self.index = Some(self.list.len - 1);
        } else {
            //empty list
            //no nothing
        }
    }

    // methods to look at elements around cursor
    // borrows a MUTABLE reference to self and the
    // returns should be tied to the borrow.
    // we cannot let users get multiple copies of a mutable reference
    // and use splice/remove/split APIs
    pub fn current(&mut self) -> Option<&mut T> {
        self.cur.map(|current| unsafe {
            &mut (*current.as_ptr()).elem
        })
    }

    pub fn peek_next(&mut self) -> Option<&mut T> {
        unsafe {
            let next = if let Some(cur) = self.cur {
                (*cur.as_ptr()).back
            } else {
                self.list.front
            };
            next.map(|node| &mut (*node.as_ptr()).elem)
        }
    }

    pub fn peek_prev(&mut self) -> Option<&mut T> {
        unsafe {
            let prev = if let Some(cur) = self.cur {
                (*cur.as_ptr()).front
            } else {
                self.list.back
            };
            prev.map(|node| &mut (*node.as_ptr()).elem)
        }
    }

    pub fn remove_current(&mut self) -> Option<T> {
        unsafe {
            if let Some(cur) = self.cur {
                let prev = (*cur.as_ptr()).front.take();
                let next = (*cur.as_ptr()).back.take();
                //get current node (dropped after function return)
                let current= Box::from_raw(cur.as_ptr());
                let ret= current.elem;

                if prev.is_none() && next.is_none() {
                    //list is of length one
                    self.list.clear();
                } else {
                    if prev.is_none() {
                        //prev is ghost node (front)
                        (*next.unwrap().as_ptr()).front = None;
                        self.list.front = next;
                    } else if next.is_none() {
                        //next is ghost node (back)
                        (*prev.unwrap().as_ptr()).back = None;
                        self.list.back = prev;
                    } else {
                        //general case
                        (*prev.unwrap().as_ptr()).back = next;
                        (*next.unwrap().as_ptr()).front = prev;
                    }
                    self.list.len -= 1;
                    self.cur = next;
                }
                Some(ret)
            } else {
                //at ghost node or empty list
                None
            }
        }
    }

    //HERE IT COMES
    pub fn split_before(&mut self) -> LinkedList<T> {
        if let Some(cur) = self.cur {
            unsafe {
                let old_len = self.list.len;
                let old_idx = self.index.unwrap();
                let prev = (*cur.as_ptr()).front;

                let new_len = old_len - old_idx;
                let new_idx = Some(0);
                let new_front = self.cur;

                let ret_len = old_len - new_len;
                let ret_front = self.list.front;
                let ret_back = prev;

                if let Some(prev_node) = prev {
                    //sever links
                    (*prev_node.as_ptr()).back = None;
                    (*cur.as_ptr()).front = None;
                }

                //update self
                self.list.len = new_len;
                self.index = new_idx;
                self.list.front = new_front;

                //return list
                LinkedList {
                    front: ret_front,
                    back: ret_back,
                    len: ret_len,
                    _phantom: PhantomData,
                }
            }
        } else {
            mem::replace(self.list, LinkedList::new())
        }
    }

    //Following the book
    pub fn split_after(&mut self) -> LinkedList<T> {
        if let Some(cur) = self.cur {
            unsafe {
                //get current data
                let old_len = self.list.len;
                let old_idx = self.index;
                let next = (*cur.as_ptr()).back;

                //what self will become
                let new_len = old_idx.unwrap() + 1;
                let new_back = self.cur;

                //what output will be
                let ret_len = old_len - new_len;
                let ret_front = next;
                let ret_back = self.list.back;

                // Sever link between curr and next
                if let Some(node) = next {
                    (*node.as_ptr()).front = None;
                    (*cur.as_ptr()).back = None;
                }

                //produce result
                self.list.len = new_len;
                self.list.back = new_back;

                //Return
                LinkedList {
                    front: ret_front,
                    back: ret_back,
                    len: ret_len,
                    _phantom: PhantomData,
                }
            }

        } else {
            //ghost node
            //replace list with empty one
            mem::replace(&mut self.list, LinkedList::new())
        }
    }

    //add list before the cursor
    pub fn splice_before(&mut self, mut input: LinkedList<T>) {
        unsafe {
            if input.is_empty() {

            } else if let Some(cur) = self.cur {
                let input_front = input.front.take().unwrap();
                let input_back = input.back.take().unwrap();
                if let Some(prev) = (*cur.as_ptr()).front {
                    (*prev.as_ptr()).back = Some(input_front);
                    (*cur.as_ptr()).front = Some(input_back);
                    (*input_front.as_ptr()).front = Some(prev);
                    (*input_back.as_ptr()).back = Some(cur);
                } else {
                    (*cur.as_ptr()).front = Some(input_back);
                    (*input_back.as_ptr()).back = Some(cur);
                    self.list.front = Some(input_front);
                }
                *self.index.as_mut().unwrap() += input.len;
            } else if let Some(back) = self.list.back {
                let input_front = input.front.take().unwrap();
                let input_back = input.back.take().unwrap();

                self.list.back = Some(input_back);
                (*input_front.as_ptr()).front = Some(back);
                (*back.as_ptr()).back = Some(input_front);
            } else {
                mem::swap(self.list, &mut input);
            }

            self.list.len += input.len;
            input.len = 0;
        }
    }

    pub fn splice_after(&mut self, mut input: LinkedList<T>) {
        unsafe {
            if input.is_empty() {

            } else if let Some(cur) = self.cur {
                let input_front = input.front.take().unwrap();
                let input_back = input.back.take().unwrap();
                if let Some(next) = (*cur.as_ptr()).back {
                    (*next.as_ptr()).front = Some(input_back);
                    (*cur.as_ptr()).back = Some(input_front);
                    (*input_front.as_ptr()).front = Some(cur);
                    (*input_back.as_ptr()).back = Some(next);
                } else {
                    (*cur.as_ptr()).back = Some(input_front);
                    (*input_front.as_ptr()).front = Some(cur);
                    self.list.back = Some(input_back);
                }
            } else if let Some(front) = self.list.front {
                let input_front = input.front.take().unwrap();
                let input_back = input.back.take().unwrap();

                self.list.front = Some(input_front);
                (*input_back.as_ptr()).back = Some(front);
                (*front.as_ptr()).front = Some(input_back);

                //index
                self.index = Some(input.len - 1);
            } else {
                mem::swap(self.list, &mut input);
            }

            self.list.len += input.len;
            input.len = 0;
        }
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

    fn generate_test() -> LinkedList<i32> {
        list_from(&[0, 1, 2, 3, 4, 5, 6])
    }

    fn list_from<T: Clone>(v: &[T]) -> LinkedList<T> {
        v.iter().map(|x| (*x).clone()).collect()
    }

    #[test]
    fn test_rev_iter() {
        let m = generate_test();
        for (i, elt) in m.iter().rev().enumerate() {
            assert_eq!(6 - i as i32, *elt);
        }
        let mut n = LinkedList::new();
        assert_eq!(n.iter().rev().next(), None);
        n.push_front(4);
        let mut it = n.iter().rev();
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next().unwrap(), &4);
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_mut_iter() {
        let mut m = generate_test();
        let mut len = m.len();
        for (i, elt) in m.iter_mut().enumerate() {
            assert_eq!(i as i32, *elt);
            len -= 1;
        }
        assert_eq!(len, 0);
        let mut n = LinkedList::new();
        assert!(n.iter_mut().next().is_none());
        n.push_front(4);
        n.push_back(5);
        let mut it = n.iter_mut();
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert!(it.next().is_some());
        assert!(it.next().is_some());
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_iterator_mut_double_end() {
        let mut n = LinkedList::new();
        assert!(n.iter_mut().next_back().is_none());
        n.push_front(4);
        n.push_front(5);
        n.push_front(6);
        let mut it = n.iter_mut();
        assert_eq!(it.size_hint(), (3, Some(3)));
        assert_eq!(*it.next().unwrap(), 6);
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert_eq!(*it.next_back().unwrap(), 4);
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(*it.next_back().unwrap(), 5);
        assert!(it.next_back().is_none());
        assert!(it.next().is_none());
    }

    #[test]
    fn test_eq() {
        let mut n: LinkedList<u8> = list_from(&[]);
        let mut m = list_from(&[]);
        assert!(n == m);
        n.push_front(1);
        assert!(n != m);
        m.push_back(1);
        assert!(n == m);

        let n = list_from(&[2, 3, 4]);
        let m = list_from(&[1, 2, 3]);
        assert!(n != m);
    }

    #[test]
    fn test_ord() {
        let n = list_from(&[]);
        let m = list_from(&[1, 2, 3]);
        assert!(n < m);
        assert!(m > n);
        assert!(n <= n);
        assert!(n >= n);
    }

    #[test]
    fn test_ord_nan() {
        let nan = 0.0f64 / 0.0;
        let n = list_from(&[nan]);
        let m = list_from(&[nan]);
        assert!(!(n < m));
        assert!(!(n > m));
        assert!(!(n <= m));
        assert!(!(n >= m));

        let n = list_from(&[nan]);
        let one = list_from(&[1.0f64]);
        assert!(!(n < one));
        assert!(!(n > one));
        assert!(!(n <= one));
        assert!(!(n >= one));

        let u = list_from(&[1.0f64, 2.0, nan]);
        let v = list_from(&[1.0f64, 2.0, 3.0]);
        assert!(!(u < v));
        assert!(!(u > v));
        assert!(!(u <= v));
        assert!(!(u >= v));

        let s = list_from(&[1.0f64, 2.0, 4.0, 2.0]);
        let t = list_from(&[1.0f64, 2.0, 3.0, 2.0]);
        assert!(!(s < t));
        assert!(s > one);
        assert!(!(s <= one));
        assert!(s >= one);
    }

    #[test]
    fn test_debug() {
        let list: LinkedList<i32> = (0..10).collect();
        assert_eq!(format!("{:?}", list), "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]");

        let list: LinkedList<&str> = vec!["just", "one", "test", "more"]
            .iter().copied()
            .collect();
        //the r#""# is there for strings that contiains double quotation marks
        assert_eq!(format!("{:?}", list), r#"["just", "one", "test", "more"]"#);
    }

    #[test]
    fn test_hashmap() {
        // Check that HashMap works with this as a key

        let list1: LinkedList<i32> = (0..10).collect();
        let list2: LinkedList<i32> = (1..11).collect();
        let mut map = std::collections::HashMap::new();

        assert_eq!(map.insert(list1.clone(), "list1"), None);
        assert_eq!(map.insert(list2.clone(), "list2"), None);

        assert_eq!(map.len(), 2);

        assert_eq!(map.get(&list1), Some(&"list1"));
        assert_eq!(map.get(&list2), Some(&"list2"));

        assert_eq!(map.remove(&list1), Some("list1"));
        assert_eq!(map.remove(&list2), Some("list2"));

        assert!(map.is_empty());
    }

    #[test]
    fn test_cursor_move_peek() {
        let mut m: LinkedList<u32> = LinkedList::new();
        m.extend([1, 2, 3, 4, 5, 6]);
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        assert_eq!(cursor.current(), Some(&mut 1));
        assert_eq!(cursor.peek_next(), Some(&mut 2));
        assert_eq!(cursor.peek_prev(), None);
        assert_eq!(cursor.index(), Some(0));
        cursor.move_prev();
        assert_eq!(cursor.current(), None);
        assert_eq!(cursor.peek_next(), Some(&mut 1));
        assert_eq!(cursor.peek_prev(), Some(&mut 6));
        assert_eq!(cursor.index(), None);
        cursor.move_next();
        cursor.move_next();
        assert_eq!(cursor.current(), Some(&mut 2));
        assert_eq!(cursor.peek_next(), Some(&mut 3));
        assert_eq!(cursor.peek_prev(), Some(&mut 1));
        assert_eq!(cursor.index(), Some(1));

        let mut cursor = m.cursor_mut();
        cursor.move_prev();
        assert_eq!(cursor.current(), Some(&mut 6));
        assert_eq!(cursor.peek_next(), None);
        assert_eq!(cursor.peek_prev(), Some(&mut 5));
        assert_eq!(cursor.index(), Some(5));
        cursor.move_next();
        assert_eq!(cursor.current(), None);
        assert_eq!(cursor.peek_next(), Some(&mut 1));
        assert_eq!(cursor.peek_prev(), Some(&mut 6));
        assert_eq!(cursor.index(), None);
        cursor.move_prev();
        cursor.move_prev();
        assert_eq!(cursor.current(), Some(&mut 5));
        assert_eq!(cursor.peek_next(), Some(&mut 6));
        assert_eq!(cursor.peek_prev(), Some(&mut 4));
        assert_eq!(cursor.index(), Some(4));
    }

    #[test]
    fn test_cursor_mut_insert() {
        let mut m: LinkedList<u32> = LinkedList::new();
        m.extend([1, 2, 3, 4, 5, 6]);
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.splice_before(Some(7).into_iter().collect());
        cursor.splice_after(Some(8).into_iter().collect());
        // check_links(&m);
        assert_eq!(m.iter().cloned().collect::<Vec<_>>(), &[7, 1, 8, 2, 3, 4, 5, 6]);
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.move_prev();
        cursor.splice_before(Some(9).into_iter().collect());
        cursor.splice_after(Some(10).into_iter().collect());
        check_links(&m);
        assert_eq!(m.iter().cloned().collect::<Vec<_>>(), &[10, 7, 1, 8, 2, 3, 4, 5, 6, 9]);
        
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.move_prev();
        assert_eq!(cursor.remove_current(), None);
        cursor.move_next();
        cursor.move_next();
        assert_eq!(cursor.remove_current(), Some(7));
        cursor.move_prev();
        cursor.move_prev();
        cursor.move_prev();
        assert_eq!(cursor.remove_current(), Some(9));
        cursor.move_next();
        assert_eq!(cursor.remove_current(), Some(10));
        check_links(&m);
        assert_eq!(m.iter().cloned().collect::<Vec<_>>(), &[1, 8, 2, 3, 4, 5, 6]);

        let mut m: LinkedList<u32> = LinkedList::new();
        m.extend([1, 8, 2, 3, 4, 5, 6]);
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        let mut p: LinkedList<u32> = LinkedList::new();
        p.extend([100, 101, 102, 103]);
        let mut q: LinkedList<u32> = LinkedList::new();
        q.extend([200, 201, 202, 203]);
        cursor.splice_after(p);
        cursor.splice_before(q);
        check_links(&m);
        assert_eq!(
            m.iter().cloned().collect::<Vec<_>>(),
            &[200, 201, 202, 203, 1, 100, 101, 102, 103, 8, 2, 3, 4, 5, 6]
        );
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.move_prev();
        let tmp = cursor.split_before();
        assert_eq!(m.into_iter().collect::<Vec<_>>(), &[]);
        m = tmp;
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        let tmp = cursor.split_after();
        assert_eq!(tmp.into_iter().collect::<Vec<_>>(), &[102, 103, 8, 2, 3, 4, 5, 6]);
        check_links(&m);
        assert_eq!(m.iter().cloned().collect::<Vec<_>>(), &[200, 201, 202, 203, 1, 100, 101]);
    }

    fn check_links<T: Eq + std::fmt::Debug>(list: &LinkedList<T>) {
        let from_front: Vec<_> = list.iter().collect();
        let from_back: Vec<_> = list.iter().rev().collect();
        let re_reved: Vec<_> = from_back.into_iter().rev().collect();
        assert_eq!(from_front, re_reved);
    }

}