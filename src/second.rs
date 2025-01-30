pub struct List<T> {
    head: Link<T>,
}

type Link<T> = Option<Box<Node<T>>>;

struct  Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> List<T> {
        List {
            head: None,
        }
    }

    pub fn push(&mut self, elem: T) {
        let new_node = Box::new(Node {
            elem,
            next: self.head.take(),
        });

        self.head = Some(new_node);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.head.take()
            //Mapping None    -> None
            //        Some(x) -> Some(y)
            .map(|node| {
                self.head = node.next;
                node.elem
            })
    }

    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|node_ref| {
            &node_ref.elem
        })
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| {
            &mut node.elem
        })
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();
        while let Some(mut node) = cur_link {
            cur_link = node.next.take();
        }

    }
}

// ITERATORS
// into iter: Consumes self -> creates Iterable object
// next: Option<T>
// iter: takes reference of self -> creates iterable object
// next: Option<&T>
// iter mut: takes mutable reference of self -> creates iterable object

pub struct IntoIter<T>(List<T>);
impl <T> List<T> {
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

impl <T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<T> List<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref()
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        // self.next is an IMMUTABLE reference (&)
        // which has a Copy trait
        // even though map takes ownership of self.next
        // the reference COPIES itself before doing so,
        // making this possible without take()
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}


pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<T> List<T> {
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            next: self.head.as_deref_mut()
        }

    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        // we can't have more than one mutable reference
        // map() takes ownership of self, which moves self.next
        // by mapping whats take()n, we can have exclusive access
        // to the mutable reference
        // note that by doing this the self reference is "sharded"
        // in that we are only taking a part (next) of self as mutable reference
        self.next.take().map(|node| {
            // put back what's been "taken"
            self.next = node.next.as_deref_mut();
            &mut node.elem
        })

    }
}


#[cfg(test)]
mod test {
    use super::List;
    #[test]
    fn generics() {
        let mut test_int: List<i32> = List::new();

        assert_eq!(test_int.pop(), None);

        test_int.push(1);
        test_int.push(2);
        test_int.push(3);

        assert_eq!(test_int.pop(), Some(3)); 
        assert_eq!(test_int.pop(), Some(2)); 
        assert_eq!(test_int.pop(), Some(1)); 
        assert_eq!(test_int.pop(), None); 


        let mut test_string: List<String> = List::new();
        assert_eq!(test_string.pop(), None);

        test_string.push("Hello".to_string());
        test_string.push("World".to_string());

        assert_eq!(test_string.pop(), Some(String::from("World")));
        assert_eq!(test_string.pop(), Some(String::from("Hello")));
        assert_eq!(test_string.pop(), None);

    }

    #[test]
    fn test_peek() {
        let mut test_int: List<i32> = List::new();

        test_int.push(1);
        test_int.push(2);
        test_int.push(3);

        assert_eq!(test_int.peek(), Some(&3));
    }


    #[test]
    fn test_peek_mut() {
        let mut test_int: List<i32> = List::new();

        test_int.push(1);
        test_int.push(2);
        test_int.push(3);

        assert_eq!(test_int.peek(), Some(&3));

        let top = test_int.peek_mut();
        *top.unwrap() += 1;
        assert_eq!(test_int.peek(), Some(&4));

        //changing values within Option > Map!
        test_int.peek_mut()
            .map(|val| {
                *val = 42;
            });
        assert_eq!(test_int.peek(), Some(&42));
        assert_eq!(test_int.pop(), Some(42));
    }

    #[test]
    fn into_iter_test() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_test() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), None);
    }
}
