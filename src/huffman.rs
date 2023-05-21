#![allow(dead_code)] // XX: Remove this.

use std::{
    cmp::{self, Reverse},
    collections::{BinaryHeap, HashMap},
    io,
};

use bitvec::vec::BitVec;

use crate::shared::read_u8;

type Char = u8;
type Freq = u32;

type FreqMap = HashMap<Char, Freq>;
type CodeMap = HashMap<Char, BitVec>;

#[derive(Debug, PartialEq, Eq)]
struct Stat {
    freq: Freq,
    char: Char,
}

#[derive(Debug, PartialEq, Eq)]
enum Tree {
    Node {
        freq: Freq,
        left: usize,
        right: usize,
    },
    Leaf(Stat),
}

type TreeArena = Vec<Tree>;

/// Encodes the given data.
///
/// # Errors
///
/// Fails if any of the underlying I/O operations fail (i.e., reading from `src`
pub fn enc(_src: &mut dyn io::Read, _out: &mut dyn io::Write) -> io::Result<()> {
    Ok(())
}

/// Decodes the given data.
///
/// # Errors
///
/// Fails if any of the underlying I/O operations fail (i.e., reading from `src`
/// or writing to `out`).
pub fn dec(_src: &mut dyn io::Read, _out: &mut dyn io::Write) -> io::Result<()> {
    Ok(())
}

fn code_map_from_reader(reader: &mut dyn io::Read) -> io::Result<CodeMap> {
    let freq_map = freq_map_from_reader(reader)?;
    let freq_map_len = freq_map.len();
    let tree_arena = tree_from_freq_map(freq_map);
    Ok(code_map_from_tree(freq_map_len, &tree_arena))
}

fn freq_map_from_reader(reader: &mut dyn io::Read) -> io::Result<FreqMap> {
    let mut map = HashMap::new();
    while let Some(char) = read_u8(reader)? {
        *map.entry(char).or_insert(0) += 1;
    }
    Ok(map)
}

fn tree_from_freq_map(map: FreqMap) -> TreeArena {
    let mut queue = BinaryHeap::with_capacity(map.len());
    for (char, freq) in map {
        let leaf = Tree::Leaf(Stat { char, freq });
        // One needs a minimum heap.
        queue.push(Reverse(leaf));
    }

    // A binary tree with `L` leaf nodes may have at most `2L - 1` nodes.
    let node_count = queue.len() * 2 - 1;
    let mut arena = Vec::with_capacity(node_count);

    // The root will be placed at the first index (i.e., `0`). However, since
    // the root node is the last to be inserted, one needs to manually skip
    // its position here.
    //
    // The following is safe since the code below doesn't index `arena[0]`.
    unsafe { arena.set_len(1) };

    while queue.len() >= 2 {
        // SAFETY: See `while` predicate.
        let fst = unsafe { queue.pop().unwrap_unchecked() }.0;
        let snd = unsafe { queue.pop().unwrap_unchecked() }.0;

        let freq = fst.freq() + snd.freq();
        let left = ins(&mut arena, fst);
        let right = ins(&mut arena, snd);

        let node = Tree::Node { freq, left, right };
        queue.push(Reverse(node));
    }

    // At the end of each `while` iteration, one always inserts a new node,
    // hence the following is safe.
    let root = unsafe { queue.pop().unwrap_unchecked() }.0;

    // `0` is is bounds.
    *unsafe { arena.get_unchecked_mut(0) } = root;

    arena
}

fn code_map_from_tree(size_hint: usize, arena: &TreeArena) -> CodeMap {
    fn go(i: usize, arena: &TreeArena, map: &mut CodeMap, vec: BitVec) {
        match &arena[i] {
            Tree::Node { left, right, .. } => {
                let mut left_vec = vec.clone();
                left_vec.push(false);
                go(*left, arena, map, left_vec);

                let mut right_vec = vec;
                right_vec.push(true);
                go(*right, arena, map, right_vec);
            }
            Tree::Leaf(Stat { char, .. }) => {
                map.insert(*char, vec);
            }
        }
    }

    let mut map = HashMap::with_capacity(size_hint);
    go(/* root */ 0, arena, &mut map, BitVec::new());
    map
}

/// Pushes the given element into the vector and returns the inserted-to index.
fn ins<T>(vec: &mut Vec<T>, el: T) -> usize {
    let index = vec.len();
    debug_assert_ne!(index, vec.capacity()); // Do not re-alloc, plz. :)
    vec.push(el);
    index
}

impl Tree {
    fn freq(&self) -> Freq {
        match self {
            Tree::Node { freq, .. } => *freq,
            Tree::Leaf(stat) => stat.freq,
        }
    }
}

impl PartialOrd for Stat {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.freq.partial_cmp(&other.freq)
    }
}

impl Ord for Stat {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.freq.cmp(&other.freq)
    }
}

impl PartialOrd for Tree {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.freq().partial_cmp(&other.freq())
    }
}

impl Ord for Tree {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.freq().cmp(&other.freq())
    }
}

#[cfg(test)]
mod tests {
    use bitvec::{bitvec, order::Lsb0};

    use super::*;

    #[test]
    #[rustfmt::skip]
    fn test_ord_impl() {
        let a = Stat { freq: 5, char: b'A' };
        let b = Stat { freq: 5, char: b'A' };
        let c = Stat { freq: 6, char: b'A' };
        assert_eq!(a, b);
        assert!(a < c);
    }

    #[test]
    fn test_freq_map() {
        let mut src = b"AAABBBAABACD".as_ref();
        let map = freq_map_from_reader(&mut src).unwrap();
        assert_eq!(
            map,
            HashMap::from([(b'A', 6), (b'B', 4), (b'C', 1), (b'D', 1),])
        );
    }

    #[test]
    fn test_code_map() {
        let mut src = b"AAABBBAABACD".as_ref();
        let map = code_map_from_reader(&mut src).unwrap();

        assert_eq!(map[&b'A'], bitvec![usize, Lsb0; 0]);
        assert_eq!(map[&b'B'], bitvec![usize, Lsb0; 1, 1]);

        // The order is not specified, just the bit length.
        assert_ne!(map[&b'C'], map[&b'D']);
        assert!(
            map[&b'C'] == bitvec![usize, Lsb0; 1, 0, 0]
                || map[&b'C'] == bitvec![usize, Lsb0; 1, 0, 1]
        );
        assert!(
            map[&b'D'] == bitvec![usize, Lsb0; 1, 0, 0]
                || map[&b'D'] == bitvec![usize, Lsb0; 1, 0, 1]
        );
    }
}
