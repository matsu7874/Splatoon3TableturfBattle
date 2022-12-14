use log::{debug, error};
use std::convert::From;
use std::{
    collections::{HashMap, VecDeque},
    fmt::{Display, Formatter},
};

pub mod text_protocol;
pub type CardId = usize;
pub type FieldId = usize;
pub type PlayerId = usize;

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
    pub player_size: usize,
    pub hand_size: usize,
    pub max_turn: usize,
    pub deck_size: usize,
    pub is_deplicated_pick_enabled: bool,
}
impl Environment {
    pub fn new(
        player_size: usize,
        deck_size: usize,
        hand_size: usize,
        max_turn: usize,
        is_deplicated_pick_enabled: bool,
    ) -> Self {
        assert!(max_turn + hand_size <= deck_size + 1);
        Self {
            player_size,
            hand_size,
            max_turn,
            deck_size,
            is_deplicated_pick_enabled,
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash)]
pub enum FieldSquareType {
    Colored {
        player_id: PlayerId,
    },
    Special {
        player_id: PlayerId,
        activeted: bool,
    },
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
            FieldSquareType::Special {
                player_id,
                activeted: _,
            } => {
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
    fn activate(&mut self) {
        if let Self::Special {
            player_id,
            activeted: false,
        } = self
        {
            *self = Self::Special {
                player_id: *player_id,
                activeted: true,
            };
        } else {
            unreachable!();
        }
    }
}
impl From<char> for FieldSquareType {
    fn from(item: char) -> Self {
        match item {
            'y' => FieldSquareType::Colored { player_id: 0 },
            'Y' => FieldSquareType::Special {
                player_id: 0,
                activeted: false,
            },
            'b' => FieldSquareType::Colored { player_id: 1 },
            'B' => FieldSquareType::Special {
                player_id: 1,
                activeted: false,
            },
            '#' => FieldSquareType::Block,
            '.' => FieldSquareType::Empty,
            _ => {
                error!("invalid input {}", item);
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
                error!("invalid input {}", item);
                unreachable!()
            }
        }
    }
}

impl Display for CardSquareType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
    pub fn new(item: &str) -> Self {
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
            FieldSquareType::Special {
                player_id,
                activeted: true,
            },
            FieldSquareType::Special {
                player_id,
                activeted: false,
            },
        ])
    }
}
impl std::fmt::Display for FieldShape {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let s = self
            .squares
            .iter()
            .map(|row| String::from_iter(row.iter().map(|c| c.to_char())))
            .collect::<Vec<String>>()
            .join("\n");
        write!(f, "{}", s)
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
        let mut max_y = 0;
        let mut min_x = seed.width;
        let mut max_x = 0;
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
    fn count_squares(&self, card_square_types: &[CardSquareType]) -> usize {
        let mut count = 0;
        for i in 0..self.height {
            for j in 0..self.width {
                if card_square_types.contains(&self.squares[i][j]) {
                    count += 1;
                }
            }
        }
        count
    }
    pub fn count_colored_squares(&self) -> usize {
        self.count_squares(&[CardSquareType::Colored, CardSquareType::Special])
    }
    // ???90?????????
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
impl std::fmt::Display for CardShape {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let s = self
            .squares
            .iter()
            .map(|row| String::from_iter(row.iter().map(|c| c.to_char())))
            .collect::<Vec<String>>()
            .join("\n");
        write!(f, "{}", s)
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
impl Card {
    pub fn new(id: CardId, name: &str, cost: usize, shape: CardShape) -> Self {
        Self {
            id,
            name: String::from(name),
            cost,
            power: shape.count_colored_squares(),
            shape,
        }
    }
}

impl Default for Card {
    fn default() -> Self {
        Self {
            id: 1,
            name: String::from("???????????????????????????"),
            cost: 5,
            power: 12, // shape????????????????????????
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
impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let s = match self {
            Direction::Up => 'U',
            Direction::Down => 'D',
            Direction::Right => 'R',
            Direction::Left => 'L',
        };
        write!(f, "{}", s)
    }
}
impl From<char> for Direction {
    fn from(c: char) -> Self {
        match c {
            'U' => Self::Up,
            'D' => Self::Down,
            'R' => Self::Right,
            'L' => Self::Left,
            _ => {
                error!("Direction from error by: {}", c);
                unreachable!()
            }
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, PartialOrd)]
pub enum MulliganAction {
    Pass,
    Mulligan,
}
impl std::fmt::Display for MulliganAction {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let s = match self {
            Self::Pass => "PASS",
            Self::Mulligan => "MULLIGAN",
        };
        write!(f, "{}", s)
    }
}
impl From<&str> for MulliganAction {
    fn from(s: &str) -> Self {
        match s.trim() {
            "PASS" => Self::Pass,
            "MULLIGAN" => Self::Mulligan,
            _ => unreachable!(),
        }
    }
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
impl Action {
    fn get_card_id(&self) -> CardId {
        match self {
            Action::Pass { card_id } => *card_id,
            Action::Put {
                card_id,
                dir: _,
                y: _,
                x: _,
            } => *card_id,
            Action::SpecialPut {
                card_id,
                dir: _,
                y: _,
                x: _,
            } => *card_id,
        }
    }
}
impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let s = match self {
            Action::Pass { card_id } => format!("PASS {}", card_id),
            Action::Put { card_id, dir, y, x } => format!("PUT {} {} {} {}", card_id, dir, y, x),
            Action::SpecialPut { card_id, dir, y, x } => {
                format!("SPECIAL_PUT {} {} {} {}", card_id, dir, y, x)
            }
        };
        write!(f, "{}", s)
    }
}
impl From<&str> for Action {
    fn from(s: &str) -> Self {
        let chunks = s
            .trim()
            .split(' ')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>();
        match chunks[0] {
            "PASS" => Self::Pass {
                card_id: chunks[1].parse::<usize>().unwrap(),
            },
            "PUT" => Self::Put {
                card_id: chunks[1].parse::<usize>().unwrap(),
                dir: Direction::from(chunks[2].parse::<char>().unwrap()),
                y: chunks[3].parse::<usize>().unwrap(),
                x: chunks[4].parse::<usize>().unwrap(),
            },
            "SPECIAL_PUT" => Self::SpecialPut {
                card_id: chunks[1].parse::<usize>().unwrap(),
                dir: Direction::from(chunks[2].parse::<char>().unwrap()),
                y: chunks[3].parse::<usize>().unwrap(),
                x: chunks[4].parse::<usize>().unwrap(),
            },
            _ => unreachable!(),
        }
    }
}

fn get_cursor(
    reference_point_y: usize,
    reference_point_x: usize,
    target_y: usize,
    target_x: usize,
    i: usize,
    j: usize,
) -> Option<(usize, usize)> {
    // ?????????(ry,rx)???(ty,tx)??????????????????????????????????????????????????????????????????????????????
    if target_y + i < reference_point_y || target_x + j < reference_point_x {
        return None;
    }
    let cy = target_y + i - reference_point_y;
    let cx = target_x + j - reference_point_x;
    Some((cy, cx))
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
        cards: &HashMap<CardId, &Card>,
        field: &Field,
        decks: &[Vec<CardId>],
    ) -> Self {
        for deck in decks.iter() {
            assert_eq!(deck.len(), env.deck_size);
            // ????????????????????????????????????????????????????????????????????????????????????????????????????????????
            for card_id in deck.iter() {
                assert!(cards.contains_key(card_id));
            }
        }

        let mut cloned_decks = decks
            .iter()
            .map(|deck| VecDeque::from(deck.clone()))
            .collect::<Vec<VecDeque<CardId>>>();

        let mut hands = vec![];
        for player_id in 0..env.player_size {
            let mut hand: Vec<CardId> = vec![];
            for _ in 0..env.hand_size {
                if let Some(card_id) = cloned_decks[player_id].pop_front() {
                    hand.push(card_id);
                }
            }
            hands.push(hand);
        }

        State {
            turn: 1,
            field: field.shape.clone(),
            players: hands
                .into_iter()
                .zip(cloned_decks.into_iter())
                .map(|(hand, deck)| PlayerState {
                    special_point: 0,
                    hands: hand,
                    deck,
                })
                .collect::<Vec<PlayerState>>(),
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
        self.turn > env.max_turn
    }
    fn is_valid_action(
        &self,
        cards: &HashMap<CardId, &Card>,
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
                // 1. ?????????????????????????????????????????????
                // 2. ???????????????????????????????????????????????????????????????????????????
                let mut is_adjacent = false;
                let (ry, rx) = shape.find_reference_point(0);
                for i in 0..card.shape.height {
                    for j in 0..card.shape.width {
                        if matches!(
                            card.shape.squares[i][j],
                            CardSquareType::Colored | CardSquareType::Special
                        ) {
                            // ?????????(ry,rx)???(y,x)??????????????????????????????????????????????????????????????????????????????
                            let cur = get_cursor(ry, rx, *y, *x, i, j);
                            if cur.is_none() {
                                return false;
                            }
                            let (cy, cx) = cur.expect("none??????????????????????????????????????????");
                            if self.field.height <= cy || self.field.width <= cx {
                                return false;
                            }

                            if self.field.squares[cy][cx] != FieldSquareType::Empty {
                                return false;
                            }
                            // ????????????????????????????????????????????????????????????
                            for (dy, dx) in DYDX8.iter() {
                                if is_adjacent {
                                    break;
                                }
                                if self.field.height <= cy.wrapping_add(*dy)
                                    || self.field.width <= cx.wrapping_add(*dx)
                                {
                                    continue;
                                }
                                if let FieldSquareType::Colored { player_id: pid }
                                | FieldSquareType::Special {
                                    player_id: pid,
                                    activeted: _,
                                } =
                                    self.field.squares[cy.wrapping_add(*dy)][cx.wrapping_add(*dx)]
                                {
                                    if pid == player_id {
                                        is_adjacent = true;
                                    }
                                }
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
                // ????????????????????????????????????????????????????????????
                if self.players[player_id].special_point < card.cost {
                    return false;
                }

                let shape = match dir {
                    Direction::Up => card.shape.clone(),
                    Direction::Right => card.shape.rotate(),
                    Direction::Down => card.shape.rotate().rotate(),
                    Direction::Left => card.shape.rotate().rotate().rotate(),
                };
                // 1. ????????????????????????????????????????????????????????????
                // 2. ????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????????
                let mut is_adjacent = false;
                let (ry, rx) = shape.find_reference_point(0);
                for i in 0..card.shape.height {
                    for j in 0..card.shape.width {
                        if matches!(
                            card.shape.squares[i][j],
                            CardSquareType::Colored | CardSquareType::Special
                        ) {
                            // ?????????Special???Block?????????Special?????????
                            // ?????????(ry,rx)???(y,x)??????????????????????????????????????????????????????????????????????????????
                            let cur = get_cursor(ry, rx, *y, *x, i, j);
                            if cur.is_none() {
                                return false;
                            }
                            let (cy, cx) = cur.expect("none??????????????????????????????????????????");
                            if self.field.height <= cy || self.field.width <= cx {
                                return false;
                            }

                            if matches!(
                                self.field.squares[cy][cx],
                                FieldSquareType::Special {
                                    player_id: _,
                                    activeted: _
                                } | FieldSquareType::Block
                            ) {
                                return false;
                            }

                            // ???????????????????????????????????????????????????????????????????????????
                            for (dy, dx) in DYDX8.iter() {
                                if is_adjacent {
                                    break;
                                }
                                if self.field.height <= cy.wrapping_add(*dy)
                                    || self.field.width <= cx.wrapping_add(*dx)
                                {
                                    continue;
                                }
                                if let FieldSquareType::Special {
                                    player_id: pid,
                                    activeted: _,
                                } =
                                    self.field.squares[cy.wrapping_add(*dy)][cx.wrapping_add(*dx)]
                                {
                                    if pid == player_id {
                                        is_adjacent = true;
                                    }
                                }
                            }
                        }
                    }
                }
                is_adjacent
            }
        }
    }
    pub fn generate_valid_actions(
        &mut self,
        cards: &HashMap<CardId, &Card>,
        player_id: PlayerId,
    ) -> Vec<Action> {
        let mut candidates = vec![];
        for card_id in self.players[player_id].hands.iter() {
            let cost = cards
                .get(card_id)
                .expect("all cards in deck are contained cards")
                .cost;
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
                if cost <= self.players[player_id].special_point {
                    for y in 0..self.field.height {
                        for x in 0..self.field.width {
                            let action = Action::SpecialPut {
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
            }
            candidates.push(Action::Pass { card_id: *card_id });
        }
        candidates
    }

    fn activates(&mut self, putted_this_turn_squares: &[(usize, usize)]) -> Vec<usize> {
        let mut activated_counts = vec![0; self.players.len()];
        for (y, x) in putted_this_turn_squares {
            for (dy, dx) in DYDX8.iter() {
                let ny = y.wrapping_add(*dy);
                let nx = x.wrapping_add(*dx);
                if self.field.height <= ny || self.field.width <= nx {
                    continue;
                }
                if let FieldSquareType::Special {
                    player_id,
                    activeted: false,
                } = self.field.squares[ny][nx]
                {
                    let special_y = ny;
                    let special_x = nx;
                    let mut has_neighbor_empty = false;
                    for (dy, dx) in DYDX8.iter() {
                        if self.field.height <= special_y.wrapping_add(*dy)
                            || self.field.width <= special_x.wrapping_add(*dx)
                        {
                            continue;
                        }
                        if matches!(
                            self.field.squares[special_y.wrapping_add(*dy)]
                                [special_x.wrapping_add(*dx)],
                            FieldSquareType::Empty
                        ) {
                            has_neighbor_empty = true;
                            break;
                        }
                    }
                    if !has_neighbor_empty {
                        self.players[player_id].special_point += 1;
                        activated_counts[player_id] += 1;
                    }
                    self.field.squares[special_y][special_x].activate();
                }
            }
        }
        activated_counts
    }

    pub fn apply(&mut self, env: &Environment, cards: &HashMap<CardId, &Card>, actions: &[Action]) {
        // ????????????????????????????????????????????????????????????????????????
        if !self.is_valid_action(cards, &actions[0], 0)
            || !self.is_valid_action(cards, &actions[1], 1)
        {
            todo!("invalid??????????????????????????????");
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
        action_orders.sort_by_key(|x| std::cmp::Reverse(*x)); // ???????????????????????????????????????

        // ??????power???????????????????????????????????????
        let mut unfixed_squares = HashMap::<(usize, usize), usize>::new();
        for (_, action_index) in action_orders.iter() {
            match actions[*action_index] {
                Action::Pass { card_id: _ } => {
                    self.players[*action_index].special_point += 1;
                }
                Action::Put { card_id, dir, y, x } => {
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
                                let cur = get_cursor(ry, rx, y, x, i, j);
                                let (cy, cx) =
                                    cur.expect("??????????????????valid??????????????????????????????????????????");
                                match self.field.squares[cy][cx] {
                                    FieldSquareType::Empty => {
                                        unfixed_squares.insert((cy, cx), card.power);
                                        self.field.squares[cy][cx] = FieldSquareType::Colored {
                                            player_id: *action_index,
                                        };
                                    }
                                    FieldSquareType::Colored { player_id: _ } => {
                                        let unfixed_square_power = unfixed_squares.get(&(cy,cx)).expect("is_valid_action????????????????????????????????????????????????????????????????????????");
                                        self.field.squares[cy][cx] =
                                            if *unfixed_square_power == card.power {
                                                FieldSquareType::Block
                                            } else {
                                                unfixed_squares.insert((cy, cx), card.power);
                                                FieldSquareType::Colored {
                                                    player_id: *action_index,
                                                }
                                            };
                                    }
                                    _ => { /* ?????????????????????????????? */ }
                                }
                            } else if matches!(card.shape.squares[i][j], CardSquareType::Special) {
                                let cur = get_cursor(ry, rx, y, x, i, j);
                                let (cy, cx) =
                                    cur.expect("??????????????????valid??????????????????????????????????????????");
                                match self.field.squares[cy][cx] {
                                    FieldSquareType::Empty
                                    | FieldSquareType::Colored { player_id: _ } => {
                                        unfixed_squares.insert((cy, cx), card.power);
                                        self.field.squares[cy][cx] = FieldSquareType::Special {
                                            player_id: *action_index,
                                            activeted: false,
                                        };
                                    }
                                    FieldSquareType::Special {
                                        player_id: _,
                                        activeted: _,
                                    } => {
                                        let unfixed_square_power = unfixed_squares.get(&(cy,cx)).expect("is_valid_action????????????????????????????????????????????????????????????????????????");
                                        self.field.squares[cy][cx] =
                                            if *unfixed_square_power == card.power {
                                                FieldSquareType::Block
                                            } else {
                                                unfixed_squares.insert((cy, cx), card.power);
                                                FieldSquareType::Special {
                                                    player_id: *action_index,
                                                    activeted: false,
                                                }
                                            };
                                    }
                                    _ => { /* ?????????????????????????????? */ }
                                }
                            }
                        }
                    }
                }
                Action::SpecialPut { card_id, dir, y, x } => {
                    debug!("use SpecialPut:{:?}", actions[*action_index]);
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
                                let cur = get_cursor(ry, rx, y, x, i, j);
                                let (cy, cx) =
                                    cur.expect("??????????????????valid??????????????????????????????????????????");
                                match self.field.squares[cy][cx] {
                                    FieldSquareType::Empty => {
                                        unfixed_squares.insert((cy, cx), card.power);
                                        self.field.squares[cy][cx] = FieldSquareType::Colored {
                                            player_id: *action_index,
                                        };
                                    }
                                    FieldSquareType::Colored { player_id: _ } => {
                                        self.field.squares[cy][cx] = match unfixed_squares
                                            .get(&(cy, cx))
                                        {
                                            Some(unfixed_square_power) => {
                                                // ???????????????????????????
                                                if *unfixed_square_power == card.power {
                                                    FieldSquareType::Block
                                                } else {
                                                    // ???????????????????????????
                                                    unfixed_squares.insert((cy, cx), card.power);
                                                    FieldSquareType::Colored {
                                                        player_id: *action_index,
                                                    }
                                                }
                                            }
                                            None => {
                                                // ???????????????
                                                unfixed_squares.insert((cy, cx), card.power);
                                                FieldSquareType::Colored {
                                                    player_id: *action_index,
                                                }
                                            }
                                        };
                                    }
                                    _ => { /* ?????????????????????????????? */ }
                                }
                            } else if matches!(card.shape.squares[i][j], CardSquareType::Special) {
                                let cur = get_cursor(ry, rx, y, x, i, j);
                                let (cy, cx) =
                                    cur.expect("??????????????????valid??????????????????????????????????????????");
                                match self.field.squares[cy][cx] {
                                    FieldSquareType::Empty
                                    | FieldSquareType::Colored { player_id: _ } => {
                                        unfixed_squares.insert((cy, cx), card.power);
                                        self.field.squares[cy][cx] = FieldSquareType::Special {
                                            player_id: *action_index,
                                            activeted: false,
                                        };
                                    }
                                    FieldSquareType::Special {
                                        player_id: _,
                                        activeted: _,
                                    } => {
                                        self.field.squares[cy][cx] = match unfixed_squares
                                            .get(&(cy, cx))
                                        {
                                            Some(unfixed_square_power) => {
                                                // ???????????????????????????
                                                if *unfixed_square_power == card.power {
                                                    FieldSquareType::Block
                                                } else {
                                                    // ???????????????????????????
                                                    unfixed_squares.insert((cy, cx), card.power);
                                                    FieldSquareType::Colored {
                                                        player_id: *action_index,
                                                    }
                                                }
                                            }
                                            None => unreachable!(),
                                        };
                                    }
                                    _ => { /* ?????????????????????????????? */ }
                                }
                            }
                        }
                    }
                    self.players[*action_index].special_point -= card.cost;
                }
            }
        }

        // ???????????????????????????????????????
        let putted_this_turn_squares = unfixed_squares
            .keys()
            .cloned()
            .collect::<Vec<(usize, usize)>>();
        self.activates(&putted_this_turn_squares);

        // ??????????????????????????????
        for (i, &action) in actions.iter().enumerate() {
            let card_id = action.get_card_id();
            let mut index = 0;
            for (j, &v) in self.players[i].hands.iter().enumerate() {
                if v == card_id {
                    index = j;
                    break;
                }
            }
            assert!(index <= self.players[i].hands.len()); // remove???panic????????????????????????????????????????????????
            assert_eq!(self.players[i].hands[index], card_id);
            self.players[i].hands.remove(index);
        }

        // ???????????????
        self.turn += 1;
        // ???????????????????????????
        if !self.is_done(env) {
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
            name: String::from("???????????????????????????"),
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
                FieldSquareType::Special {
                    player_id: 0,
                    activeted: false,
                },
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
    fn test_activates() {
        let env = Environment::new(2, 1, 1, 1);
        let card_catalog = vec![Card::new(1, "hoge", 1, CardShape::new("y"))];
        let mut cards = HashMap::new();
        cards.insert(1usize, &card_catalog[0]);
        let field = Field {
            id: 1,
            name: "hoge".to_string(),
            shape: FieldShape::new("YB\nbY"),
        };
        let mut state = State::new(&env, &cards, &field, &vec![vec![1], vec![1]]);
        let putted_this_turn_squares = vec![(1, 0)];
        let actual = state.activates(&putted_this_turn_squares);
        let expected = vec![2, 1];
        eprintln!("{}", state.field);
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
    #[test]
    fn test_get_cursor() {
        assert_eq!(get_cursor(0, 0, 1, 2, 3, 7), Some((4, 9)));
        assert_eq!(get_cursor(4, 0, 1, 2, 3, 7), Some((0, 9)));
        assert_eq!(get_cursor(4, 10, 1, 2, 3, 7), None);
    }
}
