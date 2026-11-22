use crate::{board::BoardView, dawg::Dawg, player_move::PlayMove, tile::Tile};


pub fn generate_play_moves() {

}

struct GenMoveContext<'a> {
    output: &'a mut Vec<PlayMove>,
    so_far: Vec<&'a Tile>,
    rack: &'a Vec<Tile>,
    view: &'a BoardView<'a>,
    dict: &'a Dawg,
    dawg_node: usize,
    rack_used_mask: u32,
    row: u8,
    col: u8,
    min_length: u8
}

fn gen_moves_at(ctx: &mut GenMoveContext) {
    // i = index of the tile we are trying to place from the player's rack
    for i in 0..ctx.rack.len() {
        if ctx.rack_used_mask & (1 << i) != 0 {
            // higher stack frame used this tile already
            continue;
        }
        let tile = &ctx.rack[i];
        let mask = tile.letter.expect("blank nyi").mask();
        if mask & ctx.view.checkmask_at(ctx.row, ctx.col) == 0 {
            // The crosscheck disallows this placement
            continue;
        }
        if mask & ctx.dict.0[ctx.dawg_node] == 0 {
            // There is no outbound dawg node from here
            continue;
        }
        ctx.so_far.push(tile);

        // If this is a valid end of word, we can emit it
        // TODO: only valid if next cell is empty or this is
        // end-of-word, plus we must meet minLength reqs
        if ctx.dict.0[ctx.dawg_node] & 1 == 1 {
            let mut s = String::new();
            for tile in &ctx.so_far {
                s.push(tile.letter.expect("??").to_char());
            }
            println!("{}", s);
        }

        // TODO: Handle fast-forwarding 'col' through placed tiles

        let old_dawg = ctx.dawg_node;
        // Presumptively valid tile placement, add it and recurse
        ctx.rack_used_mask |= 1 << i;
        ctx.col += 1;
        ctx.dawg_node = ctx.dict.read_target(ctx.dawg_node, tile.letter.expect("blank nyi"));
        gen_moves_at(ctx);
        ctx.dawg_node = old_dawg;
        ctx.col -= 1;
        ctx.rack_used_mask ^= 1 << i;
        ctx.so_far.pop();

    }
}

#[cfg(test)]
mod tests {
    use crate::{board::Board, player_state::PlayerState, tile_bag::TileBag};

use super::*;

    #[test]
    fn minimal_dawg_test() {
        let bytes_raw = include_bytes!("../data/nwl23.bin");
        let mut bytes: Vec<u8> = vec![];
        for b in bytes_raw {
            bytes.push(*b);
        }
        let dawg = Dawg::from_bytes(&bytes);
        let mut bag = TileBag::new_standard();
        let mut ps = PlayerState {
            score: 0,
            rack: vec![]
        };
        ps.rack.push(bag.fish('T'));
        ps.rack.push(bag.fish('O'));
        ps.rack.push(bag.fish('P'));
        ps.rack.push(bag.fish('S'));

        let board: Board = Board::new();
        let mut output: Vec<PlayMove> = vec![];

        let mut ctx = GenMoveContext {
            so_far: vec![],
            output: &mut output,
            rack: &ps.rack,
            view: &BoardView(&board, false),
            dict: &dawg,
            dawg_node: 0,
            rack_used_mask: 0,
            row: 7,
            col: 7,
            min_length: 2
        };
        gen_moves_at(&mut ctx);
    }
}
