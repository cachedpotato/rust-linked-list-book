// borrow() and borrow_mut() enforces ownership at runtime
// acts the same as & and &mut
use std::rc::Rc;
use std::cell::{Ref, RefCell, RefMut};


// Doubly linked list with Rc<RefCell<Node<T>>>
// This is a bad idea btw
pub struct List<T> {
    head: Link<T>,
    tail: Link<T>,
}

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

pub struct Node<T> {
    elem: T,
    next: Link<T>,
    prev: Link<T>,
}

impl<T> Node<T> {
    pub fn new(elem: T) -> Rc<RefCell<Node<T>>> {
        Rc::new(RefCell::new(
            Node {
                elem,
                next: None,
                prev: None,
            }
        ))
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: None,
            tail: None,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        let new_head = Node::new(elem);
        // new_head    -> next -> old_head
        // new_head    <- prev <- old_head
        // ^ self.head <- move <-  ^ self.head (prev) - take()
        match self.head.take() {
            Some(old_head) => {
                old_head.borrow_mut().prev = Some(new_head.clone()); // +1 new head
                new_head.borrow_mut().next = Some(old_head.clone()); // +1 old head
                self.head = Some(new_head);                          // +1 new head, -1 old_head
            },
            None => {
                self.tail = Some(new_head.clone());
                self.head = Some(new_head);
            }
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        //old_head -> next -> new_head
        // ^ self.head
        self.head.take().map(|old_head| {   // -1 old (self.head)
            match old_head.borrow_mut().next.take() {             // -1 new (next)
                Some(new_head) => {         
                    new_head.borrow_mut().prev.take();            // -1 old
                    self.head = Some(new_head);                   // +1 new
                    // -2 old, 0 new
                },
                None => {
                    self.tail.take();                             // -1 old
                    // -2 old
                }
            }
            Rc::try_unwrap(old_head)
                .ok() // Result<T, Err> -> Option<T>
                .unwrap()
                .into_inner()
                .elem
        })
    }

    // borrow() and borrow_mut() returns Ref<>
    // the lifetime of which is bound to whatever we're trying to return
    // meaning, as soon as Ref<> goes out of scope (this function)
    // the returning value will go out of scope as well.
    // Conclusion: We need to keep the Ref<T> AS the return value
    pub fn peek_front(&self) -> Option<Ref<T>> {
        self.head.as_ref()
            .map(|node| {
                //Yes Ref also has map()
                Ref::map(node.borrow(), |node_ref| &node_ref.elem)
            })
    }

    pub fn peek_front_mut(&mut self) -> Option<RefMut<T>> {
        self.head.as_mut()
            .map(|node| {
                RefMut::map(node.borrow_mut(), |borrowed_node| &mut borrowed_node.elem)
            })
    }


    //head -> next -> next -> ... -> tail
    //head <- prev <- prev <- ... <- tail
    pub fn push_back(&mut self, elem: T) {
        let new_tail = Node::new(elem);
        match self.tail.take() {                                        // -1 old
            Some(old_tail) => {
                old_tail.borrow_mut().next = Some(new_tail.clone());    // +1 new
                new_tail.borrow_mut().prev = Some(old_tail.clone());    // +1 old
                self.tail = Some(new_tail);                             // +1 new
            },
            None => {
                self.head = Some(new_tail.clone());
                self.tail = Some(new_tail);
            }
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.tail
            .take()
            .map(|old_tail| {
                match old_tail.borrow_mut().prev.take() {
                    Some(new_tail) => {
                        new_tail.borrow_mut().next.take();
                        self.tail = Some(new_tail);
                    },
                    None => {
                        self.head.take();
                    }
                }
                Rc::try_unwrap(old_tail).ok().unwrap().into_inner().elem
            })

    }

    pub fn peek_back(&self) -> Option<Ref<T>> {
        self.tail
            .as_ref()
            .map(|cur_tail| {
                Ref::map(cur_tail.borrow(), |borrowed_node| &borrowed_node.elem)
            })

    }

    pub fn peek_back_mut(&mut self) -> Option<RefMut<T>> {
        self.tail.as_mut()
            .map(|tail| {
                RefMut::map(tail.borrow_mut(), |node| &mut node.elem)
            })
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {

        }
    }
}

//IntoIter -> consumes self
pub struct IntoIter<T>(List<T>);
impl<T> List<T> {
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}
impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T>{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop_back()
    }
}

//Iter -> takes reference of self
//pub struct Iter<'a, T>(Option<Ref<'a, Node<T>>>);
//impl<T> List<T> {
//    pub fn iter(&self) -> Iter<T> {
//        Iter(
//            self.head
//                .as_ref()
//                .map(|head| head.borrow())
//        )
//
//    }
//}
//impl<'a, T>  Iterator for Iter<'a, T> {
//    type Item = Ref<'a, T>;
//    fn next(&mut self) -> Option<Self::Item> {
//        self.0.take()
//            .map(|node_ref| {
//                let (next, elem) = Ref::map_split(node_ref, |node| {
//                    (&node.next, &node.elem)
//                });
//
//                self.0 = next.as_ref().map(|head| head.borrow());
//
//                elem
//            })
//    }
//
//}

#[cfg(test)]
mod test {
    use super::List;
    #[test]
    fn basics() {
        let mut list = List::new();
        assert_eq!(list.pop_front(), None);

        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));

        list.push_front(4);
        list.push_front(5);

        assert_eq!(list.pop_front(), Some(5)); 
        assert_eq!(list.pop_front(), Some(4)); 
        assert_eq!(list.pop_front(), Some(1)); 
        assert_eq!(list.pop_front(), None); 
    }

    #[test]
    fn test_peek() {
        let mut list = List::new();
        assert!(list.peek_front().is_none());

        list.push_front(1);
        list.push_front(2);
        list.push_front(3);
        //Ref<T> -> T via * dereferencing
        // T -> &T via & referencing
        assert_eq!(&*list.peek_front().unwrap(), &3);
    }

    #[test]
    fn test_tail() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        assert_eq!(&*list.peek_front().unwrap(), &1);
        assert_eq!(&*list.peek_back().unwrap(), &3);

        *list.peek_back_mut().unwrap() = 42;
        assert_eq!(&*list.peek_back().unwrap(), &42);

        assert_eq!(list.pop_back(), Some(42));
        assert_eq!(list.pop_back(), Some(2));
        assert_eq!(list.pop_back(), Some(1));
        assert_eq!(list.pop_back(), None);
    }
}