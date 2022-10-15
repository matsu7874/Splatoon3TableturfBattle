use std::collections::HashMap;
use std::io;
use svg::node::{element::Rectangle, Text};
use tableturfbattle::{
    parse_input, read_line,
    Action, Card, CardId, CardShape, CardSquareType, Environment, Field, FieldShape,
    FieldSquareType, MulliganAction, State,
};
fn main() {
    let chunks = read_line!();
    let env = Environment {
        player_size: parse_input!(chunks[0], usize),
        deck_size: parse_input!(chunks[1], usize),
        hand_size: parse_input!(chunks[2], usize),
        max_turn: parse_input!(chunks[3], usize),
        is_deplicated_pick_enabled: parse_input!(chunks[4], usize) == 1,
    };
    let chunks = read_line!();
    let field_y = parse_input!(chunks[0], usize);
    let _field_x = parse_input!(chunks[1], usize);
    let mut rows = vec![];
    for _ in 0..field_y {
        rows.push(read_line!()[0].to_owned());
    }

    let field = Field {
        id: 1,
        name: "unknown".to_string(),
        shape: FieldShape::new(&rows.join("\n")),
    };

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
            rows.push(read_line!()[0].to_owned());
        }
        let shape = CardShape::new(&rows.join("\n"));
        cards.push(Card::new(card_id, "unknown", card_cost, shape));
    }

    let mut player_names = vec![];
    for _ in 0..env.player_size {
        let chunks = read_line!();
        player_names.push(chunks[0].to_owned());
    }

    // selected deck
    for _ in 0..env.player_size {
        let chunks = read_line!();
        // player_names.push(chunks[0].to_owned());
    }
    let mut decks = vec![];
    for _ in 0..env.player_size {
        // shuffled deck
        let chunks = read_line!();

        // mulligan_action
        let chunks = read_line!();
        let _mulligan_action = MulliganAction::from(chunks[0].as_str());
    }
    for _ in 0..env.player_size {
        // shuffled deck
        let chunks = read_line!();
        decks.push(
            chunks
                .iter()
                .map(|v| v.parse::<usize>().expect("信頼する"))
                .collect::<Vec<usize>>(),
        );
    }
    let mut card_catalog = HashMap::new();
    for card in cards.iter() {
        card_catalog.insert(card.id, card);
    }
    let mut state = State::new(&env, &card_catalog, &field, &decks);

    for turn in 1..=env.max_turn {
        generate_svg(&env, &card_catalog, &state, turn);
        let mut actions = vec![];
        for _ in 0..env.player_size {
            let chunks = read_line!();
            actions.push(Action::from(chunks.join(" ").as_str()));
        }
        state.apply(&env, &card_catalog, &actions);
    }
    generate_svg(&env, &card_catalog, &state, env.max_turn);
}

fn rect(x: i32, y: i32, w: i32, h: i32, fill: &str) -> Rectangle {
    Rectangle::new()
        .set("x", x)
        .set("y", y)
        .set("width", w)
        .set("height", h)
        .set("fill", fill)
}
const COLORS: [[&str; 2]; 2] = [["yellow", "orange"], ["blue", "aqua"]];
const BLOCK_COLOR: &str = "gray";
const EMPTY_COLOR: &str = "white";
fn field_color(square: &FieldSquareType) -> &'static str {
    match &square {
        FieldSquareType::Block => BLOCK_COLOR,
        FieldSquareType::Empty => EMPTY_COLOR,
        FieldSquareType::Special {
            player_id,
            activeted,
        } => match player_id {
            0 => COLORS[0][1],
            1 => COLORS[1][1],
            _ => EMPTY_COLOR,
        },
        FieldSquareType::Colored { player_id } => match player_id {
            0 => COLORS[0][0],
            1 => COLORS[1][0],
            _ => EMPTY_COLOR,
        },
    }
}
fn hand_color(square: &CardSquareType, player_id: usize) -> &'static str {
    match &square {
        CardSquareType::Empty => EMPTY_COLOR,
        CardSquareType::Special => match player_id {
            0 => COLORS[0][1],
            1 => COLORS[1][1],
            _ => EMPTY_COLOR,
        },
        CardSquareType::Colored => match player_id {
            0 => COLORS[0][0],
            1 => COLORS[1][0],
            _ => EMPTY_COLOR,
        },
    }
}
pub fn generate_svg(env: &Environment, cards: &HashMap<CardId, &Card>, state: &State, turn: usize) {
    eprintln!("turn:{}", turn);
    for row in state.field.squares.iter() {
        eprintln!("{:?}", row);
    }
    // とりあえず公式のサイズ感で崩れないようにする
    let cell_size = 30;
    let font_size = 60;
    let padding = cell_size;

    let card_height = cell_size
        * cards
            .values()
            .map(|c| c.shape.height)
            .max()
            .expect("大丈夫なはず");
    let card_width = cell_size
        * cards
            .values()
            .map(|c| c.shape.width)
            .max()
            .expect("大丈夫なはず");
    let padding_hand_zone_width = padding * 2 + card_width * 2;

    // hand zone
    let hand_zone_height = padding * 2 + card_height * 4 + padding * 3;
    let hand_zone_width = padding * 2 + card_width * 2 + padding;
    // filed
    let field_height = (state.field.height) * cell_size + padding * 2;
    let field_width = (state.field.width) * cell_size + padding * 2;

    let canvas_h = std::cmp::max(hand_zone_height, field_height);
    let canvas_w = hand_zone_width + field_width;

    // 画像サイズ設定
    let mut doc = svg::Document::new()
        .set("id", "visualizer")
        .set("viewBox", (0, 0, canvas_w, canvas_h))
        .set("width", canvas_w)
        .set("height", canvas_h);
    // 背景色描画
    doc = doc.add(rect(0, 0, canvas_w as i32, canvas_h as i32, BLOCK_COLOR));

    // handsを描画
    for (player_id, player) in state.players.iter().enumerate() {
        for (card_index, card_id) in player.hands.iter().enumerate() {
            let base_y = padding
                + (card_index / 2 * (padding + card_height))
                + player_id * 2 * (padding + card_height);
            let base_x = padding + card_index % 2 * (card_width + padding);
            // 背景
            doc = doc.add(rect(
                base_x as i32,
                base_y as i32,
                card_width as i32,
                card_height as i32,
                EMPTY_COLOR,
            ));

            // 本体
            let card = cards.get(card_id).expect("ある");
            for i in 0..card.shape.height {
                for j in 0..card.shape.width {
                    let y = base_y + i * cell_size;
                    let x = base_x + j * cell_size;
                    let color = hand_color(&card.shape.squares[i][j], player_id);
                    doc = doc.add(rect(
                        x as i32,
                        y as i32,
                        cell_size as i32,
                        cell_size as i32,
                        color,
                    ));
                }
            }
        }
    }
    // fieldを描画
    for i in 0..state.field.height {
        for j in 0..state.field.width {
            let y = padding + i * cell_size;
            let x = padding + j * cell_size + padding_hand_zone_width;
            let color = field_color(&state.field.squares[i][j]);
            doc = doc.add(rect(
                x as i32,
                y as i32,
                cell_size as i32,
                cell_size as i32,
                color,
            ));
        }
    }
    // special pointを描画
    doc = doc.add(
        svg::node::element::Text::new()
            .set("x", hand_zone_width)
            .set("y", field_height as f64 + font_size as f64)
            .set("font-size", font_size)
            .set("text-anchor", "left")
            .add(svg::node::Text::new(format!(
                "t{},y{},b{}",
                turn, state.players[0].special_point, state.players[1].special_point
            ))),
    );

    // ファイルに書き出し
    let filename = format!("tmp/TableturfBattle_{:04}.svg", turn);
    std::fs::write(filename, &doc.to_string()).unwrap();
}
