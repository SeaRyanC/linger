// A compacted Dawg is stored in memory as a flat Box<u32>.
// The layout of each node is as follows
// offs+0: layout mask
//    - bit 31: is this node terminal?
//    - bits 0, 1, 2, ...: is there a node target for A, B, C, ...?
// offs+N: for each node target that exists, the offs of that node

pub struct Dawg(Vec<u32>);

impl Dawg {
    const TERMINAL: u32 = 1 << 31;

    pub fn make_fillable(capacity: usize) -> Dawg {
        Dawg(vec![0; capacity])
    }

    pub fn print_contents(self: &Self) {
        let mut chars: Vec<char> = Vec::new();
        self.print_contents_worker(0, &mut chars);
    }

    fn print_contents_worker(self: &Self, current: u32, chars: &mut Vec<char>) {
        if self.0[current as usize] & Self::TERMINAL != 0 {
            let str: String = chars.iter().collect();
            println!("{}", str);
        }

        let mut offset = 1;
        let mut mask = self.0[current as usize];
        for i in 0 .. 30 {
            if mask & 1 == 1 {
                let ch = Self::index_to_char(i);
                chars.push(ch);
                let next_index = self.0[(current + offset) as usize];
                // println!("at node {}, bit offset {} ({}) goes to {}", current, offset, ch, next_index);
                // println!("mask at {} is {}", next_index, self.0[next_index as usize]);
                self.print_contents_worker(next_index, chars);
                chars.pop();
                offset += 1;
            }
            mask = mask >> 1;
        }
    }

    pub fn insert(&mut self, word: &str, mut next_node: u32) -> u32 {
        let mut current: u32 = 0;
        for c in word.chars() {
            // println!("{}", c);
            let target_node: u32;
            // Bump-alloc to a new target node if one isn't
            // there already
            let my_mask = self.0[current as usize];
            // println!("this mask: {:26b}", my_mask);
            let mask: u32 = 1 << Self::char_to_index(c);
            let low_mask: u32 = mask - 1;
            let offset_to_mine = (low_mask & my_mask).count_ones() + 1;
            if my_mask & mask == mask {
                // The target node already exists
                target_node = self.0[(current + offset_to_mine) as usize];
                // println!("target node already exists in {}, move to {}", current, target_node);
            } else {
                // "Create" the target node
                // N.B. 32 must be > possible chars + 1 (e.g. could be 27)
                next_node += 32;
                target_node = next_node;
                // println!("target node is new, is at {}", target_node);

                let my_target_pos = (current + offset_to_mine) as usize;
                // Fail if a non-sorted insert attempted to occur
                if self.0[my_target_pos] != 0 {
                    panic!("Non-lexical insertion occurred");
                }

                // Point to it
                self.0[my_target_pos] = target_node;
                // Update our mask
                self.0[current as usize] = my_mask | mask;
            }

            current = target_node;
        }

        // Mark last node as terminal
        self.0[current as usize] = self.0[current as usize] | Self::TERMINAL;
        
        next_node
    }

    fn char_to_index(ch: char) -> usize {
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ".find(ch).unwrap()
    }

    fn index_to_char(n: usize) -> char {
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().nth(n).unwrap()
    }
}
