use std::vec::Vec;
use std::hash::{Hash, Hasher};

pub const MAX_SIZE: usize = 500000;
pub const ARITY: usize = 2;

pub struct Node<T: Hash> {
    hash: u64,
    data: T
}

pub struct btree<T> {
    buf: Vec<T>
}

impl <T> btree<T> {
    pub fn new(size: Option<usize>) -> Self {
        match size {
            None => btree { buf: Vec::new() },
            Some(length) => match length {
                length if length <= MAX_SIZE => btree { buf: Vec::with_capacity(length) },
                _ => panic!(format!("initial size must be less or equal to usize={} (pagination not yet implemented)", MAX_SIZE))
            }
        }
    }

    pub fn get(&mut self) -> &mut Vec<T> { &mut self.buf }

    pub fn root(&mut self) -> &mut T { &mut self.buf[0] }

    pub fn left(&mut self, idx: usize) -> usize { ARITY * idx + 1 }

    pub fn right(&mut self, idx: usize) -> usize { ARITY * idx + 2 }

    pub fn parent(&mut self, idx: usize) -> usize { (idx - 1) / 2 }
}

// impl <T> Into<T> for btree<T> {
//     fn into(self) -> T {
//         unimplemented!()
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Borrow;

    #[test]
    #[should_panic]
    fn test_btree_new() {
        assert_eq!(0, btree::<i32>::new(None).buf.capacity());
        assert_eq!(MAX_SIZE, btree::<i32>::new(Some(MAX_SIZE)).buf.capacity());
        assert_eq!(MAX_SIZE + 1, btree::<i32>::new(Some(MAX_SIZE + 1)).buf.capacity());
    }

    #[test]
    fn test_btree_root() {
        let len: i8 = 3;
        let mut btree = btree::<i32>::new(Some(3));

        for i in 1..4 { btree.buf.push(i); }

        assert_eq!(len as usize, btree.buf.len());
        assert_eq!(1, *btree.root());
    }

    #[test]
    fn test_btree_left_right() {
        let len: i8 = 3;
        let mut btree = btree::<i32>::new(Some(3));
        println!("{:?}", b"totosdfsdf");
        for i in 1..4 { btree.buf.push(i); }

        let mut left_idx = btree.left(0);

        assert_eq!(len as usize, btree.buf.len());
        assert_eq!(1, btree.left(0));
        assert_eq!(2, btree.right(0 as usize));

        let left_idx = btree.left(0);
        let right_idx = btree.right(0);

        assert_eq!(2, btree.buf[ left_idx]);
        assert_eq!(3, btree.buf[ right_idx]);
    }

    #[test]
    fn test_btree_parent() {
        let len: i8 = 3;
        let mut btree = btree::<i32>::new(Some(3));

        for i in 1..4 { btree.buf.push(i); }

        let left_parent = btree.parent(1);
        let right_parent = btree.parent(2);
        assert_eq!(0, left_parent);
        assert_eq!(0, right_parent);
        assert_eq!(1, *btree.root());
        assert_eq!(1, btree.buf[left_parent]);
        assert_eq!(1, btree.buf[right_parent]);
    }
}
