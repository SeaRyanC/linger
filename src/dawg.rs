// A compacted Dawg is stored in memory as a flat Vec<u32>.
// The layout of each node is as follows
// offs+0: layout mask
//    - bit 0: is this node terminal?
//    - bits 1, 2, 3, ...: is there a node target for A, B, C, ...?
// offs+N: for each node target that exists, the offs of that node

use crate::letter::Letter;
use std::hash::Hasher;
use std::{collections::HashMap, debug_assert, hash::DefaultHasher};

pub struct DynamicDawg {
    pub dawg: Dawg,
    last_alloc: u32,
}
pub struct Dawg(pub Vec<u32>);

// Maximum size of a node, for when dynamically
// populating an unreduced tree
const NODE_STRIDE: u32 = 32;
// Bit mask of whether this node is terminal
const TERMINAL: u32 = 1;

impl DynamicDawg {
    pub fn make_fillable(capacity: usize) -> DynamicDawg {
        DynamicDawg {
            dawg: Dawg(vec![0; capacity * NODE_STRIDE as usize]),
            last_alloc: 0,
        }
    }

    fn compute_height_map(&self) -> HashMap<u32, u32> {
        let mut result: HashMap<u32, u32> = HashMap::new();

        // Height of a node is 1 + max(children's height),
        // do depth-first out-traversal

        for i in (0..self.dawg.0.len()).step_by(NODE_STRIDE as usize) {
            // Reached the end of the node buffer
            if self.dawg.0[i] == 0 {
                break;
            }
            self.height_of(&mut result, i as u32);
        }

        result
    }

    fn height_of(&self, height_map: &mut HashMap<u32, u32>, node: u32) -> u32 {
        if let Some(entry) = height_map.get(&node) {
            *entry
        } else {
            let mut i = node + 1;
            let mut height = 0;
            loop {
                let c = self.dawg.0[i as usize];
                if c == 0 {
                    break;
                }
                height = std::cmp::max(height, 1 + self.height_of(height_map, c));
                i += 1;
            }
            height_map.insert(node, height);
            height
        }
    }

    // Returns the new number of canonical nodes
    pub fn canonicalize(&mut self) -> usize {
        let height_map = self.compute_height_map();
        // Map from hash of node content to canonical position
        let mut canonical: HashMap<u64, u32> = HashMap::new();
        // Map from original offset to canonical offset
        let mut reassigned: HashMap<u32, u32> = HashMap::new();

        // Canonicalize in reverse depth order (starting at leaves)
        for depth in 0..15 {
            for i in (0..(self.dawg.0.len() as u32)).step_by(NODE_STRIDE as usize) {
                // End of node buffer
                if self.dawg.0[i as usize] == 0 {
                    break;
                }
                // Rewrite this node's children to their canonical forms
                let mut j = i + 1;
                loop {
                    // End of nodes
                    if self.dawg.0[j as usize] == 0 {
                        break;
                    }
                    if let Some(target) = reassigned.get(&self.dawg.0[j as usize]) {
                        self.dawg.0[j as usize] = *target;
                    }
                    j += 1;
                }
                // Is this a node of the relevant depth?
                if *height_map.get(&i).expect("all nodes must be in height map") == depth {
                    let hash = self.node_hash(i);
                    // Either promote to canonical or rewrite
                    if let Some(canon) = canonical.get(&hash) {
                        reassigned.insert(i, *canon);
                    } else {
                        canonical.insert(hash, i);
                    }
                }
            }
        }
        canonical.len()
    }

    pub fn minimize(&self) -> Dawg {
        struct MinContext<'a> {
            minimized: Vec<u32>,
            next_alloc_slot: u32,
            address_map: HashMap<u32, u32>,
            dawg: &'a Dawg,
        }

        // TODO: Compute minimized's size ahead of time
        let mut ctx = MinContext {
            minimized: vec![],
            next_alloc_slot: 0,
            address_map: HashMap::new(),
            dawg: &self.dawg,
        };
        ctx.minimized.resize(128, 0);

        fn recur(ctx: &mut MinContext, node: u32) {
            // If this node has already been allocated, nothing to do.
            if ctx.address_map.contains_key(&node) {
                return;
            }

            // Allocate this node, mapping its original address to its new
            // address, then allocate its child nodes before rewriting
            // its node pointers
            let this_ptr = ctx.next_alloc_slot;
            ctx.address_map.insert(node, this_ptr);
            // Bump the allocator based on this node's size
            let size_to_alloc = (ctx.dawg.0[node as usize] & !TERMINAL).count_ones() + 1;
            ctx.next_alloc_slot += size_to_alloc;
            ctx.minimized.resize(ctx.next_alloc_slot as usize, 0);
            // Allocate each child node
            let mut i = node + 1;
            loop {
                let child = ctx.dawg.0[i as usize];
                if child == 0 {
                    break;
                }
                recur(ctx, child);
                i += 1;
            }
            // Write self + child node contents
            ctx.minimized[this_ptr as usize] = ctx.dawg.0[node as usize];
            for j in 1..size_to_alloc {
                let src = ctx.dawg.0[(node + j) as usize];
                ctx.minimized[(this_ptr + j) as usize] =
                    *ctx.address_map.get(&src).expect("must be alloc'd already");
            }
        }

        recur(&mut ctx, 0);

        Dawg(ctx.minimized)
    }

    pub fn node_hash(&self, offset: u32) -> u64 {
        let slice = &self.dawg.0[offset as usize..(offset + NODE_STRIDE) as usize];
        let mut hash = DefaultHasher::new();
        for s in slice {
            hash.write_u32(*s);
        }
        hash.finish()
    }

    pub fn insert(&mut self, word: &str) {
        let mut current: u32 = 0;
        for c in word.chars() {
            let letter = Letter::from_char(c);
            let target_node: u32;
            let my_mask = self.dawg.0[current as usize];
            // This is the single bit that represents presence of this letter
            // in the node already, e.g. 0b100 for 'B'
            let letter_mask: u32 = letter.mask();
            // This mask is for counting which letters already exist in the node,
            // so that we can determine the distance to this letter's node target.
            let low_mask: u32 = (letter_mask - 1) & !TERMINAL;
            let offset_to_mine: u32 = (low_mask & my_mask).count_ones() + 1;
            debug_assert!(offset_to_mine < NODE_STRIDE);
            if my_mask & letter_mask == letter_mask {
                // The target node already exists
                target_node = self.dawg.0[(current + offset_to_mine) as usize];
            } else {
                // "Create" the target node by moving the allocator
                self.last_alloc += NODE_STRIDE;
                target_node = self.last_alloc;

                let my_target_pos = (current + offset_to_mine) as usize;
                // Fail if a non-sorted insert attempted to occur
                if self.dawg.0[my_target_pos] != 0 {
                    panic!("Non-lexical insertion occurred");
                }

                // Point to it
                self.dawg.0[my_target_pos] = target_node;
                // Update our mask
                self.dawg.0[current as usize] = my_mask | letter_mask;
            }

            current = target_node;
        }

        // Mark last node as terminal
        self.dawg.0[current as usize] = self.dawg.0[current as usize] | TERMINAL;
    }

    pub fn node_count(&self) -> u32 {
        self.last_alloc / NODE_STRIDE
    }
}

impl Dawg {
    pub fn from(vec: Vec<u32>) -> Dawg {
        Dawg(vec)
    }

    pub fn from_bytes(vec: &Vec<u8>) -> Dawg {
        let mut output: Vec<u32> = vec![];
        for i in 0..(vec.len() / 4) {
            let j = i * 4;
            output.push(u32::from_le_bytes([
                vec[j],
                vec[j + 1],
                vec[j + 2],
                vec[j + 3],
            ]));
        }
        Dawg::from(output)
    }

    pub fn read_target(&self, current: usize, letter: Letter) -> usize {
        let mask = letter.mask();
        let low_mask = (mask - 1) & !TERMINAL;
        let low_count = (self.0[current] & low_mask).count_ones() as usize;
        return self.0[current + 1 + low_count] as usize;
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.iter().flat_map(|b| b.to_le_bytes()).collect()
    }

    pub fn get_strings(self: &Self) -> Vec<String> {
        let mut chars: Vec<char> = Vec::new();
        let mut strs: Vec<String> = Vec::new();
        self.get_contents_worker(0, &mut chars, &mut strs);
        strs
    }

    fn get_contents_worker(
        self: &Self,
        current: u32,
        chars: &mut Vec<char>,
        strs: &mut Vec<String>,
    ) {
        let mut mask = self.0[current as usize];
        if mask & TERMINAL == TERMINAL {
            let str: String = chars.iter().collect();
            strs.push(str);
        }
        mask = mask >> 1;

        let mut offset = 1;
        let mut i = 0;
        while mask != 0 {
            if mask & 1 == 1 {
                chars.push(Letter::from_index(i).to_char());
                let next_index = self.0[(current + offset) as usize];
                self.get_contents_worker(next_index, chars, strs);
                chars.pop();
                offset += 1;
            }
            i += 1;
            mask = mask >> 1;
        }
    }

    pub fn print_contents(self: &Self) {
        let mut chars: Vec<char> = Vec::new();
        self.print_contents_worker(0, &mut chars);
    }

    fn print_contents_worker(self: &Self, current: u32, chars: &mut Vec<char>) {
        let mut mask = self.0[current as usize];
        if mask & TERMINAL == TERMINAL {
            let str: String = chars.iter().collect();
            println!("{}", str);
        }
        mask = mask >> 1;

        let mut offset = 1;
        let mut i = 0;
        while mask != 0 {
            if mask & 1 == 1 {
                chars.push(Letter::from_index(i).to_char());
                let next_index = self.0[(current + offset) as usize];
                self.print_contents_worker(next_index, chars);
                chars.pop();
                offset += 1;
            }
            i += 1;
            mask = mask >> 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_insert() {
        let mut dd = DynamicDawg::make_fillable(1024);
        dd.insert("ALPHA");
        dd.dawg.print_contents();
        let output = dd.dawg.get_strings();
        assert_eq!(output.len(), 1);
        assert_eq!("ALPHA", output[0]);
    }

    #[test]
    fn bulk_insert() {
        const SAMPLE_WORDS: &str = include_str!("../data/words-2974.txt");
        let word_list: Vec<&str> = SAMPLE_WORDS
            .lines()
            .filter(|w| -> bool { w.len() != 0 })
            .collect();
        let mut empty = DynamicDawg::make_fillable(SAMPLE_WORDS.len());
        for line in &word_list {
            empty.insert(line);
        }

        let uncanonicalized = empty.dawg.get_strings();
        assert_eq!(word_list, uncanonicalized);

        empty.canonicalize();
        let canonicalized = empty.dawg.get_strings();
        assert_eq!(word_list, canonicalized);

        let min = empty.minimize();
        let minimized = min.get_strings();
        assert_eq!(word_list, minimized);

        let bytes = min.to_bytes();
        let from_bytes = Dawg::from_bytes(&bytes);
        assert_eq!(word_list, from_bytes.get_strings());
    }
}
