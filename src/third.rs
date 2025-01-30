use std::rc::Rc;

pub struct List<T> {
    head: Link<T>,
}

type Link<T> = Option<Rc<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> List<T> {
        List {
            head: None,
        }
    }

    // appends element to head
    pub fn prepend(&self, elem: T) -> List<T> {
        List {
            head: Some(Rc::new(Node {
                elem,
                // Clone for Option<T>
                // Some(x) -> Some(x.clone()) <= in this case we invode Rc::clone()
                // None -> None
                next: self.head.clone(),
            }))
        }
    }

    // Returns list with first element removed
    pub fn tail(&self) -> List<T> {
        //List {
        //    head: self.head
        //        .as_ref()
        //        .map(|node| {
        //            node.next         <- returns Option<Rc<Node<T>>>
        //                .clone()
        //        }),                   <- becomes Option<Option<Rc<Node<T>>>>
        //}

        List {
            head: self.head
                .as_ref()
                .and_then(|node| { // <- and_then "flattens" the Option<Option<T>> statement
                    node.next
                        .clone()
                })
        }

    }

    //returns first element
    pub fn head(&self) -> Option<&T> {
        self.head
            .as_ref()
            .map(|node| {
                &node.elem
            })
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut head = self.head.take();
        while let Some(node) = head {
            if let Ok(mut node) = Rc::try_unwrap(node) {
                // only one strong reference to node
                // can be destroyed
                head = node.next.take();
            }
            else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::List;
    #[test]

    fn basics() {
        let list = List::new();
        assert_eq!(list.head(), None);

        let list = list.prepend(1).prepend(2).prepend(3);
        assert_eq!(list.head(), Some(&3));

        let list = list.tail();
        assert_eq!(list.head(), Some(&2));
        let list = list.tail();
        assert_eq!(list.head(), Some(&1));
        let list = list.tail();
        assert_eq!(list.head(), None);

    }
}

