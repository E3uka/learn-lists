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

// Iter is generic over T and has a lifetime 'a where Iter only lives as long as reference to the
// inner reference to the Node of the same type.
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            // doc ref:
            // as_deref Converts from `Option<T>` (or `&Option<T>`) to `Option<&T::Target>`.
            // Leaves the original Option in-place, creating a new one with a reference
            // to the original one, additionally coercing the contents via [`Deref`].
            self.next = node.next.as_deref();

            &node.elem
        })
    }
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

    pub fn into_iterator(self) -> IntoIter<T> {
        IntoIter(self)
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            next: self.head.as_deref(),
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

        let mut iter = list.into_iterator();

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

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }
}
