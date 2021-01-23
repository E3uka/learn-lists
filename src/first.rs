use std::mem;

// struct properties ~ to have many values at once whereas enums have 1 of several values.

// tail of the List never allocates extra junk with this method.
// enum is in null pointer optimised all elems are uniformly allocated.

enum Link {
    Empty,
    More(Box<Node>),
}

struct Node {
    elem: i32,
    next: Link,
}

pub struct List {
    head: Link, // can potentially be empty or hold a node.
}

impl List {
    pub fn new() -> Self {
        List {
            head: Link::Empty, // :: is namespace operator
        }
    }

    // pushes an element into the linked list.
    pub fn push(&mut self, elem: i32) {
        let new_node = Box::new(Node {
            elem,
            // replace self.head temporarily with Link::Empty
            next: mem::replace(&mut self.head, Link::Empty),
        });

        // replace head of list with new node.
        self.head = Link::More(new_node);
    }

    // pops a node from the linked list.
    fn pop_node(&mut self) -> Link {
        mem::replace(&mut self.head, Link::Empty) // returns head of list
    }

    // matches the popped node and returns an Option of the inner Element.
    pub fn pop(&mut self) -> Option<i32> {
        match self.pop_node() {
            Link::Empty => None,

            Link::More(node) => {
                self.head = node.next; // assign new head of list.
                Some(node.elem) // return Some elem found.
            }
        }
    }
}

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}

// drops the linked list.
impl Drop for List {
    fn drop(&mut self) {
        // pop the first node from the head of the list.
        let mut cur_link = self.pop_node();

        // while the head still contains Nodes, keep "popping".
        while let Link::More(mut boxed_node) = cur_link {
            cur_link = mem::replace(&mut boxed_node.next, Link::Empty);
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
}
