use std::convert::From;
use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
};
type CardId = usize;
type FieldId = usize;
type PlayerId = usize;

const DYDX4: [(usize, usize); 4] = [(!0, 0), (0, !0), (0, 1), (1, 0)];
const DYDX8: [(usize, usize); 8] = [
    (!0, !0),
    (!0, 0),
    (!0, 1),
    (0, !0),
    (0, 1),
    (1, !0),
    (1, 0),
    (1, 1),
];

pub struct Environment {
    pub hand_size: usize,
    pub max_turn: usize,
    pub deck_size: usize,
}
impl Environment {
    pub fn new(deck_size: usize, hand_size: usize, max_turn: usize) -> Self {
        assert!(max_turn + hand_size <= deck_size + 1);
        Self {
            hand_size,
            max_turn,
            deck_size,
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash)]
pub enum FieldSquareType {
    Colored { player_id: PlayerId },
    Special { player_id: PlayerId },
    Block,
    Empty,
}
impl FieldSquareType {
    pub fn to_char(&self) -> char {
        match &self {
            FieldSquareType::Colored { player_id } => {
                if *player_id == 0 {
                    'y'
                } else {
                    'b'
                }
            }
            FieldSquareType::Special { player_id } => {
                if *player_id == 0 {
                    'Y'
                } else {
                    'B'
                }
            }
            FieldSquareType::Block => '#',
            FieldSquareType::Empty => '.',
        }
    }
}
impl From<char> for FieldSquareType {
    fn from(item: char) -> Self {
        match item {
            'y' => FieldSquareType::Colored { player_id: 0 },
            'Y' => FieldSquareType::Special { player_id: 0 },
            'b' => FieldSquareType::Colored { player_id: 1 },
            'B' => FieldSquareType::Special { player_id: 1 },
            '#' => FieldSquareType::Block,
            '.' => FieldSquareType::Empty,
            _ => {
                eprintln!("invalid input {}", item);
                unreachable!()
            }
        }
    }
}
impl Display for FieldSquareType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash)]
pub enum CardSquareType {
    Colored,
    Special,
    Empty,
}
impl CardSquareType {
    pub fn to_char(&self) -> char {
        match &self {
            CardSquareType::Colored => 'y',
            CardSquareType::Special => 'Y',
            CardSquareType::Empty => '.',
        }
    }
}
impl From<char> for CardSquareType {
    fn from(item: char) -> Self {
        match item {
            'y' => CardSquareType::Colored,
            'Y' => CardSquareType::Special,
            '.' => CardSquareType::Empty,
            _ => {
                eprintln!("invalid input {}", item);
                unreachable!()
            }
        }
    }
}

impl Display for CardSquareType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct FieldShape {
    pub height: usize,
    pub width: usize,
    pub squares: Vec<Vec<FieldSquareType>>,
}
impl FieldShape {
    fn new(item: &str) -> Self {
        let mut squares: Vec<Vec<FieldSquareType>> = vec![];
        for row in item.split('\n') {
            squares.push(row.chars().map(FieldSquareType::from).collect())
        }
        Self {
            height: squares.len(),
            width: squares[0].len(),
            squares,
        }
    }
    fn count_squares(&self, field_square_types: &[FieldSquareType]) -> usize {
        let mut count = 0;
        for i in 0..self.height {
            for j in 0..self.width {
                if field_square_types.contains(&self.squares[i][j]) {
                    count += 1;
                }
            }
        }
        count
    }
    pub fn count_player(&self, player_id: PlayerId) -> usize {
        self.count_squares(&[
            FieldSquareType::Colored { player_id },
            FieldSquareType::Special { player_id },
        ])
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct CardShape {
    pub height: usize,
    pub width: usize,
    pub squares: Vec<Vec<CardSquareType>>,
}
impl CardShape {
    pub fn new(item: &str) -> Self {
        let mut squares: Vec<Vec<CardSquareType>> = vec![];
        for row in item.split('\n') {
            squares.push(row.chars().map(CardSquareType::from).collect())
        }
        Self {
            height: squares.len(),
            width: squares[0].len(),
            squares,
        }
    }
    pub fn trim(seed: &CardShape) -> CardShape {
        let mut min_y = seed.height;
        let mut max_y = 1;
        let mut min_x = seed.width;
        let mut max_x = 1;
        for i in 0..seed.height {
            for j in 0..seed.width {
                if matches!(
                    seed.squares[i][j],
                    CardSquareType::Colored | CardSquareType::Special
                ) {
                    min_y = std::cmp::min(min_y, i);
                    max_y = std::cmp::max(max_y, i);
                    min_x = std::cmp::min(min_x, j);
                    max_x = std::cmp::max(max_x, j);
                }
            }
        }

        let mut trimmed = vec![];
        for i in min_y..=max_y {
            let mut row = vec![];
            for j in min_x..=max_x {
                row.push(seed.squares[i][j]);
            }
            trimmed.push(row);
        }
        Self {
            height: trimmed.len(),
            width: trimmed[0].len(),
            squares: trimmed,
        }
    }

    fn find_reference_point(&self, _player_id: PlayerId) -> (usize, usize) {
        for i in 0..self.height {
            for j in 0..self.width {
                if matches!(
                    self.squares[i][j],
                    CardSquareType::Colored | CardSquareType::Special
                ) {
                    return (i, j);
                }
            }
        }
        unreachable!()
    }
    fn count_squares(&self, CardSquareTypes: &[CardSquareType]) -> usize {
        let mut count = 0;
        for i in 0..self.height {
            for j in 0..self.width {
                if CardSquareTypes.contains(&self.squares[i][j]) {
                    count += 1;
                }
            }
        }
        count
    }
    pub fn count_colored_squares(&self) -> usize {
        self.count_squares(&[CardSquareType::Colored, CardSquareType::Special])
    }
    // 右90度回転
    fn rotate(&self) -> Self {
        let height = self.width;
        let width = self.height;
        let mut squares = vec![];
        for i in 0..height {
            let mut row = vec![];
            for j in 0..width {
                row.push(self.squares[self.height - 1 - j][i]);
            }
            squares.push(row);
        }
        Self {
            height,
            width,
            squares,
        }
    }
}

#[derive(Eq, PartialEq, Hash)]
pub struct Card {
    pub id: CardId,
    pub name: String,
    pub cost: usize,
    pub power: usize,
    pub shape: CardShape,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            id: 1,
            name: String::from("ヒーローシューター"),
            cost: 5,
            power: 12, // shapeと対応を取ること
            shape: CardShape::new("yyyyy\nyyyYy\n.y...\ny...."),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, PartialOrd)]
pub enum Direction {
    Up,
    Down,
    Right,
    Left,
}
#[derive(PartialEq, Eq, Debug, Clone, Copy, PartialOrd)]
pub enum Action {
    Pass {
        card_id: CardId,
    },
    Put {
        card_id: CardId,
        dir: Direction,
        y: usize,
        x: usize,
    },
    SpecialPut {
        card_id: CardId,
        dir: Direction,
        y: usize,
        x: usize,
    },
}

pub struct PlayerState {
    pub special_point: usize,
    pub hands: Vec<CardId>,
    pub deck: VecDeque<CardId>,
}
pub struct State {
    pub turn: usize,
    pub field: FieldShape,
    pub players: Vec<PlayerState>,
}
impl State {
    pub fn new(
        env: &Environment,
        cards: &HashMap<CardId, Card>,
        field: &Field,
        yellow_deck: &Vec<CardId>,
        blue_deck: &Vec<CardId>,
    ) -> Self {
        assert_eq!(yellow_deck.len(), env.deck_size);
        assert_eq!(blue_deck.len(), env.deck_size);
        // デッキに含まれるのはカード情報の存在するカードのみであることを検査する。
        for card_id in yellow_deck.iter() {
            assert!(cards.contains_key(card_id));
        }
        for card_id in blue_deck.iter() {
            assert!(cards.contains_key(card_id));
        }
        let mut yellow_deck: VecDeque<CardId> = VecDeque::from(yellow_deck.clone());
        let mut blue_deck: VecDeque<CardId> = VecDeque::from(blue_deck.clone());
        let mut yellow_hands = vec![];
        let mut blue_hands = vec![];
        for _ in 0..env.hand_size {
            if let Some(card_id) = yellow_deck.pop_front() {
                yellow_hands.push(card_id);
            }
            if let Some(card_id) = blue_deck.pop_front() {
                blue_hands.push(card_id);
            }
        }

        State {
            turn: 0,
            field: field.shape.clone(),
            players: vec![
                PlayerState {
                    special_point: 0,
                    hands: yellow_hands,
                    deck: yellow_deck,
                },
                PlayerState {
                    special_point: 0,
                    hands: blue_hands,
                    deck: blue_deck,
                },
            ],
        }
    }
    pub fn is_win(&self, env: &Environment, player_id: PlayerId) -> bool {
        self.is_done(env)
            && self.field.count_player(player_id) > self.field.count_player(1 - player_id)
    }
    pub fn is_lose(&self, env: &Environment, player_id: PlayerId) -> bool {
        self.is_done(env)
            && self.field.count_player(player_id) < self.field.count_player(1 - player_id)
    }
    pub fn is_draw(&self, env: &Environment) -> bool {
        self.is_done(env) && self.field.count_player(0) == self.field.count_player(1)
    }
    pub fn is_done(&self, env: &Environment) -> bool {
        self.turn == env.max_turn
    }
    fn is_valid_action(
        &self,
        cards: &HashMap<CardId, Card>,
        action: &Action,
        player_id: usize,
    ) -> bool {
        assert!(player_id < self.players.len());
        match action {
            Action::Pass { card_id } => self.players[player_id].hands.contains(card_id),
            Action::Put { card_id, dir, y, x } => {
                let card = cards
                    .get(card_id)
                    .expect("all cards in deck are contained cards");
                let shape = match dir {
                    Direction::Up => card.shape.clone(),
                    Direction::Right => card.shape.rotate(),
                    Direction::Down => card.shape.rotate().rotate(),
                    Direction::Left => card.shape.rotate().rotate().rotate(),
                };
                // 置けるのか？
                let mut is_adjacent = false;
                let (ry, rx) = shape.find_reference_point(0);
                for i in 0..card.shape.height {
                    for j in 0..card.shape.width {
                        match card.shape.squares[i][j] {
                            CardSquareType::Colored | CardSquareType::Special => {
                                // 基準点(ry,rx)が(y,x)に置かれるので左下の座標がマイナスになる可能性がある
                                if y + i < ry || x + j < rx {
                                    return false;
                                }
                                let cy = y + i - ry;
                                let cx = x + j - rx;
                                if self.field.height <= cy || self.field.width <= cx {
                                    return false;
                                }
                                if self.field.squares[cy][cx] != FieldSquareType::Empty {
                                    return false;
                                }
                                // 既存の自身のマスに隣接しているか判定する
                                for (dy, dx) in DYDX8.iter() {
                                    if self.field.height <= cy.wrapping_add(*dy)
                                        || self.field.width <= cx.wrapping_add(*dx)
                                    {
                                        continue;
                                    }
                                    if let FieldSquareType::Colored { player_id: pid }
                                    | FieldSquareType::Special { player_id: pid } = self
                                        .field
                                        .squares[cy.wrapping_add(*dy)][cx.wrapping_add(*dx)]
                                    {
                                        if pid == player_id {
                                            is_adjacent = true;
                                        }
                                    }
                                }
                            }
                            _ => {
                                //do_nothing
                            }
                        }
                    }
                }
                is_adjacent
            }
            Action::SpecialPut { card_id, dir, y, x } => {
                let card = cards
                    .get(card_id)
                    .expect("all cards in deck are contained cards");
                // スペシャルショットのコストが払えるのか？
                if self.players[player_id].special_point < card.cost {
                    return false;
                }

                let shape = match dir {
                    Direction::Up => card.shape.clone(),
                    Direction::Right => card.shape.rotate(),
                    Direction::Down => card.shape.rotate().rotate(),
                    Direction::Left => card.shape.rotate().rotate().rotate(),
                };
                // 置けるのか？
                let (ry, rx) = shape.find_reference_point(0);
                for i in 0..card.shape.height {
                    for j in 0..card.shape.width {
                        match card.shape.squares[i][j] {
                            CardSquareType::Colored | CardSquareType::Special => {
                                // 基準点(ry,rx)が(y,x)に置かれるので左下の座標がマイナスになる可能性がある
                                if y + i < ry || x + j < rx {
                                    return false;
                                }
                                let cy = y + i - ry;
                                let cx = x + j - rx;
                                if matches!(
                                    self.field.squares[cy][cx],
                                    FieldSquareType::Special { player_id: _ }
                                        | FieldSquareType::Block
                                ) {
                                    return false;
                                }
                            }
                            _ => {
                                //do_nothing
                            }
                        }
                    }
                }
                true
            }
        }
    }
    pub fn generate_valid_actions(
        &mut self,
        cards: &HashMap<CardId, Card>,
        player_id: PlayerId,
    ) -> Vec<Action> {
        let mut candidates = vec![];
        for card_id in self.players[player_id].hands.iter() {
            // let card = cards
            //     .get(card_id)
            //     .expect("all cards in deck are contained cards");
            for dir in [
                Direction::Up,
                Direction::Right,
                Direction::Down,
                Direction::Left,
            ]
            .iter()
            {
                for y in 0..self.field.height {
                    for x in 0..self.field.width {
                        let action = Action::Put {
                            card_id: *card_id,
                            dir: *dir,
                            y,
                            x,
                        };
                        if self.is_valid_action(cards, &action, player_id) {
                            candidates.push(action);
                        }
                    }
                }
            }
            candidates.push(Action::Pass { card_id: *card_id });
        }
        candidates
    }

    pub fn apply_put(&mut self, _cards: &HashMap<CardId, Card>, actions: &[Action]) {
        assert!(actions.iter().all(|a| matches!(
            a,
            Action::Put {
                card_id: _,
                dir: _,
                y: _,
                x: _,
            } | Action::SpecialPut {
                card_id: _,
                dir: _,
                y: _,
                x: _,
            }
        )))
    }
    pub fn apply(&mut self, env: &Environment, cards: &HashMap<CardId, Card>, actions: &[Action]) {
        // まず受け取ったアクションが有効なことを確認する。
        if !self.is_valid_action(cards, &actions[0], 0)
            || !self.is_valid_action(cards, &actions[1], 1)
        {
            todo!("invalidな時の処理は後で書く");
        }

        let mut action_orders = vec![];
        for (i, action) in actions.iter().enumerate() {
            match &action {
                Action::Pass { card_id: _ } => {
                    action_orders.push((0, i));
                }
                Action::Put {
                    card_id,
                    dir: _,
                    y: _,
                    x: _,
                } => {
                    let card_power = cards
                        .get(card_id)
                        .expect("all cards in deck are contained cards")
                        .power;
                    action_orders.push((card_power, i));
                }
                Action::SpecialPut {
                    card_id,
                    dir: _,
                    y: _,
                    x: _,
                } => {
                    let card_power = cards
                        .get(card_id)
                        .expect("all cards in deck are contained cards")
                        .power;
                    action_orders.push((card_power, i));
                }
            }
        }
        action_orders.sort_by_key(|x| std::cmp::Reverse(*x));

        // TODO: 同一powerのカードの衝突を考慮する。
        for (_, action_index) in action_orders.iter() {
            // PASS
            if let Action::Pass { card_id: _ } = actions[*action_index] {
                self.players[*action_index].special_point += 1;
            }
            if let Action::Put { card_id, dir, y, x } = actions[*action_index] {
                let card = cards
                    .get(&card_id)
                    .expect("all cards in deck are contained cards");
                let shape = match dir {
                    Direction::Up => card.shape.clone(),
                    Direction::Right => card.shape.rotate(),
                    Direction::Down => card.shape.rotate().rotate(),
                    Direction::Left => card.shape.rotate().rotate().rotate(),
                };
                let (ry, rx) = shape.find_reference_point(0);
                for i in 0..card.shape.height {
                    for j in 0..card.shape.width {
                        if matches!(card.shape.squares[i][j], CardSquareType::Colored) {
                            let cy = y + i - ry;
                            let cx = x + j - rx;
                            self.field.squares[cy][cx] = FieldSquareType::Colored {
                                player_id: *action_index,
                            };
                        } else if matches!(card.shape.squares[i][j], CardSquareType::Colored) {
                            let cy = y + i - ry;
                            let cx = x + j - rx;
                            self.field.squares[cy][cx] = FieldSquareType::Special {
                                player_id: *action_index,
                            };
                        }
                    }
                }
            }
        }
        // powerの大きい方から順番に処理
        // 2人用で実装してしまう

        // match (actions[0], actions[1]) {
        //     (Action::Pass { card_id: _ }, Action::Pass { card_id: _ }) => {
        //         // 処理済み
        //     }
        //     (Action::Pass { card_id: _ }, _) => {
        //         // player0がPASSならplayer1は衝突を気にしなくて良い
        //         match actions[1] {
        //             Action::Put {
        //                 card_id: _,
        //                 dir: _,
        //                 y: _,
        //                 x: _,
        //             } => {
        //                 todo!()
        //             }
        //             Action::SpecialPut {
        //                 card_id: _,
        //                 dir: _,
        //                 y: _,
        //                 x: _,
        //             } => {
        //                 todo!()
        //             }
        //             _ => unreachable!(),
        //         }
        //     }
        //     (_, Action::Pass { card_id: _ }) => {
        //         // player1がPASSならplayer0は衝突を気にしなくて良い
        //     }
        //     _ => {
        //         //同時に置こうとしているケース
        //         // powerが同じか？
        //         // 大きい方から処理
        //         // ぶつかるマスを計算
        //         todo!()
        //     }
        // }

        // 使ったカードを捨てる
        for i in 0..self.players.len() {
            let card_id = match actions[i] {
                Action::Pass { card_id } => card_id,
                Action::Put {
                    card_id,
                    dir: _,
                    y: _,
                    x: _,
                } => card_id,
                Action::SpecialPut {
                    card_id,
                    dir: _,
                    y: _,
                    x: _,
                } => card_id,
            };
            let mut index = 0;
            for (j, &v) in self.players[i].hands.iter().enumerate() {
                if v == card_id {
                    index = j;
                    break;
                }
            }
            assert!(index <= self.players[i].hands.len()); // removeはpanicする可能性があるので検証しておく
            assert_eq!(self.players[i].hands[index], card_id);
            self.players[i].hands.remove(index);
        }
        // 次のターン
        self.turn += 1;
        // 新しいカードを引く
        if self.turn < env.max_turn {
            for i in 0..self.players.len() {
                let new_card_id = self.players[i]
                    .deck
                    .pop_front()
                    .expect("deck has enugh cards");
                self.players[i].hands.push(new_card_id);
            }
        }
    }
}

#[derive(Clone)]
pub struct Field {
    pub id: FieldId,
    pub name: String,
    pub shape: FieldShape,
}
impl Default for Field {
    fn default() -> Self {
        Self {
            id: 1,
            name: String::from("まっすぐストリート"),
            shape: FieldShape::new(
                &(String::from(".........\n").repeat(3)
                    + "....B....\n"
                    + &String::from(".........\n").repeat(18)
                    + "....Y....\n"
                    + String::from(".........\n")
                        .repeat(3)
                        .strip_suffix('\n')
                        .unwrap()),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_squares_from_str() {
        let expected = vec![
            vec![
                FieldSquareType::Colored { player_id: 0 },
                FieldSquareType::Colored { player_id: 0 },
                FieldSquareType::Colored { player_id: 0 },
                FieldSquareType::Colored { player_id: 0 },
                FieldSquareType::Colored { player_id: 0 },
            ],
            vec![
                FieldSquareType::Colored { player_id: 0 },
                FieldSquareType::Colored { player_id: 0 },
                FieldSquareType::Colored { player_id: 0 },
                FieldSquareType::Special { player_id: 0 },
                FieldSquareType::Colored { player_id: 0 },
            ],
            vec![
                FieldSquareType::Empty,
                FieldSquareType::Colored { player_id: 0 },
                FieldSquareType::Empty,
                FieldSquareType::Empty,
                FieldSquareType::Empty,
            ],
            vec![
                FieldSquareType::Colored { player_id: 0 },
                FieldSquareType::Empty,
                FieldSquareType::Empty,
                FieldSquareType::Empty,
                FieldSquareType::Empty,
            ],
        ];
        let actual = FieldShape::new("yyyyy\nyyyYy\n.y...\ny....").squares;
        assert_eq!(actual, expected);
    }
    #[test]
    fn test_find_reference_point() {
        let expected = (0, 0);
        let actual = CardShape::new("yyyyy\nyyyYy\n.y...\ny....").find_reference_point(0);
        assert_eq!(actual, expected);

        let expected = (1, 0);
        let actual = CardShape::new("..\ny.").find_reference_point(0);
        assert_eq!(actual, expected);

        let expected = (0, 2);
        let actual = CardShape::new("..Y.\ny.yy").find_reference_point(0);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_rotate() {
        let expected = CardShape::new("Yy\n.y\n.y\n..\n..");
        let shape = CardShape::new("yyy..\nY....");
        let actual = shape.rotate();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_trim() {
        let big = CardShape::new(
            "........\n........\n.yyyyy..\n.yyyYy..\n..y.....\n.y......\n........\n........",
        );
        let trimmed = CardShape::trim(&big);
        let expected = CardShape::new("yyyyy\nyyyYy\n.y...\ny....");
        assert_eq!(trimmed, expected);
    }
}