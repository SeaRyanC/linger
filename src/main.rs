mod dawg;

fn main() {
    let mut empty = dawg::Dawg::make_fillable(256);
    println!("printing empty content");
    empty.print_contents();
    let mut next_node = 0;

    println!("inserting 'car' and 'hat'");
    next_node = empty.insert("CAR", next_node);
    next_node = empty.insert("HAT", next_node);
    println!("next node ptr is now {}", next_node);

    println!("printing contents");
    empty.print_contents();

    println!("inserting 'cat'");
    empty.insert("CAT", next_node);

    // println!("inserting 'cap'");
    // empty.insert("CAP", next_node);

    empty.insert("CAZ", next_node);

    println!("printing contents");
    empty.print_contents();
}
