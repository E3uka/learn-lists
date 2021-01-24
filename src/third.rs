// aims of this file:
//
// list1 = A -> B -> C -> D
// list2 = tail(list1) = B -> C -> D
// list3 = push(list2, X) = X -> B -> C -> D
//
// should create:
//
// list1 -> A ---+
//               |
//               v
// list2 ------> B -> C -> D
//               ^
//               |
// list3 -> X ---+

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
    pub fn new() -> Self {
        List { head: None }
    }

    // append takes a list and an element and returns a List.
    // it creates a new Node that has the old list as its next value.
    pub fn append(&self, elem: T) -> List<T> {
        List {
            head: Some(Rc::new(Node {
                elem,
                next: self.head.clone(),
            })),
        }
    }

    // tail takes a lsit and returns the whole list with the first element removed.
    pub fn tail(&self) -> List<T> {
        List {
            // doc ref for and_then():
            // Returns [`None`] if the option is [`None`], otherwise calls `f` with the
            // wrapped value and returns the result.
            // Some languages call this operation flatmap.
            //
            // clone() returns a copy of next
            head: self.head.as_ref().and_then(|node| node.next.clone()),
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }

    // gets the head of the linked list.
    pub fn head(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.elem)
    }
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            // reassign self.next ready for the next call.
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn heads_and_tails() {
        let list = List::new();
        assert_eq!(list.head(), None);

        // returns a new list instead of mutating.
        let list = list.append(1).append(2).append(3);
        assert_eq!(list.head(), Some(&3));

        // remove the head of the list.
        let list = list.tail();
        assert_eq!(list.head(), Some(&2));

        let list = list.tail();
        assert_eq!(list.head(), Some(&1));

        let list = list.tail();
        assert_eq!(list.head(), None);

        // assert that empty tail works correctly.
        let list = list.tail();
        assert_eq!(list.head(), None);
    }

    #[test]
    fn iter() {
        let list = List::new().append(1).append(2).append(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), None);
    }
}
