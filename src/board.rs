use crate::letter::Letter;
use crate::player_move::*;

pub const BOARD_SIZE: u8 = 15;
pub struct Board {
    letters: Vec<Option<Letter>>,
    scores: Vec<u8>,
    primary_checks: Vec<u32>,
    transposed_checks: Vec<u32>
}

pub struct BoardView<'a> (pub &'a Board, pub bool);

impl Board {
    pub fn new() -> Board {
        let size = (BOARD_SIZE * BOARD_SIZE) as usize;
        Board {
            letters: vec![Option::None; size],
            scores: vec![0; size],
            primary_checks: vec![!0; size],
            transposed_checks: vec![!0; size]
        }
    }

    pub fn with_move(&self, mv: PlayerMove) -> Board {
        let mut res = Board {
            letters: self.letters.clone(),
            scores: self.scores.clone(),
            primary_checks: self.primary_checks.clone(),
            transposed_checks: self.transposed_checks.clone()
        };

        match mv {
            PlayerMove::Pass => (),
            PlayerMove::Play(pm) => {
                let mut row = pm.row;
                let mut col = pm.col;
                let mut i = 0;
                loop {
                    res.letters[(row * BOARD_SIZE + col) as usize] = Some(pm.letters[i]);
                    i += 1;
                    if i == pm.letters.len() {
                        break;
                    }
                    // Find next open cell
                    loop {
                        if pm.transposed {
                            row += 1;
                        } else {
                            col += 1;
                        }
                        if let None = BoardView(&res, false).letter_at(row, col) {
                            break;
                        }
                    }
                }
            }
        }

        res
    }
}

impl BoardView<'_> {
    pub fn letter_at(&self, row: u8, col: u8) -> Option<Letter> {
        self.0.letters[self.idx(row, col)]
    }
    
    fn idx(&self, row: u8, col: u8) -> usize {
        if self.1 {
            (col * BOARD_SIZE + row) as usize
        } else {
            (row * BOARD_SIZE + col) as usize
        }
    }

    pub fn checkmask_at(&self, row: u8, col: u8) -> u32 {
        if self.1 {
            self.0.transposed_checks[self.idx(row, col)]
        } else {
            self.0.primary_checks[self.idx(row, col)]
        }
    }

    pub fn to_string(&self) -> String {
        let mut res: String = String::new();
        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if let Some(letter) = self.letter_at(row, col) {
                    res.push(letter.to_char());
                } else {
                    res.push(' ');
                }
            }
            res.push('\n');
        }

        res
    }
}
