use crate::{tile::Tile, tile_distrib::TileDistrib};
use crate::letter::Letter;
use rand::{Rng, RngExt};

pub struct TileBag {
    tiles: Vec<Tile>,
    definition: &'static TileDistrib
}

impl TileBag {
    pub fn new_standard() -> TileBag {
        let mut tiles: Vec<Tile> = vec![];
        let definition = crate::tile_distrib::standard();
        for def in definition {
            for _ in 0..def.count {
                tiles.push(Tile { letter: def.letter, value: def.value });
            }
        }
        TileBag { tiles, definition }
    }

    pub fn draw_tile(&mut self, rnd: &mut impl Rng) -> Tile {
        let n = rnd.random_range(0..self.tiles.len());
        self.tiles.swap_remove(n)
    }

    pub fn add_tile(&mut self, tile: Tile) {
        self.tiles.push(tile);
    }

    pub fn fish(&mut self, c: char) -> Tile {
        for i in 0..self.tiles.len() {
            if let Some(tile) = self.tiles[i].letter {
                if tile.to_char() == c {
                    return self.tiles.swap_remove(i);
                }
            }
        }
        // false error from rust-analyer when there
        // isn't a 'return' here
        return panic!("{} is not in the bag", c);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_hundred_tiles() {
        let bag = TileBag::new_standard();
        assert_eq!(100, bag.tiles.len());
    }
}
