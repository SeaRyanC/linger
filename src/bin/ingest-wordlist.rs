use linger::dawg::DynamicDawg;

fn main() -> () {
    let args: Vec<String> = std::env::args().collect();
    let path = &args.get(1).expect("No input file provided");
    let content = std::fs::read_to_string(path).expect("Couldn't read file");
    let mut word_list: Vec<&str> = content.lines().filter(|w| -> bool { w.len() != 0 }).collect();
    word_list.sort();
    println!("Read {} words from {}", word_list.len(), path);

    let mut empty: DynamicDawg = DynamicDawg::make_fillable(content.len());
    for line in &word_list {
        empty.insert(line);
    }
    println!("Created {} raw nodes", empty.node_count());

    let uncanonicalized = empty.dawg.get_strings();
    assert_eq!(word_list, uncanonicalized);
    let canon_count = empty.canonicalize();
    let canonicalized = empty.dawg.get_strings();
    assert_eq!(word_list, canonicalized);
    println!("Successfully canonicalized to {} nodes", canon_count);

    let min = empty.minimize();
    let minimized = min.get_strings();
    assert_eq!(word_list, minimized);
    let node_count = min.0.len();
    println!("Successfully minimzed to {} bytes", node_count * 4);

    if let Some(output) = args.get(2) {
        let bytes: Vec<u8> = min.0.iter().flat_map(|b| b.to_le_bytes()).collect();
        std::fs::write(output, bytes).expect("Failed to write output");
    }
}
