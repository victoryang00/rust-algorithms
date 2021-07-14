use std::slice;
use std::fmt;
use std::cmp;
use core::ptr;
use std::mem;

pub trait Rdx {
    /// Set the number of buckets used by the generic implementation
    fn cfg_nbuckets() -> usize;
    /// Set the number of rounds scheduled by the generic implementation
    fn cfg_nrounds() -> usize;
    /// Returns the bucket, depending on the round.
    ///
    /// This should respect the radix, e.g.:
    ///
    /// - if the number of buckets is `2` and the type is an unsigned integer, then the result is
    ///   the bit starting with the least significant one.
    /// - if the number of buckets is `8` and the type is an unsigned integer, then the result is
    ///   the byte starting with the least significant one.
    ///
    /// **Never** return a bucker greater or equal the number of buckets. See warning above!
    fn get_bucket(&self, round: usize) -> usize;
    /// Describes the fact that the content of a bucket should be copied back in reverse order
    /// after a certain round.
    fn reverse(round: usize, bucket: usize) -> bool;
}

macro_rules! impl_rdxsort {
    ($t:ty, $alias:ty, $min:expr, $zero:expr) => {
        impl Rdx for $t {
            #[inline]
            fn cfg_nbuckets() -> usize {
                cmp::max(<$alias as Rdx>::cfg_nbuckets(), 3)
            }

            #[inline]
            fn cfg_nrounds() -> usize {
                <$alias as Rdx>::cfg_nrounds() + 1
            }

            #[inline]
            fn get_bucket(&self, round: usize) -> usize {
                if round < <$alias as Rdx>::cfg_nrounds() {
                    let alias = unsafe { mem::transmute::<$t, $alias>(*self) };
                    alias.get_bucket(round)
                } else {
                    if *self == $min {
                        0
                    } else if *self >= $zero {
                        2
                    } else {
                        1
                    }

                }
            }

            #[inline]
            fn reverse(_round: usize, _bucket: usize) -> bool {
                false
            }
        }
    }
}

impl Rdx for u8 {
    #[inline]
    fn cfg_nbuckets() -> usize {
        16
    }

    #[inline]
    fn cfg_nrounds() -> usize {
        2
    }

    #[inline]
    fn get_bucket(&self, round: usize) -> usize {
        let shift = round << 2;
        ((self >> shift) & 15u8) as usize
    }

    #[inline]
    fn reverse(_round: usize, _bucket: usize) -> bool {
        false
    }
}

impl Rdx for u16 {
    #[inline]
    fn cfg_nbuckets() -> usize {
        16
    }

    #[inline]
    fn cfg_nrounds() -> usize {
        4
    }

    #[inline]
    fn get_bucket(&self, round: usize) -> usize {
        let shift = round << 2;
        ((self >> shift) & 15u16) as usize
    }

    #[inline]
    fn reverse(_round: usize, _bucket: usize) -> bool {
        false
    }
}

impl Rdx for u32 {
    #[inline]
    fn cfg_nbuckets() -> usize {
        16
    }

    #[inline]
    fn cfg_nrounds() -> usize {
        8
    }

    #[inline]
    fn get_bucket(&self, round: usize) -> usize {
        let shift = round << 2;
        ((self >> shift) & 15u32) as usize
    }

    #[inline]
    fn reverse(_round: usize, _bucket: usize) -> bool {
        false
    }
}

impl Rdx for u64 {
    #[inline]
    fn cfg_nbuckets() -> usize {
        16
    }

    #[inline]
    fn cfg_nrounds() -> usize {
        16
    }

    #[inline]
    fn get_bucket(&self, round: usize) -> usize {
        let shift = round << 2;
        ((self >> shift) & 15u64) as usize
    }

    #[inline]
    fn reverse(_round: usize, _bucket: usize) -> bool {
        false
    }
}


impl Rdx for bool {
    #[inline]
    fn cfg_nbuckets() -> usize {
        2
    }

    #[inline]
    fn cfg_nrounds() -> usize {
        1
    }

    #[inline]
    fn get_bucket(&self, _round: usize) -> usize {
        if *self {
            1
        } else {
            0
        }
    }

    #[inline]
    fn reverse(_round: usize, _bucket: usize) -> bool {
        false
    }
}

impl_rdxsort!(i8, u8, i8::min_value(), 0i8);
impl_rdxsort!(i16, u16, i16::min_value(), 0i16);
impl_rdxsort!(i32, u32, i32::min_value(), 0i32);
impl_rdxsort!(i64, u64, i64::min_value(), 0i64);


enum Node<T: Rdx> {
    Inner(NodeInner<T>),
    Child(T),
    Free,
}

struct NodeInner<T: Rdx> {
    round: usize,
    children: Vec<Node<T>>,
}

impl<T: Rdx> NodeInner<T> {
    fn new(round: usize, nbuckets: usize) -> NodeInner<T> {
        let mut children = Vec::with_capacity(nbuckets);
        for _ in 0..nbuckets {
            children.push(Node::Free);
        }
        NodeInner {
            round: round,
            children: children,
        }
    }

    fn insert(&mut self, x: T) {
        let bucket = x.get_bucket(self.round - 1);

        if self.round > 1 {
            let clen = self.children.len();
            let replace = match self.children[bucket] {
                Node::Free => {
                    let mut inner = NodeInner::new(self.round - 1, clen);
                    inner.insert(x);
                    Some(inner)
                }
                Node::Inner(ref mut inner) => {
                    inner.insert(x);
                    None
                }
                Node::Child(_) => unreachable!(),
            };

            if let Some(inner) = replace {
                self.children[bucket] = Node::Inner(inner);
            }
        } else {
            let alloc = match self.children[bucket] {
                Node::Free => true,
                Node::Child(_) => false,
                Node::Inner(_) => unreachable!(),
            };

            if alloc {
                self.children[bucket] = Node::Child(x);
            } else {
                match self.children[bucket] {
                    Node::Child(ref mut y) => *y = x, // XXX: is that a good idea?
                    _ => unreachable!(),
                }
            }
        }
    }

    fn nnodes(&self) -> usize {
        let mut result = 1;
        for c in self.children.iter() {
            match c {
                &Node::Inner(ref inner) => {
                    result += inner.nnodes();
                }
                _ => {}
            }
        }
        result
    }
}

pub struct RdxTree<T: Rdx> {
    root: Node<T>,
}

impl<T: Rdx> RdxTree<T> {
    pub fn new() -> RdxTree<T> {
        let rounds = <T as Rdx>::cfg_nrounds();
        let buckets = <T as Rdx>::cfg_nbuckets();
        RdxTree {
            root: Node::Inner(NodeInner::<T>::new(rounds, buckets)),
        }
    }

    pub fn insert(&mut self, x: T) {
        match self.root {
            Node::Inner(ref mut inner) => {
                inner.insert(x);
            }
            _ => {
                unreachable!();
            }
        }
    }

    pub fn iter<'a>(&'a self) -> RdxTreeIter<'a, T> {
        let mut iters = Vec::new();
        match self.root {
            Node::Inner(ref inner) => {
                iters.push(inner.children.iter());
            }
            _ => unreachable!(),
        }
        RdxTreeIter { iters: iters }
    }

    pub fn nnodes(&self) -> usize {
        match self.root {
            Node::Inner(ref inner) => inner.nnodes(),
            _ => {
                unreachable!()
            }
        }
    }
}

pub struct RdxTreeIter<'a, T: Rdx + 'a> {
    iters: Vec<slice::Iter<'a, Node<T>>>,
}

impl<'a, T: Rdx + 'a> Iterator for RdxTreeIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        let mut result: Option<&'a T> = None;

        while self.iters.len() > 0 && result.is_none() {
            let mut push: Option<slice::Iter<'a, Node<T>>> = None;
            let mut pop = false;

            if let Some(mut it) = self.iters.last_mut() {
                match it.next() {
                    Some(&Node::Free) => {}
                    Some(&Node::Child(ref x)) => {
                        result = Some(x);
                    }
                    Some(&Node::Inner(ref inner)) => {
                        push = Some(inner.children.iter());
                    }
                    None => {
                        pop = true;
                    }
                }
            } else {
                unreachable!();
            }

            if pop {
                self.iters.pop();
            } else if let Some(next) = push {
                self.iters.push(next);
            }
        }

        result
    }
}

fn print_node<T: fmt::Display + Rdx>(node: &Node<T>, depth: usize) {
    let prefix: String = (0..depth).map(|_| ' ').collect();
    match *node {
        Node::Inner(ref inner) => {
            for (i, c) in inner.children.iter().enumerate() {
                println!("{}{}:", prefix, i);
                print_node(c, depth + 1);
            }
        }
        Node::Child(ref x) => {
            println!("{}=> {}", prefix, x);
        }
        Node::Free => {
            println!("{}X", prefix);
        }
    }
}

fn print_tree<T: fmt::Display + Rdx>(tree: &RdxTree<T>) {
    print_node(&tree.root, 0);
}


/// Radix Sort implementation for some type
pub trait RdxSort {
    /// Execute Radix Sort, overwrites (unsorted) content of the type.
    fn rdxsort(&mut self);
}

#[inline]
fn helper_bucket<T, I>(buckets_b: &mut Vec<Vec<T>>, iter: I, cfg_nbuckets: usize, round: usize)
    where T: Rdx,
          I: Iterator<Item = T>
{
    for x in iter {
        let b = x.get_bucket(round);
        assert!(b < cfg_nbuckets,
                "Your Rdx implementation returns a bucket >= cfg_nbuckets()!");
        unsafe {
            buckets_b.get_unchecked_mut(b).push(x);
        }
    }
}

impl<T> RdxSort for [T] where T: Rdx + Clone
{
    fn rdxsort(&mut self) {
        // config
        let cfg_nbuckets = T::cfg_nbuckets();
        let cfg_nrounds = T::cfg_nrounds();

        // early return
        if cfg_nrounds == 0 {
            return;
        }

        let n = self.len();
        let presize = cmp::max(16, (n << 2) / cfg_nbuckets);  // TODO: justify the presize value
        let mut buckets_a: Vec<Vec<T>> = Vec::with_capacity(cfg_nbuckets);
        let mut buckets_b: Vec<Vec<T>> = Vec::with_capacity(cfg_nbuckets);
        for _ in 0..cfg_nbuckets {
            buckets_a.push(Vec::with_capacity(presize));
            buckets_b.push(Vec::with_capacity(presize));
        }

        helper_bucket(&mut buckets_a, self.iter().cloned(), cfg_nbuckets, 0);

        for round in 1..cfg_nrounds {
            for bucket in &mut buckets_b {
                bucket.clear();
            }
            for (i, bucket) in buckets_a.iter().enumerate() {
                if T::reverse(round - 1, i) {
                    helper_bucket(&mut buckets_b,
                                  bucket.iter().rev().cloned(),
                                  cfg_nbuckets,
                                  round);
                } else {
                    helper_bucket(&mut buckets_b, bucket.iter().cloned(), cfg_nbuckets, round);
                }
            }
            mem::swap(&mut buckets_a, &mut buckets_b);
        }

        let mut pos = 0;
        for (i, bucket) in buckets_a.iter_mut().enumerate() {
            assert!(pos + bucket.len() <= self.len(),
                    "bug: a buckets got oversized");

            if T::reverse(cfg_nrounds - 1, i) {
                for x in bucket.iter().rev().cloned() {
                    unsafe {
                        *self.get_unchecked_mut(pos) = x;
                    }
                    pos += 1;
                }
            } else {
                unsafe {
                    ptr::copy_nonoverlapping(bucket.as_ptr(),
                                             self.get_unchecked_mut(pos),
                                             bucket.len());
                }
                pos += bucket.len();
            }
        }

        assert!(pos == self.len(), "bug: bucket size does not sum up");
    }
}

impl<T> RdxSort for Vec<T> where [T]: RdxSort
{
    fn rdxsort(&mut self) {
        self.as_mut_slice().rdxsort();
    }
}