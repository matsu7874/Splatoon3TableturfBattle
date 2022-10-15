use rand::prelude::*;
use std::io;

use tableturfbattle::{Card, CardId, CardShape, FieldShape};
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
macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

struct InitialInput {
    deck_size: usize,
    hand_size: usize,
    max_turn: usize,
    field_size_y: usize,
    field_size_x: usize,
    field: FieldShape,
    cards: Vec<Card>,
}
fn read_initial_input() -> InitialInput {
    let chunks = read_line!();
    let deck_size = parse_input!(chunks[0], usize);
    let hand_size = parse_input!(chunks[1], usize);
    let max_turn = parse_input!(chunks[2], usize);

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
        deck_size,
        hand_size,
        max_turn,
        field_size_y,
        field_size_x,
        field,
        cards,
    }
}

fn read_hands() -> Vec<usize> {
    let chunks = read_line!();
    let hands = chunks
        .iter()
        .map(|s| s.parse::<usize>().unwrap())
        .collect::<Vec<usize>>();
    hands
}

struct TurnInput {
    turn: usize,
    special_points: Vec<usize>,
    field: FieldShape,
    hands: Vec<CardId>,
    valid_actions: Vec<String>, // TODO: Stringを解釈する
}
fn read_turn_input(field_size_y: usize) -> TurnInput {
    let chunks = read_line!();
    let turn = parse_input!(chunks[0], usize);

    let chunks = read_line!();
    let special_points = chunks
        .iter()
        .map(|s| s.parse::<usize>().unwrap())
        .collect::<Vec<usize>>();

    let mut rows = vec![];
    for _ in 0..field_size_y {
        let chunks = read_line!();
        rows.push(chunks.join(""))
    }
    let field = FieldShape::new(&rows.join("\n"));

    let hands = read_hands();
    let chunks = read_line!();
    let n_actions = parse_input!(chunks[0], usize);
    let mut valid_actions = vec![];
    for _ in 0..n_actions {
        let chunks = read_line!();
        valid_actions.push(chunks.join(" "));
    }
    TurnInput {
        turn,
        special_points,
        field,
        hands,
        valid_actions,
    }
}

fn main() {
    let mut rng = rand::thread_rng();

    let initial_input = read_initial_input();
    // スターターデッキ
    let deck = [
        6, 13, 22, 28, 40, 34, 45, 52, 55, 56, 159, 137, 141, 103, 92,
    ];
    assert!(deck
        .iter()
        .all(|card_id| initial_input.cards.iter().any(|card| card.id == *card_id)));
    assert_eq!(deck.len(), initial_input.deck_size);
    println!(
        "{}",
        deck.iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(" ")
    );

    // マリガン判定
    let hands = read_hands();
    if hands.contains(&deck[0]) {
        println!("PASS");
    } else {
        println!("MULLIGAN");
    }

    loop {
        let turn_input = read_turn_input(initial_input.field_size_y);
        assert_eq!(turn_input.hands.len(), initial_input.hand_size);
        let action_index = rng.gen_range(0..turn_input.valid_actions.len());
        // TODO: 行動を実装
        println!("{}", turn_input.valid_actions[action_index]);
        // PASS {card_id}
        // PUT {card_id} {dir} {y} {x}
        // SPECIAL {card_id} {dir} {y} {x}

        if turn_input.turn == initial_input.max_turn {
            break;
        }
    }
}
