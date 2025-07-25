use std::mem;

use crate::bitwise::ops::ExternalByteSize;
use crate::bitwise::BitString;
use crate::bitwise::Index;
use crate::tree::traversal;
use crate::tree::traversal::DepthFirstIter;
use crate::tree::traversal::DepthFirstTraverse;
use crate::tree::Edge;
use crate::tree::Labels;
use crate::tree::Node;

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Letter<T> {
    pub end_of_word: bool,
    pub value: T,
}
impl<T> Letter<T> {
    pub fn new(end_of_word: bool, value: T) -> Self {
        Letter { end_of_word, value }
    }
}

#[derive(Debug, Clone)]
pub struct Letters<T> {
    pub end_of_words: BitString,
    pub values: Vec<T>,
}
impl<T> Letters<T> {
    pub fn new() -> Self {
        Letters {
            end_of_words: BitString::new(),
            values: Vec::new(),
        }
    }
}
impl<T> Default for Letters<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> ExternalByteSize for Letters<T>
where
    T: Sized,
{
    fn external_byte_size(&self) -> u64 {
        self.end_of_words.external_byte_size()
            + mem::size_of_val(&self.values.len()) as u64
            + mem::size_of::<T>() as u64 * self.values.len() as u64
    }
}
impl<T> Labels for Letters<T>
where
    T: Clone,
{
    type Label = Letter<T>;
    fn push(&mut self, label: Self::Label) {
        self.end_of_words.push(label.end_of_word);
        self.values.push(label.value);
    }
    fn get(&self, index: usize) -> Option<Self::Label> {
        self.values
            .get(index)
            .cloned()
            .map(|v| Letter::new(self.end_of_words.get(index as Index).unwrap(), v))
    }
    fn len(&self) -> usize {
        self.values.len()
    }
    fn shrink_to_fit(&mut self) {
        self.end_of_words.shrink_to_fit();
        self.values.shrink_to_fit();
    }
}

#[derive(Debug)]
pub struct Words<T, N> {
    buf: Vec<T>,
    stack: Vec<Vec<Edge<Letter<T>, N>>>,
}
impl<T, N> Words<T, N>
where
    N: Node<Letter<T>>,
{
    pub fn new(root: N) -> Self {
        let mut words = Words {
            buf: Vec::new(),
            stack: Vec::new(),
        };
        let mut children = root.children().collect::<Vec<_>>();
        if !children.is_empty() {
            children.reverse();
            words.stack.push(children);
        }
        words
    }
}
impl<T, N> Iterator for Words<T, N>
where
    N: Node<Letter<T>>,
    T: Clone,
{
    type Item = Vec<T>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(mut children) = self.stack.pop() {
            if let Some(e) = children.pop() {
                self.stack.push(children);
                self.buf.push(e.label.value);

                let mut grand_children = e.node.children().collect::<Vec<_>>();
                grand_children.reverse();
                self.stack.push(grand_children);

                if e.label.end_of_word {
                    let word = self.buf.clone();
                    return Some(word);
                }
            } else {
                self.buf.pop();
            }
        }
        debug_assert!(self.buf.is_empty());
        None
    }
}

pub struct DepthFirstTraversal<T, W> {
    buf: Vec<T>,
    path: Vec<(Option<Letter<T>>, usize)>,
    words: W,
}
impl<T, W> DepthFirstTraversal<T, W>
where
    W: Iterator<Item = Vec<T>>,
{
    pub fn new(words: W) -> Self {
        DepthFirstTraversal {
            buf: Vec::new(),
            path: vec![(None, 0)],
            words,
        }
    }
    pub fn iter(self) -> DepthFirstIter<Self> {
        DepthFirstIter::new(self)
    }
}
impl<T, W> DepthFirstTraverse for DepthFirstTraversal<T, W>
where
    W: Iterator<Item = Vec<T>>,
    T: Clone + Eq,
{
    type Label = Letter<T>;
    fn next(&mut self) -> Option<traversal::VisitNode<Self::Label>> {
        loop {
            if self.path.len() <= self.buf.len() {
                let level = self.path.len() - 1;
                let is_terminal = self.path.len() == self.buf.len();
                let label = Letter::new(is_terminal, self.buf[level].clone());
                let nth_child = self.path[level].1;
                self.path.push((Some(label.clone()), 0));
                let node = traversal::VisitNode::new(label, level, nth_child);
                return Some(node);
            } else {
                match self.words.next() {
                    Some(v) => {
                        self.buf = v;
                        if let Some(tail) = self
                            .path
                            .iter()
                            .skip(1)
                            .zip(self.buf.iter())
                            .position(|((l, _), b)| l.as_ref().unwrap().value != *b)
                        {
                            self.path.truncate(tail + 1);
                            self.path[tail].1 += 1;
                        }
                    }
                    None => return None,
                }
            }
        }
    }
}
