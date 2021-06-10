use std::vec::Vec;
// use std::hash::{Hash, Hasher};
// use std::collections::hash_map::DefaultHasher;
// use std::any::*;
// use std::ops::*;
use std::cmp::{PartialEq, PartialOrd};
use std::convert::{From, Into};
use std::iter::FromIterator;
use std::{fmt, thread};
use std::borrow::Borrow;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::slice::ChunksMut;
use crossbeam_channel::bounded;
use std::fmt::Debug;

pub const MAX_SIZE: usize = 500000;
pub const ARITY: usize = 2;

#[derive(Debug, Clone, Copy)]
pub struct Node<T> {
    data: T
}

impl<T: 'static + Sized + PartialOrd + PartialEq + Sync + Send> Node<T> {
    pub fn new(data: T) -> Self {
        Node { data }
    }
}

impl<T: 'static + Sized + PartialOrd + PartialEq + Sync + Send> From<T> for Node<T> { fn from(data: T) -> Self { Node { data } } }

#[derive(Debug, Clone)]
pub struct VecBtree<T> {
    buf: Vec<Node<T>>
}

pub trait FromIteratorSized<T>: FromIterator<T> {
    fn from_iter_sized<I: IntoIterator<Item=T>>(iter: I, size: usize) -> Self;
}

impl<T: 'static + Sized + PartialOrd + PartialEq + Sync + Send> FromIteratorSized<T> for VecBtree<T> {
    fn from_iter_sized<I: IntoIterator<Item=T>>(iterable: I, size: usize) -> Self {
        let mut btree = VecBtree::<T>::new(Some(size));

        for i in iterable {
            btree.push(i);
        }

        btree
    }
}

impl<T: 'static + Sized + PartialOrd + PartialEq + Sync + Send> FromIterator<T> for VecBtree<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iterable: I) -> Self {
        VecBtree::<T>::from_iter_sized(iterable, MAX_SIZE)
    }
}

impl<T: 'static + Sized + PartialOrd + PartialEq + Sync + Send> VecBtree<T> {

    pub fn new(size: Option<usize>) -> Self {
        match size {
            None => VecBtree { buf: Vec::<Node<T>>::new()},
            Some(length) => match length {
                length if length <= MAX_SIZE => VecBtree { buf: Vec::<Node<T>>::with_capacity(length) },
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
        while *start_idx <= *end_idx {
            let idx = (*start_idx + *end_idx >> 1isize) as usize;
            if self.buf[idx].data == *data {
                return Some(&mut self.buf[idx].data);
            } else if self.buf[idx].data < *data {
                *start_idx = idx as isize + 1;
            } else {
                *end_idx = idx as isize - 1;
            }
        }
        None
    }

}

fn binary_search<'a, T: std::cmp::PartialEq + std::cmp::PartialOrd>(buf: &'a [Node<T>], data: &'a T, start_idx: &'a isize, end_idx: &'a isize) -> Option<&'a T> {
    match (*start_idx, *end_idx) {
        (mut start, mut end) => {
            while start <= end {
                let idx = (start + end >> 1isize) as usize;
                if buf[idx].data == *data {
                    return Some(&buf[idx].data);
                } else if buf[idx].data < *data {
                    start = idx as isize + 1;
                } else {
                    end = idx as isize - 1;
                }
            }
            None
        }
    }
}

fn search_by_chunk<T: PartialEq + PartialOrd + Sync + Send + Copy + Debug>(btree: &mut VecBtree<T>, data: &T, chunk_size: usize) -> Option<(usize, T)> {
    let nb_chunk = (btree.buf.len() / chunk_size) + (if btree.buf.len() % chunk_size > 0 { 1 } else { 0 });
    let mut result: Option<(usize, T)> = None;
    let lines: Vec<&mut [Node<T>]> = btree.buf.chunks_mut(nb_chunk).collect();
    let (s, r): (crossbeam_channel::Sender<(usize, T)>, crossbeam_channel::Receiver<(usize, T)>) = bounded(nb_chunk);

    crossbeam::scope(|spawner| {
        for (i, line) in lines.into_iter().enumerate() {
            if !r.is_empty() {
                break;
            }
            let sender = s.clone();
            spawner.spawn(move |scope| {
                if let Some(v) = binary_search(line, data, &0isize, &((line.len() - 1usize) as isize)) {
                    println!("iteration : {}, input : {:?} => found {:?}", i, line, v);
                    sender.send((i, *v));
                }
            });
        }
    });

    if !r.is_empty() {
        if let Ok((i, res)) = r.recv() { result = Some((i, res)) };
    }

    drop(s);
    drop(r);

    result
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_search_by_chunk() {
        let mut btree = VecBtree::from_iter_sized(('a'..='z').collect::<Vec<char>>(), 26);

        assert_eq!((0, 'b'), search_by_chunk::<char>(&mut btree, &'b', 4usize).unwrap());
        assert_eq!((3, 'z'), search_by_chunk::<char>(&mut btree, &'z', 4usize).unwrap());
        assert_eq!((0, 'a'), search_by_chunk::<char>(&mut btree, &'a', 4usize).unwrap());
        assert_eq!((2, 'r'), search_by_chunk::<char>(&mut btree, &'r', 4usize).unwrap());
        assert_eq!((None), search_by_chunk::<char>(&mut btree, &'~', 4usize));
    }
}
