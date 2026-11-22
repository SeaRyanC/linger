use crate::{letter::Letter, tile::Tile};
use std::sync::OnceLock;
use regex::Regex;

pub type TileDistrib = Vec<TileDistribRow>;

pub struct TileDistribRow {
    pub letter: Option<Letter>,
    pub value: u8,
    pub count: u8
}

const DISTRIB: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/tiles.txt"));

static STANDARD: OnceLock<TileDistrib> = OnceLock::new();
pub fn standard() -> &'static TileDistrib {
    STANDARD.get_or_init(|| compute_standard())
}

fn compute_standard() -> TileDistrib {
    let mut tile_list = DISTRIB.lines().filter(|w| -> bool { w.len() != 0 });
    let re = Regex::new(r"([A-Z?]) (\d+) (\d+)").unwrap();
    let mut res: TileDistrib = TileDistrib::new();
    for line in tile_list {
        if let Some(captures) = re.captures(line) {
            let char = captures[1].chars().nth(0).expect("Must have a char");
            let value: u8 = captures[2].parse().expect("Must have a value");
            let count: u8 = captures[3].parse().expect("Must have a quantity");
            let letter = if char == '?' {
                None
            } else {
                Option::Some(Letter::from_char(char))
            };
            res.push(TileDistribRow { letter, value, count })
        } else {
            panic!("Malformed line in tile list {}", line);
        }
    }
    res
}