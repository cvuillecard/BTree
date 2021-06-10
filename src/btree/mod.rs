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
// impl <T: PartialOrd + PartialEq> Into<T> for Node<T> { fn into(self) -> T where T: PartialOrd + PartialEq { self.data } }

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

// impl <T: 'static + Sized + PartialOrd + PartialEq + Sync + Send> fmt::Display for VecBtree<T> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         let iter = self.buf.iter();
//         // Use `self.number` to refer to each positional data point.
//         for n in self.buf {
//             println!(f, "{}", String::from(n.data));
//         }
//         Ok(())
//         // for e in 0..self.buf.len() {
//         //     write!(f, "{}", self.buf[i]);
//         // }
//     }
// }


impl<T: 'static + Sized + PartialOrd + PartialEq + Sync + Send> VecBtree<T> {
    pub fn new(size: Option<usize>) -> Self {
        match size {
            None => VecBtree { buf: Vec::<Node<T>>::new() },
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
        match (*start_idx, *end_idx) {
            (mut start, mut end) => {
                while start <= end {
                    let idx = (start + end >> 1isize) as usize;
                    if self.buf[idx].data == *data {
                        return Some(&mut self.buf[idx].data);
                    } else if self.buf[idx].data < *data {
                        start = idx as isize + 1;
                    } else {
                        end = idx as isize - 1;
                    }
                }
                None
            }
        }
    }

    // fn search_fast(&mut self, data: &T, start_idx: &mut isize, end_idx: &mut isize, nb_chunk: usize) -> Option<&mut T> {
    //     let rest: usize = self.buf.len() % nb_chunk;
    //     // let chunk_length: usize = self.buf.len() / nb_chunk;
    //
    //     crossbeam::scope(|spawner| {
    //        for chunk in self.buf.chunks_mut(3) {
    //            let mut startIdx = 0isize;
    //            let mut chunk_length = chunk.len() as isize;
    //            for elem in chunk.iter_mut() {
    //                let data = elem.data;
    //                spawner.spawn(move || self.binary_search(data, &mut startIdx, &mut chunk_length))
    //            }
    //            // spawner.spawn(move || {
    //            //     for &mut elem in chunk.iter_mut() {
    //            //
    //            //         println!("{:?}", *elem);
    //            //         // println!("{}", *elem.data);
    //            //         // match elem {
    //            //         //     &mut v => println!("{}", v.data);
    //            //         // }
    //            //     }
    //            // });
    //        }
    //     });
    //     None
    // }

//     fn search_fast(&'static mut self, data: &'static T, start_idx: &mut isize, end_idx: &mut isize, nb_chunk: usize) -> Option<& mut T> {
//         let chunk_length: usize = self.buf.len() / nb_chunk;
//         // let chunk_length: usize = self.buf.len() / nb_chunk;
//         let (tx, rx): (Sender<Option<&mut T>>, Receiver<Option<&mut T>>) = mpsc::channel();
//         // let mut threads = Vec::with_capacity(chunk_length);
//         let responses = Vec::<Option<&mut T>>::with_capacity(chunk_length);
//
//         let chunks_mut: ChunksMut<Node<T>> = self.buf.chunks_mut(chunk_length);
//
//         for chunk in chunks_mut.into_iter() {
//             let tx = tx.clone();
//             thread::spawn( || {
//                 let mut begin = 0isize;
//                 let mut end = chunk.len() as isize;
//                 // let option: Option<&'static mut T> = VecBtree::binary_search(self, data, &mut begin, &mut end);
//
//                 tx.send(None);
//             });
//             // threads.push(thread);
//         }
//
//         for _ in 0..chunk_length {
//             let result = rx.recv();
//         }
//
//         None
//     }
//
}
// pub fn search<T: 'static + Sized + PartialOrd + PartialEq + Sync + Send>(btree: &'static mut VecBtree<T>, data: &'static T) -> Option<&'static mut T> {
//     binary_search::<T>(btree, data, &mut 0isize, &mut (btree.buf.len() as isize - 1))
// }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Borrow;
    use std::sync::{Arc, Mutex};
    use std::ops::RangeInclusive;
    use std::thread::spawn;
    use crossbeam_channel::bounded;

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

    #[test]
    fn test_thread() {
        println!("option : {:?}", test_concurrent_search());
    }

    fn test_concurrent_search() -> Option<char> {
        let chunk_div = 4usize;
        let mut btree = VecBtree::from_iter_sized(('a'..='z').collect::<Vec<char>>(), 26);
        let nb_chunk = (btree.buf.len() / chunk_div) + (if btree.buf.len() % chunk_div > 0 { 1 } else { 0 });
        let mut letter = 'z';
        let mut result: Option<char> = None;
        let lines: Vec<&mut [Node<char>]> = btree.buf.chunks_mut(nb_chunk).collect();
        let (s, r): (crossbeam_channel::Sender<char>, crossbeam_channel::Receiver<char>) = bounded(nb_chunk);

        crossbeam::scope(|spawner| {
            for (i, line) in lines.into_iter().enumerate() {
                let sender = s.clone();
                let handler = spawner.spawn(move |scope| {
                    if let Some(v) = binary_search(line, &letter, &0isize, &((line.len() - 1usize) as isize)) {
                        println!("i : {}, buf: {:?}, found {:?}", i, line, v);
                        sender.send(*v);
                    }
                });
                if !r.is_empty() {
                    break;
                }
            }
        });

        if let Ok(res) = r.recv() { result = Some(res) };

        drop(s);
        drop(r);

        result
    }
}
