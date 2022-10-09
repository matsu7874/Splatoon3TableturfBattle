use std::collections::HashMap;
use std::io;
use tableturfbattle::{Action, Card, Direction, Environment, Field, State};
macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

const YELLOW: usize = 0;
const BLUE: usize = 1;
fn main() {
    // 2つのプログラムと情報の受け渡しを行いゲームを進めるプログラム

    let env = Environment::new(15, 4, 12);

    let field = Field::default();
    let field_info = format!("{} {}", field.shape.height, field.shape.width);
    println!("{}", field_info);
    for row in field.shape.squares.iter() {
        println!("{}", String::from_iter(row.iter().map(|c| { c.to_char() })));
    }

    let mut card_catalog = HashMap::new();
    //TODO: たくさんの種類のカードを使いたい
    let cards = vec![Card::default()];
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
    let yellow_deck = vec![1usize; env.deck_size];
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
}
