use crate::{Card, CardShape, FieldShape};
use std::io;

#[macro_export]
macro_rules! read_line {
    () => {{
        let mut buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("標準入力から読める"); // 1行読み込み
        buff.split("\n")
            .map(|s| s.split(" ").map(|s| s.to_string()).collect::<Vec<String>>())
            .flatten()
            .filter(|s| s.len() > 0)
            .collect::<Vec<String>>()
    }};
}
#[macro_export]
macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

pub struct InitialInput {
    pub player_size: usize,
    pub deck_size: usize,
    pub hand_size: usize,
    pub max_turn: usize,
    pub is_deplicated_pick_enabled: bool,
    pub field_size_y: usize,
    pub field_size_x: usize,
    pub field: FieldShape,
    pub cards: Vec<Card>,
}
pub fn read_initial_input() -> InitialInput {
    let chunks = read_line!();
    let player_size = parse_input!(chunks[0], usize);
    let deck_size = parse_input!(chunks[1], usize);
    let hand_size = parse_input!(chunks[2], usize);
    let max_turn = parse_input!(chunks[3], usize);
    let is_deplicated_pick_enabled = parse_input!(chunks[4], usize);

    let chunks = read_line!();
    let field_size_y = parse_input!(chunks[0], usize);
    let field_size_x = parse_input!(chunks[1], usize);

    let mut rows = vec![];
    for _ in 0..field_size_y {
        let chunks = read_line!();
        rows.push(chunks.join(""))
    }
    assert!(rows.iter().all(|row| row.len() == field_size_x));
    let field = FieldShape::new(&rows.join("\n"));

    let chunks = read_line!();
    let n_cards = parse_input!(chunks[0], usize);
    let mut cards = vec![];
    for _ in 0..n_cards {
        let chunks = read_line!();
        let card_id = parse_input!(chunks[0], usize);
        let card_cost = parse_input!(chunks[1], usize);
        let card_size_y = parse_input!(chunks[2], usize);
        let _card_size_x = parse_input!(chunks[3], usize);
        let mut rows = vec![];
        for _ in 0..card_size_y {
            let chunks = read_line!();
            rows.push(chunks.join(""))
        }
        let card_shape = CardShape::new(&rows.join("\n"));
        cards.push(Card {
            id: card_id,
            name: "".to_string(),
            cost: card_cost,
            power: card_shape.count_colored_squares(),
            shape: card_shape,
        })
    }

    InitialInput {
        player_size,
        deck_size,
        hand_size,
        max_turn,
        is_deplicated_pick_enabled: is_deplicated_pick_enabled == 1,
        field_size_y,
        field_size_x,
        field,
        cards,
    }
}

pub fn read_hands() -> Vec<usize> {
    let chunks = read_line!();
    let hands = chunks
        .iter()
        .map(|s| s.parse::<usize>().unwrap())
        .collect::<Vec<usize>>();
    hands
}
