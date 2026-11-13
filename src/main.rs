mod letter;
mod dawg;
use std::time::Instant;

const SAMPLE_WORDS: &str = include_str!("../data/words-2974.txt");

fn main() {

    let mut empty = dawg::DynamicDawg::make_fillable(SAMPLE_WORDS.len());
    println!("printing empty content");
    empty.dawg.print_contents();

    let st = Instant::now();
    for line in SAMPLE_WORDS.lines() {
        if line.len() == 0{
            continue;
        }
        empty.insert(line);
    }
    println!("inserted words in {:?}", st.elapsed());

    let st = Instant::now();
    empty.canonicalize();
    println!("canonicalized in {:?}", st.elapsed());

    let st = Instant::now();
    let min = empty.minimize();
    println!("minimized in {:?}", st.elapsed());
    println!("{}",min.0[0]);
}
