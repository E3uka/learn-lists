// Link is generic over T and it contains an Option-Box-Generic Node.
type Link<T> = Option<Box<Node<T>>>;

// Node is generic over T.
struct Node<T> {
    elem: T,
    next: Link<T>,
}

// List is generic over T.
pub struct List<T> {
    head: Link<T>,
}

impl<T> List<T> {
    // Self refers the the object that is after the impl statement, thus we do not need to change
    // method signature when implementing generically.
    pub fn new() -> Self {
        List { head: None }
    }

    // pushes an element into the linked list.
    pub fn push(&mut self, elem: T) {
        let new_node = Box::new(Node {
            elem,
            next: self.head.take(),
        });

        // replace head of list with new node.
        self.head = Some(new_node);
    }

    // pops a node from the linked list.
    fn pop_node(&mut self) -> Link<T> {
        self.head.take()
    }

    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            node.elem
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

// generically implement a Drop for a generic List.
impl<T> Drop for List<T> {
    fn drop(&mut self) {
        // pop the first node from the head of the list.
        let mut cur_link = self.pop_node();

        // while the head still contains Nodes, keep "popping" and shadow cur_link.
        while let Some(mut boxed_node) = cur_link {
            cur_link = boxed_node.next.take();
        }
    }
}

// Iter iterates over &T and has a lifetime 'a where Iter only lives as long as reference to the
// inner reference to the Node of the same type.
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            // reassign self.next ready for the next call.
            // as_deref() Converts from `Option<T>` (or `&Option<T>`) to `Option<&T::Target>`.
            // Leaves the original Option in-place, creating a new one with a reference
            // to the original one, additionally coercing the contents via [`Deref`].
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

    // desugared:
    //
    // fn next<'b>(&'b mut self) -> Option<'a, T>
    //
    // Thus there is no constraint between the lifetime of the next functions input and output.
    // We must manually add a constraint by taking an exclusive reference to the value inside the
    // Option before mappping over the next value.
    fn next(&mut self) -> Option<Self::Item> {
        // we take the Option<&mut> so we have exclusive access to the mutable reference, thus
        // there is no need to worry about someone looking at it again.
        self.next.take().map(|node| {
            // reassign self.next ready for the next call.
            // as_deref_mut() Converts from `Option<T>` (or `&mut Option<T>`) to
            // `Option<&mut T::Target>`.
            self.next = node.next.as_deref_mut();

            &mut node.elem
        })
    }
}

// IntoIter iterates over T.
// Tuple structs are an alternative form of struct, useful for trivial wrappers around other types
// and accessible via the dot notation i.e. 'variable.0'.
pub struct IntoIter<T>(List<T>);

// implement iterator for the new struct type that List<T> changes into when you call the the
// into_iter() method.
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
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        // push some more to make sure nothing is corrupted.
        list.push(4);
        list.push(5);

        // check normal removal.
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), Some(4));

        // check exhaustion.
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn peek() {
        let mut list = List::new();
        assert_eq!(list.peek(), None);
        assert_eq!(list.peek_mut(), None);

        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.peek(), Some(&3));
        assert_eq!(list.peek_mut(), Some(&mut 3));

        // test the mutation. can do any of the ways below.
        list.peek_mut().map(|val| {
            *val = 42;
        });

        assert_eq!(list.peek(), Some(&42));
        assert_eq!(list.pop(), Some(42));
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();

        list.push(1);
        list.push(2);
        list.push(3);

        // test that you get ownership to the inner value wrapped in an option.
        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter() {
        let mut list = List::new();

        list.push(1);
        list.push(2);
        list.push(3);

        // test that you get a shared reference to the inner value wrapped in an option.
        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }

    #[test]
    fn iter_mut() {
        let mut list = List::new();

        list.push(1);
        list.push(2);
        list.push(3);

        // test that you get an exclusive reference to the inner value wrapped in an option.
        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 1));
    }
}
