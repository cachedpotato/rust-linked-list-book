use std::mem;

pub struct List {
    head: Link,
}

impl List {
    pub fn new() -> List {
        List {
            head: Link::Empty,
        }
    }

    // Push element to the HEAD of the list
    // (new element) -> (old element n) -> (old element n - 1) -> ...
    //                  ^ self.head here
    pub fn push(&mut self, elem: i32) {
        let new_node = Box::new(Node {
            elem,
            //cannot just do next: self.head
            //because it will temporarily move the head to some place else (new node)
            //which cannot be done with mutable reference
            next: mem::replace(&mut self.head, Link::Empty),
        });

        self.head = Link::More(new_node);
    }

    pub fn pop(&mut self) -> Option<i32> {
        match mem::replace(&mut self.head, Link::Empty) {
            Link::Empty => {
                None
            }
            Link::More(node) => {
                self.head = node.next;
                Some(node.elem)
            }
        }
    }
}

//iterative drop
impl Drop for List { fn drop(&mut self) {
        let mut cur_link = mem::replace(&mut self.head, Link::Empty);
        while let Link::More(mut boxed_node) = cur_link {
            cur_link = mem::replace(&mut boxed_node.next, Link::Empty);
        }
    }
}

enum Link {
    Empty,
    More(Box<Node>),
}

struct Node {
    elem: i32,
    next: Link,
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

        assert_eq!(Some(3), list.pop());
        assert_eq!(Some(2), list.pop());
        assert_eq!(Some(1), list.pop());
        assert_eq!(None, list.pop());

    }
}