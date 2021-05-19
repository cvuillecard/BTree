use std::vec::Vec;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub const MAX_SIZE: usize = 500000;
pub const ARITY: usize = 2;

pub struct Node<T> {
    hash: u64,
    data: T
}

impl <T: Hash> Node<T> {
    pub fn new(data: T, hasher: &mut DefaultHasher) -> Self {
            Node { hash: Node::hash(&data, hasher), data }
    }

    fn hash(data: &T, hasher: &mut DefaultHasher) -> u64 {
        // let mut h = DefaultHasher::new();
        data.hash(hasher);
        hasher.finish()
    }
}

pub struct btree<T> {
    hasher: DefaultHasher,
    buf: Vec<Node<T>>
}

impl <T: Hash> btree<T> {
    pub fn new(size: Option<usize>, hasher: DefaultHasher) -> Self {
        match size {
            None => btree { hasher, buf: Vec::new() },
            Some(length) => match length {
                length if length <= MAX_SIZE => btree { hasher, buf: Vec::with_capacity(length) },
                _ => panic!(format!("initial size must be less or equal to usize={} (pagination not yet implemented)", MAX_SIZE))
            }
        }
    }

    pub fn root(&mut self) -> &mut Node<T> { &mut self.buf[0] }

    pub fn left(&mut self, idx: usize) -> usize { ARITY * idx + 1 }

    pub fn right(&mut self, idx: usize) -> usize { ARITY * idx + 2 }

    pub fn parent(&mut self, idx: usize) -> usize { (idx - 1) / 2 }

    pub fn push(&mut self, data: T) -> &mut Self {
        self.buf.insert(self.buf.len(), Node::new(data, &mut self.hasher));
        self
    }

    pub fn search(&mut self, data: &T) -> Option<&mut T> where T: Hash {
        match self.buf.len() {
            0 => None,
            n => {
                let mut startIdx: usize = 0;
                let mut endIdx: usize = n - 1;
                let mut idx: usize = 0;
                data.hash(&mut self.hasher);
                let mut hash: u64 = self.hasher.finish();

                while startIdx <= endIdx {
                    let idx = startIdx + endIdx >> 1usize;
                    if self.buf[idx].hash == hash {
                        return Some(&mut self.buf[idx].data);
                    }
                    else if self.buf[idx].hash < hash {
                        startIdx = idx + 1;
                    }
                    else {
                        endIdx = idx - 1;
                    }
                }
                None
            }
        }
    }

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
        assert_eq!(0, btree::<i32>::new(None, DefaultHasher::new()).buf.capacity());
        assert_eq!(MAX_SIZE, btree::<i32>::new(Some(MAX_SIZE), DefaultHasher::new()).buf.capacity());
        assert_eq!(MAX_SIZE + 1, btree::<i32>::new(Some(MAX_SIZE + 1), DefaultHasher::new()).buf.capacity());
    }

    #[test]
    fn test_btree_root() {
        let len: i8 = 3;
        let hasher = DefaultHasher::new();
        let mut btree = btree::<i32>::new(Some(3), hasher);

        for i in 1..4 { btree.push(i); }

        assert_eq!(len as usize, btree.buf.len());
        assert_eq!(1, (*btree.root()).data);

        println!("{}", (*btree.root()).hash);
    }

    // #[test]
    // fn test_btree_left_right() {
    //     let len: i8 = 3;
    //     let mut btree = btree::<i32>::new(Some(3));
    //     println!("{:?}", b"totosdfsdf");
    //     for i in 1..4 { btree.buf.push(i); }
    //
    //     let mut left_idx = btree.left(0);
    //
    //     assert_eq!(len as usize, btree.buf.len());
    //     assert_eq!(1, btree.left(0));
    //     assert_eq!(2, btree.right(0 as usize));
    //
    //     let left_idx = btree.left(0);
    //     let right_idx = btree.right(0);
    //
    //     assert_eq!(2, btree.buf[ left_idx]);
    //     assert_eq!(3, btree.buf[ right_idx]);
    // }
    //
    // #[test]
    // fn test_btree_parent() {
    //     let len: i8 = 3;
    //     let mut btree = btree::<i32>::new(Some(3));
    //
    //     for i in 1..4 { btree.buf.push(Node::<i32>::new(i)); }
    //
    //     let left_parent = btree.parent(1);
    //     let right_parent = btree.parent(2);
    //     assert_eq!(0, left_parent);
    //     assert_eq!(0, right_parent);
    //     assert_eq!(1, (*btree.root()).data);
    //     assert_eq!(1, btree.buf[left_parent]);
    //     assert_eq!(1, btree.buf[right_parent]);
    // }
}
