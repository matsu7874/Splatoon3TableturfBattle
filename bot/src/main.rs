use rand::prelude::*;
use std::io;

use tableturfbattle::{
    parse_input, read_line,
    text_protocol::{read_hands, read_initial_input},
    CardId, FieldShape,
};

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

const STARTER_DECK: [CardId; 15] = [
    6, 13, 22, 28, 40, 34, 45, 52, 55, 56, 159, 137, 141, 103, 92,
];
fn main() {
    let mut rng = rand::thread_rng();
    println!("matsu784_bot"); //bot名

    let initial_input = read_initial_input();
    // スターターデッキ
    let deck = if STARTER_DECK
        .iter()
        .all(|card_id| initial_input.cards.iter().any(|card| card.id == *card_id))
    {
        Vec::from(STARTER_DECK)
    } else {
        let mut deck = vec![];
        if initial_input.is_deplicated_pick_enabled {
            for _ in 0..initial_input.deck_size {
                deck.push(initial_input.cards[rng.gen_range(0..initial_input.cards.len())].id);
            }
        } else {
            deck = initial_input
                .cards
                .iter()
                .take(initial_input.deck_size)
                .map(|card| card.id)
                .collect::<Vec<CardId>>();
        }
        deck
    };
    // 全てのカードがカードカタログに存在することを検証
    assert!(deck
        .iter()
        .all(|card_id| initial_input.cards.iter().any(|card| card.id == *card_id)));
    assert_eq!(deck.len(), initial_input.deck_size);
    // デッキを出力
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

        // TODO: 行動を実装（ランダムにこうどうしている）
        let action_index = rng.gen_range(0..turn_input.valid_actions.len());
        println!("{}", turn_input.valid_actions[action_index]);
        // PASS {card_id}
        // PUT {card_id} {dir} {y} {x}
        // SPECIAL {card_id} {dir} {y} {x}

        if turn_input.turn == initial_input.max_turn {
            break;
        }
    }
}
