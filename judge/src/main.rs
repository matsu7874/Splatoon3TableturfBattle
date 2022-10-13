use log::{debug, info};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use tableturfbattle::{Action, Card, CardShape, Environment, Field, State};

#[derive(Serialize, Deserialize)]
struct CardJson {
    id: usize,
    name: String,
    cost: usize,
    squares: String,
}
impl CardJson {
    fn to_card(&self) -> Card {
        let seed = CardShape::new(&self.squares);
        let shape = CardShape::trim(&seed);
        let power = shape.count_colored_squares();
        Card {
            id: self.id,
            name: self.name.clone(),
            cost: self.cost,
            power,
            shape,
        }
    }
}
fn load_card_catalog() -> serde_json::Result<Vec<Card>> {
    let text = std::fs::read_to_string("card_catalog.json").unwrap();
    let raw_cards = serde_json::from_str::<Vec<CardJson>>(&text).unwrap();
    let mut cards = vec![];
    for c in raw_cards {
        cards.push(c.to_card());
    }
    serde_json::Result::Ok(cards)
}

struct GameInfo {
    winner: Option<usize>,
    n_squares: Vec<usize>,
}
fn exec_game(env: &Environment, cards: &[Card], field: &Field, commands: Vec<String>) -> GameInfo {
    let mut bot_processes = vec![];
    for command in commands.iter() {
        let bot_process = match Command::new(command.clone())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
        {
            Err(why) => panic!("couldn't spawn bot: {}", why),
            Ok(process) => process,
        };
        bot_processes.push(bot_process);
    }

    let mut rng = rand::thread_rng();

    let cards_info = cards
        .iter()
        .map(|card| {
            format!(
                "{} {} {} {}\n{}",
                card.id, card.cost, card.shape.height, card.shape.width, card.shape
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let initial_input = format!(
        "{deck_size} {hand_size} {max_turn}\n{stage_size_y} {stage_size_x}\n{stage_shape}\n{number_of_cards}\n{cards_info}\n",
        deck_size = env.deck_size,
        hand_size = env.hand_size,
        max_turn = env.max_turn,
        stage_size_y = field.shape.height,
        stage_size_x = field.shape.width,
        stage_shape = field.shape,
        number_of_cards = cards.len(),
        cards_info = cards_info
    );
    for bot_process in bot_processes.iter() {
        let mut stdin = bot_process.stdin.as_ref().unwrap();

        if let Err(why) = stdin.write_all(initial_input.as_bytes()) {
            panic!("couldn't write to bot stdin: {}", why);
        }
        stdin.flush().unwrap_or(());
    }

    let mut card_catalog = HashMap::new();
    for card in cards {
        card_catalog.insert(card.id, card);
    }

    let mut decks = vec![];
    for (player_id, bot_process) in bot_processes.iter_mut().enumerate() {
        let stdout = bot_process.stdout.as_mut().expect("");
        let mut reader = BufReader::new(stdout);
        let mut s = String::new();
        match reader.read_line(&mut s) {
            Err(why) => panic!("couldn't read bot stdout: {}", why),
            Ok(_) => debug!("bot {} deck: {}", player_id, s),
        };
        let mut deck: Vec<usize> = s
            .trim()
            .split(' ')
            .filter(|s| !s.is_empty())
            .map(|s| {
                s.parse::<usize>()
                    .expect("botを完全に信用する。TODO:ここは入力の検証が必要。")
            })
            .collect();
        deck.shuffle(&mut rng);
        decks.push(deck);
    }

    // 毎ターンの繰り返し処理
    let mut state = State::new(env, &card_catalog, field, &decks[0], &decks[1]);
    while !state.is_done(env) {
        let mut actions = vec![];

        for (player_id, bot_process) in bot_processes.iter_mut().enumerate() {
            let action_candidates = state.generate_valid_actions(&card_catalog, player_id);
            debug!(
                "turn:{}, player_id:{}, n_action:{}",
                state.turn,
                player_id,
                action_candidates.len()
            );
            let turn_input = format!(
                "{turn}\n{special_points}\n{stage_shape}\n{hands}\n{n_action}\n{action_candidates}\n",
                turn = state.turn,
                special_points = state
                    .players
                    .iter()
                    .map(|p| p.special_point.to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
                stage_shape = field.shape,
                hands = state.players[player_id]
                    .hands
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
                    n_action = action_candidates.len(),
                    action_candidates = action_candidates.iter().map(|a| a.to_string()).collect::<Vec<String>>().join("\n")
                );
            let mut stdin = bot_process.stdin.as_ref().unwrap();

            if let Err(why) = stdin.write_all(turn_input.as_bytes()) {
                panic!("couldn't write to bot stdin: {}", why);
            }
            stdin.flush().unwrap_or(());

            // TODO: botから入力を受け取る
            let stdout = bot_process.stdout.as_mut().expect("");
            let mut reader = BufReader::new(stdout);
            let mut s = String::new();
            match reader.read_line(&mut s) {
                Err(why) => panic!("couldn't read bot stdout: {}", why),
                Ok(_) => debug!("bot {} action: {}", player_id, s),
            };
            let action = Action::from(s.as_ref());
            actions.push(action);
        }

        state.apply(env, &card_catalog, &actions);
    }
    drop(bot_processes);

    let winner = if state.is_win(env, 0) {
        info!("Player0(Yellow) won");
        Some(0)
    } else if state.is_win(env, 1) {
        info!("Player1(Blue) won");
        Some(1)
    } else {
        info!("DRAW");
        None
    };
    info!(
        "{} : {}",
        state.field.count_player(0),
        state.field.count_player(1),
    );
    info!("\n{}", state.field.to_string());

    GameInfo {
        winner,
        n_squares: vec![state.field.count_player(0), state.field.count_player(1)],
    }
}
fn main() {
    // 2つのプログラムと情報の受け渡しを行いゲームを進めるプログラム
    env_logger::init();

    let env = Environment::new(15, 4, 12);
    let cards = load_card_catalog().expect("JSON読み込みはうまくいく");
    // TODO: 別のフィールドも使えるようにする
    let field = Field::default();
    let mut total_wins = vec![0; 2];
    for i in 0..3 {
        let result = exec_game(
            &env,
            &cards,
            &field,
            vec![
                "target/release/bot".to_string(),
                "target/release/bot".to_string(),
            ],
        );
        if let Some(winner) = result.winner {
            total_wins[winner] += 1;
            info!(
                "game:{}\twinner:{}\tplayer0_square:{}\tplayer1_square:{}",
                i, winner, result.n_squares[0], result.n_squares[1]
            );
        } else {
            info!(
                "game:{}\twinner:none\tplayer0_square:{}\tplayer1_square:{}",
                i, result.n_squares[0], result.n_squares[1]
            );
        }
    }
    println!("\ntotal_wins: {:?}", total_wins);
}
