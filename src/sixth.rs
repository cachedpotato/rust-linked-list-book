use std::{fmt::Debug, hash::Hash, marker::PhantomData, mem, ops::Add, ptr::NonNull};

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
    pub fn cursor(&self) -> Cursor<'_, T> {
        Cursor {
            cur: None,
            list: self,
            index: None
        }
    }


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

//IMMUTABLE CURSOR
pub struct Cursor<'a, T> {
    cur: Link<T>,
    list: &'a LinkedList<T>,
    index: Option<usize>,
}

impl<'a, T> Cursor<'a, T> {
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

    pub fn current(&self) -> Option<&T> {
        self.cur.map(|current| unsafe {
            &(*current.as_ptr()).elem
        })
    }

    pub fn peek_next(&self) -> Option<&T> {
        unsafe {
            let next = if let Some(cur) = self.cur {
                (*cur.as_ptr()).back
            } else {
                self.list.front
            };
            next.map(|node| &(*node.as_ptr()).elem)
        }
    }

    pub fn peek_prev(&self) -> Option<&T> {
        unsafe {
            let prev = if let Some(cur) = self.cur {
                (*cur.as_ptr()).front
            } else {
                self.list.back
            };
            prev.map(|node| &(*node.as_ptr()).elem)
        }
    }
}


//MUTABLE CURSOR
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

    pub fn move_to_front(&mut self) {
        if self.cur.is_none() {

        } else {
            self.cur = self.list.front;
            self.index = Some(0);
        }
    }

    pub fn move_to_back(&mut self) {
        if self.cur.is_none() {

        } else {
            self.cur = self.list.back;
            self.index = Some(self.list.len - 1);
        }
    }

    //holy shit there's a lot of edge cases I need to consider
    //TODO: Clean this mess
    pub fn remove_current(&mut self) -> Option<T> {
        unsafe {
            //not really clean but I'm just gonna use what works
            if self.list.is_empty() {
                None
            } else if self.list.len == 1 {
                self.cur = None;
                self.index = None;
                self.list.pop_front()
            } else if self.cur == self.list.front {
                //at front
                //move cursor to back
                self.cur = (*self.list.front.unwrap().as_ptr()).back;
                self.list.pop_front()

            } else if self.cur == self.list.back {
                //at back
                self.cur = None; //ghost node
                self.index = None;
                self.list.pop_back()
            } else {
                self.cur
                    .map(|node| {
                        //general case
                        //get hold of current
                        let current = Box::from_raw(node.as_ptr());
                        let prev = current.front.unwrap();
                        let next = current.back.unwrap();

                        //rewire
                        (*prev.as_ptr()).back = Some(next);
                        (*next.as_ptr()).front = Some(prev);

                        //update list and cursor
                        //index does not change
                        // 0 1 2 3 4 5
                        //     ^HERE (idx = 2)
                        // 0 1 3 4 5
                        //     ^HERE (idx = 2)
                        self.cur = Some(next);
                        self.list.len -= 1;
                        current.elem
                    })
                    .or_else(|| {
                        //empty list or ghost node
                        None
                    })
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


/*                      NEETCODE QUESTIONS                      
Since Neetcode questions require direct access to Node<T>, It'd be better to just
solve questions (as in impl functions) here than just make everything public.
Frankly I don't really care anymore I now join the dark side of hating linked lists.
I hate linked lists.

"Oh but why not use the stdlib version instead?"
DOESN'T MATTER IF THE CODE IS ALMOST A DIRECT COPY OF THE BOOK
I WROTE THIS
I WILL USE THIS

The actual question crates will serve as a testing ground to see if the functions
work without any memory leak/errors.
*/

//impl blocks per question for convenience
impl<T> LinkedList<T> {
    //question 001: reverse list
    pub fn reverse_list(list: LinkedList<T>) -> LinkedList<T>{
        LinkedList::from_iter(list.into_iter().rev().collect::<Vec<T>>())
    }

    //Yeah that one is a cop out
    pub fn reverse(&mut self) {
        unsafe {
            let mut curr = self.front;
            while let Some(node) = curr {
                let new_front = (*node.as_ptr()).back.take();
                let new_back = (*node.as_ptr()).front.take();
                (*node.as_ptr()).front = new_front;
                (*node.as_ptr()).back = new_back;
                curr = (*node.as_ptr()).front;
            }
        }

        //swap front and back pointers
        let new_front = self.back.take();
        let new_back = self.front.take();
        self.front = new_front;
        self.back = new_back;
    }
}

impl<T: PartialOrd + PartialEq> LinkedList<T> {
    pub fn merge_two_lists(mut list1: LinkedList<T>, mut list2: LinkedList<T>) -> LinkedList<T> {
        let mut out = LinkedList::new();

        while list1.front.is_some() && list2.front.is_some() {
            unsafe {
                if (*list1.front.unwrap().as_ptr()).elem <= (*list2.front.unwrap().as_ptr()).elem {
                    out.push_back(list1.pop_front().unwrap());
                } else {
                    out.push_back(list2.pop_front().unwrap());
                }
            }
        }

        //append whatever's left to the end
        out.extend(list1);
        out.extend(list2);

        out
    }
}

//Let's pretend we can make cyclic lists here
impl<T> LinkedList<T> {
    pub fn has_cycle(&mut self) -> bool {
        let mut set = std::collections::HashSet::new();
        //I can only have one cursor sooooo....
        //TODO: IMPLEMENT IMMUTABLE CURSOR
        let mut mcur = self.cursor_mut();
        mcur.move_next();
        while let Some(idx) = mcur.index {
            if set.contains(&idx) {return true}
            set.insert(idx);
            mcur.move_next();
        }
        false
    }
}

/* 
impl<T> LinkedList<T> {
    //TODO: add insert functionality to CursorMut
    pub fn reorder_linked_list(list: Self) -> Self {
        unimplemented!()
    }

    pub fn reorder(&mut self) {

    }
}
*/

impl<T> LinkedList<T> {
    pub fn remove_nth_from_end(mut list: LinkedList<T>, n: usize) -> LinkedList<T> {
        let mut mcur = list.cursor_mut();
        for _ in 0..n {
            mcur.move_prev();
        }
        mcur.remove_current();
        list
    }
}

// You know what I know how to solve this
// I'm just gonna move on
//impl<T> LinkedList<T> {
//    pub fn copy_list_with_random_pointer(&self) -> LinkedList<T> {
//        let mut out = LinkedList::new();
//        let mut node_map = std::collections::HashMap::<usize, Self>::new();
//
//
//        out
//
//
//    }
//}

// implementing traits with associated types:
// <T: trait<type = A>>
impl<T: Add<Output = T>> LinkedList<T> {
    pub fn add_two_nodes(mut l1: LinkedList<T>, mut l2: LinkedList<T>) -> LinkedList<T> {
        let mut out: LinkedList<T> = LinkedList::new();

        // check just l1 because the question's constraint implies
        // the two lists will always be of the same length
        while l1.front.is_some() {
            out.push_back(l1.pop_front().unwrap() + l2.pop_front().unwrap());

        }
        out
    }
}

impl LinkedList<i32> {
    pub fn add_two_numbers(l1: LinkedList<i32>, l2: LinkedList<i32>) -> LinkedList<i32> {

        if l1.front.is_none() || l2.front.is_none() {
            return LinkedList::<i32>::new();
        }

        let n1: i32 = l1.into_iter()
                        .fold(String::from(""), |acc, x| format!("{}{}", acc, x))
                        .parse::<i32>().unwrap();
        let n2: i32 = l2.into_iter()
                        .fold(String::from(""), |acc, x| format!("{}{}", acc, x))
                        .parse::<i32>().unwrap();

        let res = format!("{}", n1 + n2).chars()
            .map(|c| c.to_string().parse::<i32>().unwrap())
            .collect::<Vec<i32>>();

        LinkedList::from_iter(res)
    }
}

impl<T: PartialEq + PartialOrd> LinkedList<T> {
    pub fn merge_k_lists(lists: Vec<LinkedList<T>>) -> LinkedList<T> {
        lists.into_iter()
            .fold(LinkedList::new(), |acc, l| LinkedList::merge_two_lists(acc, l))
    }
}

impl<T: Debug> LinkedList<T> {
    pub fn reverse_k_group(&mut self, k: usize) {
        let mut out = LinkedList::new();
        let l = self.len;
        let mut m = self.cursor_mut();
        m.move_next();
        for _i in 0.. l / k {
            let mut new = LinkedList::new();
            for _j in 0..k {
                //println!("{:?}", m.current());
                new.push_front(m.remove_current().unwrap());
            }
            out.extend(new.into_iter());
            //println!("{:?}", out);
        }
        //append reversed list to the front
        m.splice_before(out);
    }
}