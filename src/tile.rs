use std::{cell::RefCell, rc::Rc};

pub type TileIndex = (usize, usize);

pub struct Tile {
    index: TileIndex,
    config: TileConfig,
}

impl Tile {
    pub fn new(index: TileIndex) -> Self {
        Self {
            index,
            config: TileConfig::new(),
        }
    }

    pub fn index(&self) -> TileIndex {
        self.index
    }

    pub fn is_end(&self) -> bool {
        self.config.open_count() == 1
    }

    pub fn is_path(&self) -> bool {
        self.config.open_count() == 2
    }

    pub fn can_be_chained(&self) -> bool {
        self.is_end() || self.is_path()
    }

    pub fn has_neighbor(&self, pos: Position) -> bool {
        match pos {
            Position::Top => self.has_top_neighbor(),
            Position::Bottom => self.has_bottom_neighbor(),
            Position::Left => self.has_left_neighbor(),
            Position::Right => self.has_right_neighbor(),
        }
    }

    fn has_top_neighbor(&self) -> bool {
        self.index.0 != 0
    }

    fn has_bottom_neighbor(&self) -> bool {
        self.index.0 != 2
    }

    fn has_left_neighbor(&self) -> bool {
        self.index.1 != 0
    }

    fn has_right_neighbor(&self) -> bool {
        self.index.1 != 2
    }

    pub fn mark(&mut self, pos: Position) {
        match pos {
            Position::Top => self.config.mark_top(),
            Position::Bottom => self.config.mark_bottom(),
            Position::Left => self.config.mark_left(),
            Position::Right => self.config.mark_right(),
        }
    }

    pub fn at(&self, pos: Position) -> Option<TileIndex> {
        let mut x = self.index.0;
        let mut y = self.index.1;

        match pos {
            Position::Top => {
                if x == 0 {
                    return None;
                } else {
                    x -= 1;
                }
            }
            Position::Bottom => {
                if x == 2 {
                    return None;
                } else {
                    x += 1;
                }
            }
            Position::Left => {
                if y == 0 {
                    return None;
                } else {
                    y -= 1;
                }
            }
            Position::Right => {
                if y == 2 {
                    return None;
                } else {
                    y += 1;
                }
            }
        };

        Some((x, y))
    }

    pub fn at_unchecked(&self, pos: Position) -> TileIndex {
        self.at(pos).unwrap()
    }

    pub fn adjacent_to(&self, other: &RefCell<Tile>) -> bool {
        let x_diff = self.index.0.abs_diff(other.borrow().index.0);
        let y_diff = self.index.1.abs_diff(other.borrow().index.1);

        x_diff + y_diff == 1
    }

    // Calculate other relative position to self
    pub fn relative_position(&self, other: &RefCell<Self>) -> Option<Position> {
        for &pos in POSITIONS.iter() {
            if let Some(index) = self.at(pos) {
                if index == other.borrow().index {
                    return Some(pos);
                }
            }
        }

        None
    }

    pub fn openings(&self) -> Vec<Position> {
        let mut openings = vec![];

        for &pos in POSITIONS.iter() {
            if self.config.is_open(pos) {
                openings.push(pos);
            }
        }

        openings
    }

    pub fn is_in_edge(&self) -> bool {
        self.index != (1, 1)
    }

    pub fn is_edge_path_chain_end(&self) -> bool {
        if !self.is_path() || !self.is_in_edge() {
            return false;
        }

        let openings = self.openings();
        let opening1 = openings[0];
        let opening2 = openings[1];

        self.opening_in_edge(opening1) ^ self.opening_in_edge(opening2)
    }

    fn opening_in_edge(&self, pos: Position) -> bool {
        match pos {
            Position::Top => self.index.0 == 0,
            Position::Bottom => self.index.0 == 2,
            Position::Left => self.index.1 == 0,
            Position::Right => self.index.1 == 2,
        }
    }

    pub fn get_edge_path_chain_pos(&self) -> Position {
        let openings = self.openings();
        let opening1 = openings[0];
        let opening2 = openings[1];

        if self.has_neighbor(opening1) {
            opening1
        } else {
            opening2
        }
    }

    pub fn connected_to(&self, other: &RefCell<Tile>) -> bool {
        let relpos = self.relative_position(other);

        if let Some(pos) = relpos {
            return self.config.is_open(pos);
        }

        false
    }

    pub fn is_open(&self, pos: Position) -> bool {
        self.config.is_open(pos)
    }

    pub fn open_to_outside(&self) -> bool {
        for &pos in POSITIONS.iter() {
            if self.is_open(pos) && !self.has_neighbor(pos) {
                return true;
            }
        }

        false
    }
}

struct TileConfig {
    top: bool,
    bottom: bool,
    left: bool,
    right: bool,
}

impl TileConfig {
    pub fn new() -> Self {
        Self {
            top: false,
            bottom: false,
            left: false,
            right: false,
        }
    }

    #[allow(dead_code)]
    pub fn of(top: bool, bottom: bool, left: bool, right: bool) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    pub fn open_count(&self) -> i32 {
        self.top as i32 + self.bottom as i32 + self.left as i32 + self.right as i32
    }

    pub fn is_open(&self, pos: Position) -> bool {
        match pos {
            Position::Top => !self.top,
            Position::Bottom => !self.bottom,
            Position::Left => !self.left,
            Position::Right => !self.right,
        }
    }

    pub fn mark_top(&mut self) {
        self.top = true;
    }

    pub fn mark_bottom(&mut self) {
        self.bottom = true;
    }

    pub fn mark_left(&mut self) {
        self.left = true;
    }

    pub fn mark_right(&mut self) {
        self.right = true;
    }
}

pub struct TilePath {
    first_dir: Direction,
    second_dir: Direction,
}

impl TilePath {
    pub fn new(first_dir: Direction, second_dir: Direction) -> Self {
        Self {
            first_dir,
            second_dir,
        }
    }

    pub fn without(&self, dir: Direction) -> Option<Direction> {
        if dir == self.first_dir {
            return Some(self.second_dir);
        }

        if dir == self.second_dir {
            return Some(self.first_dir);
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    Top,
    Bottom,
    Left,
    Right,
}

impl Position {
    pub fn invert(&self) -> Position {
        match *self {
            Self::Top => Position::Bottom,
            Self::Bottom => Position::Top,
            Self::Left => Position::Right,
            Self::Right => Position::Left,
        }
    }
}

pub const POSITIONS: [Position; 4] = [
    Position::Top,
    Position::Bottom,
    Position::Right,
    Position::Left,
];

#[allow(dead_code)]
pub struct Chain {
    first_end: (Rc<RefCell<Tile>>, Option<Position>),
    second_end: (Rc<RefCell<Tile>>, Option<Position>),
    tiles: Vec<Rc<RefCell<Tile>>>,
}

impl Chain {}

pub struct ChainBuilder {
    tiles: Vec<Rc<RefCell<Tile>>>,
    first_end_pos: Option<Position>,
    second_end_pos: Option<Position>,
}

impl ChainBuilder {
    pub fn new(first: Rc<RefCell<Tile>>) -> Self {
        Self {
            tiles: vec![first],
            first_end_pos: None,
            second_end_pos: None,
        }
    }

    pub fn add(&mut self, tile: &Rc<RefCell<Tile>>) {
        let tile_ref = tile.borrow();
        if !tile_ref.is_end() && !tile_ref.is_path() {
            panic!()
        }

        let last = self.tiles.last().unwrap();
        let last_ref = last.borrow();

        if !last_ref.adjacent_to(tile) {
            panic!()
        }

        if self.tiles.len() == 1 {
            if last_ref.is_path() {
                let mut openings = last_ref.openings();
                let relpos = last.borrow().relative_position(tile).unwrap();

                if *openings.get(0).unwrap() == relpos {
                    openings.remove(0);
                } else {
                    openings.remove(1);
                }

                self.first_end_pos = Some(*openings.get(0).unwrap());
            }
        }

        drop(last_ref);

        self.tiles.push(tile.clone());
    }

    // Build the chain, consume the builder
    pub fn build(mut self) -> Chain {
        let len = self.tiles.len();

        if len == 1 {
            panic!()
        }

        // let last = self.tiles.last().unwrap();
        let last = self.tiles.last().unwrap().borrow();
        let second_last = self.tiles.get(len - 2).unwrap();

        if last.is_path() {
            let mut openings = last.openings();
            let relpos = last.relative_position(&second_last).unwrap();

            if *openings.get(0).unwrap() == relpos {
                openings.remove(0);
            } else {
                openings.remove(1);
            }

            self.second_end_pos = Some(*openings.get(0).unwrap());
        }

        drop(last);

        let first = self.tiles.first().unwrap().clone();
        let second = self.tiles.last().unwrap().clone();

        Chain {
            first_end: (first, self.first_end_pos),
            second_end: (second, self.second_end_pos),
            tiles: self.tiles,
        }
    }
}

#[allow(dead_code)]
pub struct Loop {
    tiles: Vec<Rc<RefCell<Tile>>>,
}

pub struct LoopBuilder {
    tiles: Vec<Rc<RefCell<Tile>>>,
}

impl LoopBuilder {
    pub fn new(first: Rc<RefCell<Tile>>) -> Self {
        Self { tiles: vec![first] }
    }

    pub fn add(&mut self, tile: &Rc<RefCell<Tile>>) {
        let tile_ref = tile.borrow();
        if !tile_ref.is_path() {
            panic!()
        }

        let last = self.tiles.last().unwrap();
        let last_ref = last.borrow();

        if !last_ref.connected_to(tile) {
            panic!()
        }

        drop(last_ref);

        self.tiles.push(tile.clone());
    }

    // Build the loop, consume the builder
    pub fn build(self) -> Loop {
        let len = self.tiles.len();

        if len < 4 {
            panic!()
        }

        // let last = self.tiles.last().unwrap();
        let last = self.tiles.last().unwrap();
        let first = self.tiles.first().unwrap();
        if !first.borrow().connected_to(last) {
            panic!()
        }

        Loop { tiles: self.tiles }
    }
}

pub const TOP_LEFT: TileIndex = (0, 0);
pub const TOP_CENTER: TileIndex = (0, 1);
pub const TOP_RIGHT: TileIndex = (0, 2);
pub const MIDDLE_LEFT: TileIndex = (1, 0);
pub const CENTER: TileIndex = (1, 1);
pub const MIDDLE_RIGHT: TileIndex = (1, 2);
pub const BOTTOM_LEFT: TileIndex = (2, 0);
pub const BOTTOM_CENTER: TileIndex = (2, 1);
pub const BOTTOM_RIGHT: TileIndex = (2, 2);
