use std::collections::HashMap;
use std::io;

use serde::{Deserialize, Serialize};

use tableturfbattle::{Action, Card, CardShape, Direction, Environment, Field, State};
macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

const YELLOW: usize = 0;
const BLUE: usize = 1;

#[derive(Serialize, Deserialize)]
struct CardJson {
    id: usize,
    name: String,
    cost: usize,
    cells: String,
}
impl CardJson {
    fn to_card(&self) -> Card {
        let seed = CardShape::new(&self.cells);
        let shape = CardShape::trim(&seed);
        let power = shape.count_colored_squares();
        Card {
            id: self.id,
            name: self.name.clone(),
            cost: self.cost,
            power: power,
            shape: shape,
        }
    }
}
fn load_card_catalog() -> serde_json::Result<Vec<Card>> {
    let text = std::fs::read_to_string("../card_catalog.json").unwrap();
    let raw_cards = serde_json::from_str::<Vec<CardJson>>(&text).unwrap();
    let mut cards = vec![];
    for c in raw_cards {
        cards.push(c.to_card());
    }
    serde_json::Result::Ok(cards)
}
fn main() {
    // 2つのプログラムと情報の受け渡しを行いゲームを進めるプログラム

    let env = Environment::new(15, 4, 12);
    let cards = load_card_catalog().expect("JSON読み込みはうまくいく");

    let field = Field::default();
    let field_info = format!("{} {}", field.shape.height, field.shape.width);
    println!("{}", field_info);
    for row in field.shape.squares.iter() {
        println!("{}", String::from_iter(row.iter().map(|c| { c.to_char() })));
    }

    let mut card_catalog = HashMap::new();
    println!("{}", cards.len());
    for card in cards {
        println!(
            "{} {} {} {}",
            card.id, card.cost, card.shape.height, card.shape.width
        );
        for row in card.shape.squares.iter() {
            println!("{}", String::from_iter(row.iter().map(|c| { c.to_char() })));
        }
        card_catalog.insert(card.id, card);
    }

    // TODO: botから入力を受け取る
    let yellow_deck = vec![
        6, 13, 22, 28, 40, 34, 45, 52, 55, 56, 159, 137, 141, 103, 92,
    ]; //仮にスターターデッキを渡す。
    let blue_deck = vec![1usize; env.deck_size];

    let mut state = State::new(
        &env,
        &card_catalog,
        &field.clone(),
        &yellow_deck,
        &blue_deck,
    );
    while !state.is_done(&env) {
        println!(
            "{} {} {}",
            state.turn, state.players[YELLOW].special_point, state.players[BLUE].special_point
        );
        for row in state.field.squares.iter() {
            println!("{}", String::from_iter(row.iter().map(|c| { c.to_char() })));
        }
        println!("{:?}", state.players[YELLOW].hands);
        println!("{:?}", state.players[BLUE].hands);

        // TODO: botから入力を受け取る
        let yellow_candidates = state.generate_valid_actions(&card_catalog, 0);
        let yellow_action = yellow_candidates[0];
        let blue_candidates = state.generate_valid_actions(&card_catalog, 1);
        let blue_action = blue_candidates[0];
        state.apply(&env, &card_catalog, &vec![yellow_action, blue_action]);
    }

    if state.is_win(&env, 0) {
        println!("Player0(Yellow) won");
    } else if state.is_win(&env, 0) {
        println!("Player1(Blue) won");
    } else {
        println!("DRAW");
    }
    println!(
        "{} : {}",
        state.field.count_player(0),
        state.field.count_player(1),
    );
}
