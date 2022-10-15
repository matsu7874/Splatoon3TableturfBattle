use log::{debug, info};
use rand::{rngs::ThreadRng, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::{collections::HashMap, process::Child};
use std::{io::prelude::*, path::Path};
use tableturfbattle::{
    Action, Card, CardId, CardShape, Environment, Field, MulliganAction, PlayerId, State,
};

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
        Card::new(self.id, &self.name, self.cost, shape)
    }
}
fn load_card_catalog(card_catalog_path: &str) -> serde_json::Result<Vec<Card>> {
    let text = std::fs::read_to_string(card_catalog_path).unwrap();
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
fn lunch_bots(commands: &[&str]) -> Vec<Child> {
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
    bot_processes
}
fn read_names(bot_processes: &mut Vec<Child>) -> Vec<String> {
    let mut names = vec![];
    for (player_id, bot_process) in bot_processes.iter_mut().enumerate() {
        let stdout = bot_process.stdout.as_mut().expect("");
        let mut reader = BufReader::new(stdout);
        let mut s = String::new();
        match reader.read_line(&mut s) {
            Err(why) => panic!("couldn't read player stdout: {}", why),
            Ok(_) => debug!("player {} name: {}", player_id, s),
        };
        let name = s.trim().to_string();
        names.push(name);
    }
    names
}
fn format_initial_input(env: &Environment, cards: &[Card], field: &Field) -> String {
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

    format!(
        "{player_size} {deck_size} {hand_size} {max_turn} {is_deplicated_pick_enabled}\n{stage_size_y} {stage_size_x}\n{stage_shape}\n{number_of_cards}\n{cards_info}\n",
        player_size = env.player_size,
        deck_size = env.deck_size,
        hand_size = env.hand_size,
        max_turn = env.max_turn,
        is_deplicated_pick_enabled = if env.is_deplicated_pick_enabled{"1"}else{"0"},
        stage_size_y = field.shape.height,
        stage_size_x = field.shape.width,
        stage_shape = field.shape,
        number_of_cards = cards.len(),
        cards_info = cards_info
    )
}
fn print_initial_input(bot_processes: &mut Vec<Child>, initial_input: &str) {
    for bot_process in bot_processes.iter() {
        let mut stdin = bot_process.stdin.as_ref().unwrap();

        if let Err(why) = stdin.write_all(initial_input.as_bytes()) {
            panic!("couldn't write to player stdin: {}", why);
        }
        stdin.flush().unwrap_or(());
    }
    print!("{}", initial_input); //末尾に改行が含まれる文字列であるため
}
fn read_decks(bot_processes: &mut Vec<Child>) -> Vec<Vec<usize>> {
    let mut decks = vec![];
    for (player_id, bot_process) in bot_processes.iter_mut().enumerate() {
        let stdout = bot_process.stdout.as_mut().expect("");
        let mut reader = BufReader::new(stdout);
        let mut s = String::new();
        match reader.read_line(&mut s) {
            Err(why) => panic!("couldn't read player stdout: {}", why),
            Ok(_) => debug!("player {} deck: {}", player_id, s),
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
        println!(
            "{}",
            deck.iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        );
        decks.push(deck);
    }
    decks
}
fn shuffle_and_mulligan(
    env: &Environment,
    bot_processes: &mut Vec<Child>,
    decks: &Vec<Vec<usize>>,
    rng: &mut ThreadRng,
) -> Vec<Vec<usize>> {
    let mut shuffled_decks = vec![];
    for (player_id, bot_process) in bot_processes.iter_mut().enumerate() {
        let mut deck = decks[player_id].clone();
        deck.shuffle(rng);
        // デッキの順番を記録する
        println!(
            "{}",
            deck.iter()
                .map(|card_id| { card_id.to_string() })
                .collect::<Vec<String>>()
                .join(" ")
        );
        shuffled_decks.push(deck);

        let hands = format!(
            "{}\n",
            shuffled_decks[player_id][0..env.hand_size]
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(" "),
        );
        let mut stdin = bot_process.stdin.as_ref().unwrap();
        if let Err(why) = stdin.write_all(hands.as_bytes()) {
            panic!("couldn't write to player stdin: {}", why);
        }
        stdin.flush().unwrap_or(());

        let stdout = bot_process.stdout.as_mut().expect("");
        let mut reader = BufReader::new(stdout);
        let mut s = String::new();
        match reader.read_line(&mut s) {
            Err(why) => panic!("couldn't read player stdout: {}", why),
            Ok(_) => debug!("player {} mulligan: {}", player_id, s),
        };
        let action = MulliganAction::from(s.as_str());
        println!("{}", action);
        if action == MulliganAction::Mulligan {
            debug!("player {} mulliganed", player_id);
            shuffled_decks[player_id].shuffle(rng);
        }
    }
    shuffled_decks
}
fn format_turn_input(
    field: &Field,
    card_catalog: &HashMap<usize, &Card>,
    state: &mut State,
    player_id: usize,
) -> String {
    let action_candidates = state.generate_valid_actions(card_catalog, player_id);
    debug!(
        "turn:{}, player_id:{}, n_action:{}",
        state.turn,
        player_id,
        action_candidates.len()
    );
    format!(
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
        action_candidates = action_candidates
            .iter()
            .map(|a| a.to_string())
            .collect::<Vec<String>>()
            .join("\n")
    )
}
fn read_action(bot_process: &mut Child, player_id: usize) -> Action {
    let stdout = bot_process.stdout.as_mut().expect("");
    let mut reader = BufReader::new(stdout);
    let mut s = String::new();
    match reader.read_line(&mut s) {
        Err(why) => panic!("couldn't read player stdout: {}", why),
        Ok(_) => debug!("player {} action: {}", player_id, s),
    };
    Action::from(s.as_ref())
}
fn game_loop(
    env: &Environment,
    field: &Field,
    card_catalog: &HashMap<usize, &Card>,
    bot_processes: &mut Vec<Child>,
    state: &mut State,
) {
    let mut actions = vec![];

    for (player_id, bot_process) in bot_processes.iter_mut().enumerate() {
        let turn_input = format_turn_input(field, card_catalog, state, player_id);

        let mut stdin = bot_process.stdin.as_ref().unwrap();
        if let Err(why) = stdin.write_all(turn_input.as_bytes()) {
            panic!("couldn't write to player stdin: {}", why);
        }
        stdin.flush().unwrap_or(());

        let action = read_action(bot_process, player_id);
        println!("{}", action);
        actions.push(action);
    }

    state.apply(env, card_catalog, &actions);
}

fn exec_game(env: &Environment, cards: &[Card], field: &Field, commands: &[&str]) -> GameInfo {
    let mut rng = rand::thread_rng();
    let mut bot_processes = lunch_bots(commands);
    let names = read_names(&mut bot_processes);

    let initial_input = format_initial_input(env, cards, field);
    print_initial_input(&mut bot_processes, &initial_input);

    let mut card_catalog = HashMap::new();
    for card in cards {
        card_catalog.insert(card.id, card);
    }

    println!("{}", names.join("\n"));

    let decks = read_decks(&mut bot_processes);
    let shuffled_decks = shuffle_and_mulligan(env, &mut bot_processes, &decks, &mut rng);
    // デッキの順番を記録する
    for deck in shuffled_decks.iter() {
        println!(
            "{}",
            deck.iter()
                .map(|card_id| { card_id.to_string() })
                .collect::<Vec<String>>()
                .join(" ")
        )
    }

    let mut state = State::new(env, &card_catalog, field, &shuffled_decks);
    // 毎ターンの繰り返し処理
    while !state.is_done(env) {
        game_loop(env, field, &card_catalog, &mut bot_processes, &mut state)
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

    // let env = Environment::new(2, 15, 4, 12, false);
    let env = Environment::new(2, 20, 4, 17, true);
    let cards =
        load_card_catalog("resources/card_catalog_sample.json").expect("JSON読み込みはうまくいく");
    // TODO: 別のフィールドも使えるようにする
    let field = Field::default();
    let result = exec_game(
        &env,
        &cards,
        &field,
        &["target/release/bot", "target/release/bot"],
    );
    if let Some(winner) = result.winner {
        info!(
            "winner:{}\tplayer0_square:{}\tplayer1_square:{}",
            winner, result.n_squares[0], result.n_squares[1]
        );
    } else {
        info!(
            "winner:none\tplayer0_square:{}\tplayer1_square:{}",
            result.n_squares[0], result.n_squares[1]
        );
    }
}
