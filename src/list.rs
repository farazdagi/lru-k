#![allow(dead_code)]

use crate::Id;

const INVALID: Id = Id::MAX;

#[inline]
fn is_valid(id: Id) -> bool {
    id != INVALID
}

#[derive(Clone, Copy, Default)]
struct Links {
    next: Option<Id>,
    prev: Option<Id>,
}

impl Links {
    const fn new() -> Self {
        Self {
            next: None,
            prev: None,
        }
    }
}

/// A doubly linked list of Ids, with a fixed capacity.
#[derive(Clone, Copy)]
struct List<const CAP: usize> {
    links: [Links; CAP],
    head: Option<Id>,
    tail: Option<Id>,
}

impl<const CAP: usize> List<CAP> {
    const fn new() -> Self {
        Self {
            links: [Links::new(); CAP],
            head: None,
            tail: None,
        }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    fn push_front(&mut self, id: Id) {
        assert!((id as usize) < CAP);

        let old_head = self.head;
        self.head = Some(id);
        if self.tail.is_none() {
            self.tail = Some(id);
        }

        let link = &mut self.links[id as usize];
        link.prev = None;
        link.next = old_head;

        if let Some(old_head) = old_head {
            self.links[old_head as usize].prev = Some(id);
        }
    }

    fn remove(&mut self, id: Id) {
        if (id as usize) >= CAP {
            return;
        }

        let (next, prev) = {
            let link = &self.links[id as usize];
            (link.next, link.prev)
        };

        if let Some(prev) = prev {
            self.links[prev as usize].next = next;
        } else {
            self.head = next;
        }

        if let Some(next) = next {
            self.links[next as usize].prev = prev;
        } else {
            self.tail = prev;
        }

        self.links[id as usize].next = None;
        self.links[id as usize].prev = None;
    }

    fn pop_back(&mut self) -> Option<Id> {
        let tail = self.tail;
        if let Some(t) = tail {
            self.remove(t);
        }
        tail
    }

    fn pop_front(&mut self) -> Option<Id> {
        let head = self.head;
        if let Some(head) = head {
            self.remove(head);
        }
        head
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_pop_front() {
        let mut list = List::<3>::new();

        list.push_front(0);
        list.push_front(1);
        list.push_front(2);

        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), Some(0));
        assert_eq!(list.pop_front(), None);
        assert!(list.is_empty());
    }

    #[test]
    fn push_and_pop_back() {
        let mut list = List::<2>::new();

        list.push_front(0);
        list.push_front(1);

        assert_eq!(list.pop_back(), Some(0));
        assert_eq!(list.pop_back(), Some(1));
        assert_eq!(list.pop_back(), None);
        assert!(list.is_empty());
    }

    #[test]
    fn remove_middle() {
        let mut list = List::<3>::new();

        list.push_front(0);
        list.push_front(1);
        list.push_front(2); // List: 2 -> 1 -> 0

        list.remove(1); // Remove middle

        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_front(), Some(0));
        assert_eq!(list.pop_front(), None);
        assert!(list.is_empty());
    }
}
