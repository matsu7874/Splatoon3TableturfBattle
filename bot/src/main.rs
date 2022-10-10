use std::io;
macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

fn main() {
    println!("my name");
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap(); // "y x"の形
    let field_size_y = parse_input!(input_line, usize);
    let field_size_x = parse_input!(input_line, usize);
    let mut field = vec![];
    for i in 0..field_size_y {
        let mut row = vec![];
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap(); // "y x"の形
        let field_size_y = parse_input!(input_line, usize);
        let field_size_x = parse_input!(input_line, usize);
    }

    // READ LIST OF CARDS
    println!("list of cards");
    // スターターデッキ
    let deck = [
        6, 13, 22, 28, 40, 34, 45, 52, 55, 56, 159, 137, 141, 103, 92,
    ];
    const MAX_TURN: usize = 12;
    for turn in 0..MAX_TURN {
        // read game
        // read field
        // read hands
        let hands = vec![0];
        println!("PASS {}", hands[0]);
        // PASS {card_id}
        // PUT {card_id} {dir} {y} {x}
        // SPECIAL {card_id} {dir} {y} {x}
    }
}
