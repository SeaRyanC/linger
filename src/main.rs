use linger::{board::*, player_move::*, letter::*};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;

fn main() { 
    let board = Board::new();
    let view = BoardView(&board, true);
    println!("{}", view.to_string());

    let mv: PlayMove = PlayMove { row: 7, col: 7, transposed: false, letters: vec![Letter::from_char('A'), Letter::from_char('T')] };
    let board = board.with_move(PlayerMove::Play(mv));
    let view = BoardView(&board, false);
    println!("{}", view.to_string());

    let mut tile = linger::tile_bag::TileBag::new_standard();
    let mut rng = ChaCha12Rng::from_seed(Default::default());
    for n in 0..7 {
        println!("{:?}", tile.draw_tile(&mut rng).letter.expect("tile must exist").to_char());
    }
}
