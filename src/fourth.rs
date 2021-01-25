use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

pub struct List<T> {
    head: Link<T>,
    tail: Link<T>,
}

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

struct Node<T> {
    elem: T,
    prev: Link<T>,
    next: Link<T>,
}

impl<T> Node<T> {
    fn new(elem: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            elem,
            prev: None,
            next: None,
        }))
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
        // new node needs +2 links, everything else should be +0
        let new_head = Node::new(elem);

        match self.head.take() {
            Some(old_head) => {
                old_head.borrow_mut().prev = Some(new_head.clone()); // add new_head to prev Node
                new_head.borrow_mut().next = Some(old_head); // add old_head to next Node
                self.head = Some(new_head); // set the head to the new head
            }

            None => {
                // empty list, need to set the tail
                self.tail = Some(new_head.clone()); // end of list points to self
                self.head = Some(new_head); // head of list is new_head
            }
        }
    }

    pub fn push_back(&mut self, elem: T) {
        let new_tail = Node::new(elem);

        match self.tail.take() {
            Some(old_tail) => {
                // add new tail to prev Node next
                old_tail.borrow_mut().next = Some(new_tail.clone());

                // add old Node to new tail previous
                new_tail.borrow_mut().prev = Some(old_tail);

                // replace list tail with new tail
                self.tail = Some(new_tail);
            }

            None => {
                self.head = Some(new_tail.clone()); // incr ref to tail
                self.tail = Some(new_tail); // assign tail to list
            }
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|old_head| {
            match old_head.borrow_mut().next.take() {
                // list is not empty.
                Some(new_head) => {
                    new_head.borrow_mut().prev.take(); // remove previous pointer to old_head.
                    self.head = Some(new_head); // set new_head to head of the list.
                }

                // empty the list, head is already 'taken' so take the tail.
                None => {
                    self.tail.take();
                }
            }
            // return the element from old head.
            Rc::try_unwrap(old_head).ok().unwrap().into_inner().elem
        })
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.take().map(|old_tail| {
            match old_tail.borrow_mut().prev.take() {
                Some(prev_tail) => {
                    prev_tail.borrow_mut().next.take(); // remove previous pointer to old tail.
                    self.tail = Some(prev_tail);
                }

                None => {
                    // empty the list tail is already 'taken' so take the head.
                    self.head.take();
                }
            }
            Rc::try_unwrap(old_tail).ok().unwrap().into_inner().elem
        })
    }

    pub fn peek_front(&self) -> Option<Ref<T>> {
        self.head.as_ref().map(|node| {
            // Ref::map docs:
            //
            // Makes a new `Ref` for a component of the borrowed data.
            //
            // The `RefCell` is already immutably borrowed, so this cannot fail.
            //
            // This is an associated function that needs to be used as `Ref::map(...)`.
            // A method would interfere with methods of the same name on the contents
            // of a `RefCell` used through `Deref`.
            //
            // changes the typ eof the previous ref, tied to the old lifetime, to a new one.
            Ref::map(node.borrow(), |node| &node.elem)
        })
    }

    pub fn peek_front_mut(&mut self) -> Option<RefMut<T>> {
        self.head
            .as_mut()
            .map(|node| RefMut::map(node.borrow_mut(), |node| &mut node.elem))
    }

    pub fn peek_back(&self) -> Option<Ref<T>> {
        self.tail
            .as_ref()
            .map(|node| Ref::map(node.borrow(), |node| &node.elem))
    }

    pub fn peek_back_mut(&mut self) -> Option<RefMut<T>> {
        self.tail
            .as_mut()
            .map(|node| RefMut::map(node.borrow_mut(), |node| &mut node.elem))
    }

    #[allow(clippy::should_implement_trait)] // remove into_iter ambiguos call warning
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        // is_some() docs:
        //
        // Returns `true` if the option is a [`Some`] value.
        //
        // shortcut to keep popping if the value is true; while true do nothing.
        while self.pop_front().is_some() {}
    }
}

pub struct IntoIter<T>(List<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.0.pop_front()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.0.pop_back()
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn push_and_pop() {
        let mut list = List::new();

        // check empty list behaves correctly
        assert_eq!(list.pop_front(), None);

        list.push_front(2);
        list.push_front(3);
        list.push_back(1);

        // check normal removal
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));

        // push more to make sure nothing is corrupted
        list.push_front(4);
        list.push_front(5);

        // check normal removal
        assert_eq!(list.pop_front(), Some(5));
        assert_eq!(list.pop_front(), Some(4));

        // check exhaustion
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn peek() {
        let mut list = List::new();

        // immutable peeks
        assert!(list.peek_front().is_none());
        assert!(list.peek_back().is_none());

        // mutable peeks
        assert!(list.peek_front_mut().is_none());
        assert!(list.peek_back_mut().is_none());

        list.push_front(2);
        list.push_front(3);
        list.push_back(1);

        assert_eq!(&*list.peek_front().unwrap(), &3);
        assert_eq!(&mut *list.peek_front_mut().unwrap(), &mut 3);

        assert_eq!(&*list.peek_back_mut().unwrap(), &1);
        assert_eq!(&mut *list.peek_back_mut().unwrap(), &mut 1);
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();

        list.push_front(2);
        list.push_front(3);
        list.push_back(1);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next_back(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);
    }
}
