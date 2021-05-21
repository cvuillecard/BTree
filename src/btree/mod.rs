use std::vec::Vec;
// use std::hash::{Hash, Hasher};
// use std::collections::hash_map::DefaultHasher;
// use std::any::*;
// use std::ops::*;
use std::cmp::{PartialEq, PartialOrd };
use std::convert::{From, Into};
use std::iter::FromIterator;
use std::fmt;

pub const MAX_SIZE: usize = 500000;
pub const ARITY: usize = 2;

#[derive(Debug)]
pub struct Node<T> {
    data: T
}

impl <T: Sized + PartialOrd + PartialEq> Node<T> {
    pub fn new(data: T) -> Self {
            Node { data }
    }
}

impl <T: Sized + PartialOrd + PartialEq> From<T> for Node<T> { fn from(data: T) -> Self { Node { data } } }
// impl <T: PartialOrd + PartialEq> Into<T> for Node<T> { fn into(self) -> T where T: PartialOrd + PartialEq { self.data } }

#[derive(Debug)]
pub struct VecBtree<T> {
    buf: Vec<Node<T>>
}

pub trait FromIteratorSized<T> : FromIterator<T> {
    fn from_iter_sized<I: IntoIterator<Item = T>>(iter: I, size: usize) -> Self;
}

impl <T: Sized + PartialOrd + PartialEq> FromIteratorSized<T> for VecBtree<T> {
    fn from_iter_sized<I: IntoIterator<Item=T>>(iterable: I, size: usize) -> Self {
        let mut btree = VecBtree::<T>::new(Some(size));

        for i in iterable {
            btree.push(i);
        }

        btree
    }
}

impl <T: Sized + PartialOrd + PartialEq> FromIterator<T> for VecBtree<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iterable: I) -> Self {
        VecBtree::<T>::from_iter_sized(iterable, MAX_SIZE)
    }
}

impl <T: Sized + PartialOrd + PartialEq> VecBtree<T> {
    pub fn new(size: Option<usize>) -> Self {
        match size {
            None => VecBtree { buf: Vec::new() },
            Some(length) => match length {
                length if length <= MAX_SIZE => VecBtree { buf: Vec::with_capacity(length) },
                _ => panic!("initial size must be less or equal to usize={} (pagination not yet implemented)", MAX_SIZE)
            }
        }
    }

    pub fn root(&mut self) -> &mut Node<T> { &mut self.buf[0] }

    pub fn left(&mut self, idx: usize) -> usize { ARITY * idx + 1 }

    pub fn right(&mut self, idx: usize) -> usize { ARITY * idx + 2 }

    pub fn parent(&mut self, idx: usize) -> usize { (idx - 1) / 2 }

    pub fn push(&mut self, data: T) -> &mut Self {
        self.buf.insert(self.buf.len(), Node::from(data));
        self
    }

    pub fn search(&mut self, data: &T) -> Option<&mut T> {
        self.binary_search(data, &mut 0isize, &mut (self.buf.len() as isize - 1))
    }

    fn binary_search(&mut self, data: &T, start_idx: &mut isize, end_idx: &mut isize) -> Option<&mut T> {
        match (*start_idx, *end_idx) {
            (mut start, mut end) => {
                while start <= end {
                    let idx = (start + end >> 1isize) as usize;
                    if self.buf[idx].data == *data {
                        return Some(&mut self.buf[idx].data);
                    }
                    else if self.buf[idx].data < *data {
                        start = idx as isize + 1;
                    }
                    else {
                        end = idx as isize - 1;
                    }
                }
                None
            }
        }
    }

}
//
// pub fn search_fast<'a, T>(buf: &'a mut Vec<Node<T>>, data: &T, nb_chunk: usize) -> Option<&'a mut T> {
//     // let rest = self.buf.len() % nb_chunk;
//     let chunk_size: usize = (buf.len() / nb_chunk); // + if rest > 0 { 1 } else { 0 };
//     let mut chunks: Vec<&mut [Node<T>]> = buf.chunks_mut(chunk_size).collect();
//     // let mut result: &mut Option<&mut T> = &mut None;
//
//     crossbeam::scope(|spawner | {
//         let startIdx = &mut 0isize;
//         let mut i: usize = 0;
//         while let false = i < chunks.len() {
//             spawner.spawn(|| {
//                 binary_search_in(&mut Vec::from(&mut chunks[i]), data, startIdx, *chunks[i].len() - 1);
//             });
//             i += 1;
//         }
//         // for chunck in chunks {
//         //     spawner.spawn(|| {
//         //         match (VecBtree::binary_search_in(chunck, data, startIdx, &mut (chunck.len() as isize - 1))) {
//         //             Some(e) => {
//         //
//         //             }
//         //             None => None
//         //         }
//         //     });
//         // }
//     });
//
//     None
// }
//
// fn binary_search_in<'a, T>(buf: &'a mut Vec<Node<T>>, data: &T, start_idx: &mut isize, end_idx: &mut isize) -> Option<&'a mut T> {
//     match (*start_idx, *end_idx) {
//         (mut start, mut end) => {
//             while start <= end {
//                 let idx = (start + end >> 1isize) as usize;
//                 if *buf[idx].data == *data {
//                     return Some(&mut *buf[idx].data);
//                 }
//                 else if *buf[idx].data < *data {
//                     start = idx as isize + 1;
//                 }
//                 else {
//                     end = idx as isize - 1;
//                 }
//             }
//             None
//         }
//     }
// }
//
// impl <T: Sized + PartialOrd + PartialEq> FromIterator<Node<T>> for std::vec::Vec<&mut [T]> {
//     fn from_iter<I: IntoIterator<Item=Node<T>>>(iterable: I) -> Self {
//         let mut vec: Vec<&mut [Node<T>]> = Vec::with_capacity(MAX_SIZE);
//
//         for i in iterable {
//             vec.push(i);
//         }
//
//         vec
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Borrow;

    #[test]
    #[should_panic]
    fn test_btree_new() {
        assert_eq!(0, VecBtree::<i32>::new(None).buf.capacity());
        assert_eq!(MAX_SIZE, VecBtree::<i32>::new(Some(MAX_SIZE)).buf.capacity());
        assert_eq!(MAX_SIZE + 1, VecBtree::<i32>::new(Some(MAX_SIZE + 1)).buf.capacity());
    }

    #[test]
    fn test_btree_root() {
        let len: i8 = 3;
        let mut btree = VecBtree::<i32>::new(Some(3));

        for i in 1..4 { btree.push(i); }

        assert_eq!(len as usize, btree.buf.len());
        assert_eq!(1, (*btree.root()).data);
    }

    #[test]
    fn test_btree_left_right() {
        let mut btree = VecBtree::<i32>::new(Some(3));

        for i in 1..4 { btree.push(i); }

        assert_eq!(3usize, btree.buf.len());
        assert_eq!(1, btree.left(0));
        assert_eq!(2, btree.right(0));

        let left_idx = btree.left(0);
        let right_idx = btree.right(0);

        assert_eq!(2, btree.buf[left_idx].data);
        assert_eq!(3, btree.buf[right_idx].data);
    }

    #[test]
    fn test_btree_parent() {
        let mut btree = VecBtree::<i32>::new(Some(3));

        for i in 1..4 { btree.push(i); }

        let left_parent = btree.parent(1);
        let right_parent = btree.parent(2);

        assert_eq!(0, left_parent);
        assert_eq!(0, right_parent);
        assert_eq!(1, (*btree.root()).data);
        assert_eq!(1, btree.buf[left_parent].data);
        assert_eq!(1, btree.buf[right_parent].data);
    }

    #[test]
    fn test_search() {
        let mut ints = VecBtree::from_iter((0..100000).collect::<Vec<i32>>());
        let mut letters = VecBtree::from_iter_sized(('a'..='z').collect::<Vec<char>>(), 26);

         // println!("{:?}", letters);

        assert_eq!(Some(&mut 99999), ints.search(&99999));
        assert_eq!(Some(&mut 35321), ints.search(&35321));
        assert_eq!(None, ints.search(&100000));
        assert_eq!(None, ints.search(&500000));
        assert_eq!(None, ints.search(&-500000));

        assert_eq!(Some(&mut 'a'), letters.search(&'a'));
        assert_eq!(Some(&mut 'q'), letters.search(&'q'));
        assert_eq!(Some(&mut 'z'), letters.search(&'z'));
        assert_eq!(None, letters.search(&'Z'));
        assert_eq!(None, letters.search(&'{'));
    }
}
