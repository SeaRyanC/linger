// TODO: fix storage (currently A = 0)
// Stores a letter as a u8 with 1 = A, 2 = B, etc.
// This allows 0 to mean "terminal" in the dawg
// and "blank" in a tile
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Letter(u8);
impl Letter {
    pub fn from_char(c: char) -> Letter {
        debug_assert!(c.is_ascii_uppercase());
        let res = c as u8 - b'A';
        debug_assert!(res <= 26);
        Letter(res)
    }
    pub fn from_index(c: u8) -> Letter {
        Self::from_char((b'A' + c) as char)
    }
    pub fn to_char(self) -> char {
        (b'A' + self.0) as char
    }
    pub fn mask(self) -> u32 {
        2 << self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let alpha = Letter::from_char('A');
        let delta = Letter::from_char('D');
        assert_ne!(alpha, delta);
        let alpha2 = Letter::from_char('A');
        let delta2 = Letter::from_char('D');
        assert_eq!(alpha, alpha2);
        assert_eq!(delta, delta2);
        assert_eq!('A', alpha.to_char());
        assert_eq!('D', delta.to_char());
    }
}