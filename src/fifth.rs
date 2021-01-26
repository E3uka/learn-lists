use std::ptr;

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

pub struct List<T> {
    head: Link<T>,
    tail: *mut Node<T>, // unsafe here.
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: None,
            tail: ptr::null_mut(),
        }
    }

    // Pushes an element to the end of the list.
    //
    // a lifetime of <'a> for the inner type T is declared for the impl scope
    //
    // function body of push() is declared &mut 'xyz' or desugared: (&'_ mut 'xyz')
    // this declaring an anonymous lifetime that must be inferred by the compiler
    // the function as_deref_mut() converts from and Option<T> to a Option<&mut T::Target>
    // knowledge of Target lifetime required so reference does not outlive the target
    //
    // we have to specifically tell the compiler that we are borrowing from ourself and we will
    // last as long as T exists.
    pub fn push(&mut self, elem: T) {
        let mut new_tail = Box::new(Node { elem, next: None });

        // creating a raw pointer with coercion
        // if a variable is declared to be a raw pointer, a normal reference will coerce into it
        let raw_tail: *mut _ = &mut *new_tail;

        // equivalent for checking for None but with raw pointers.
        if !self.tail.is_null() {
            // if the old tail existed, update it to point to the new tail
            // derefencing a raw pointer is unsafe; unsafety block must be explicitly shown
            //
            // operator precedence i.e. specify which operation comes first:
            //
            // (raw_pointer_derefencing).(address field) or (first).(second)
            unsafe { (*self.tail).next = Some(new_tail) };
        } else {
            self.head = Some(new_tail);
        }

        self.tail = raw_tail;
    }

    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|old_head| {
            let old_head = *old_head; // derefence the boxed head.
            self.head = old_head.next; // assign the next head to the next node of the old

            // if the current head is None set the tail to None.
            if self.head.is_none() {
                self.tail = ptr::null_mut();
            }

            old_head.elem
        })
    }

    pub fn peek(&self) -> Option<&T> {
        // as_ref demotes '&Option<T>' to 'Option<&T>'.
        self.head.as_ref().map(|node| &node.elem)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| &mut node.elem)
    }

    #[allow(clippy::should_implement_trait)] // remove into_iter ambiguos call warning
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }

    // using Rust 2018 explicitly elided lifetime syntax.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            next: self.head.as_deref_mut(),
        }
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
            self.next = node.next.as_deref();

            &node.elem
        })
    }
}

// IterMut iterates over &mut T.
pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            self.next = node.next.as_deref_mut();

            &mut node.elem
        })
    }
}

pub struct IntoIter<T>(List<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop() // use internal pop method to take ownership of internal Node.
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn push_and_pop() {
        let mut list = List::new();

        // check empty list behaves right
        assert_eq!(list.pop(), None);

        // populate list.
        list.push(1);
        list.push(2);
        list.push(3);

        // check normal removal.
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        // push some more to make sure nothing is corrupted.
        list.push(4);
        list.push(5);

        // check normal removal.
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));

        // check exhaustion.
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_mut() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(&mut 1));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), None);
    }
}
