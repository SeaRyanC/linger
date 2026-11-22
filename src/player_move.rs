use crate::letter::Letter;

pub enum PlayerMove {
    Play(PlayMove),
    Pass
}

pub struct PlayMove {
    pub row: u8,
    pub col: u8,
    pub transposed: bool,
    pub letters: Vec<Letter>
}
