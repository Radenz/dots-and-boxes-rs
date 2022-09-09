use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::tile::{Chain, ChainBuilder, Loop, LoopBuilder, Position, Tile, TileIndex, POSITIONS};

type Matrix<T> = Vec<Vec<T>>;

pub struct Board {
    tiles: Matrix<Rc<RefCell<Tile>>>,
}

impl Board {
    pub fn new() -> Self {
        let mut tiles = Vec::new();
        for x in 0..3 {
            let mut row = Vec::new();
            for y in 0..3 {
                row.push(Rc::new(RefCell::new(Tile::new((x, y)))));
            }
            tiles.push(row);
        }

        Self { tiles }
    }

    pub fn mark(&mut self, index: TileIndex, pos: Position) {
        let tile = self.get_tile(index);

        tile.borrow_mut().mark(pos);

        if tile.borrow_mut().has_neighbor(pos) {
            let neighbor_index = tile.borrow_mut().at_unchecked(pos);
            let neighbor = self.get_tile(neighbor_index);
            neighbor.borrow_mut().mark(pos.invert());
        }
    }

    fn get_tile(&mut self, index: TileIndex) -> Rc<RefCell<Tile>> {
        self.tiles
            .get_mut(index.0)
            .unwrap()
            .get_mut(index.1)
            .unwrap()
            .clone()
    }

    pub fn get_chains(&mut self) -> Vec<Chain> {
        let mut has_evaluated = vec![vec![false; 3]; 3];
        let mut chains = vec![];

        for x in 0..3 {
            for y in 0..3 {
                if !has_evaluated[x][y] {
                    has_evaluated[x][y] = true;

                    let tile = self.get_tile((x, y));
                    let tile_ref = tile.borrow();

                    if tile_ref.is_end() || tile_ref.is_edge_path_chain_end() {
                        // Guaranteed to have length 1
                        let mut builder = ChainBuilder::new(tile.clone());

                        let mut next_chain_tile = self.next_chain_tile(&tile, None);

                        let mut chaining = false;
                        while let Some((tile, pos)) = next_chain_tile {
                            chaining = true;
                            let index = tile.borrow().index();

                            if has_evaluated[index.0][index.1] {
                                // Chain is a loop
                                break;
                            }

                            has_evaluated[index.0][index.1] = true;
                            builder.add(&tile);
                            next_chain_tile = self.next_chain_tile(&tile, Some(pos));
                        }

                        if chaining {
                            chains.push(builder.build());
                        }
                    } else {
                        has_evaluated[x][y] = false;
                    }
                }
            }
        }

        chains
    }

    pub fn get_loops(&mut self) -> Vec<Loop> {
        let mut has_evaluated = vec![vec![false; 3]; 3];
        let mut loops = vec![];
        let mut indices = vec![];

        for x in 0..3 {
            for y in 0..3 {
                if !has_evaluated[x][y] {
                    has_evaluated[x][y] = true;

                    let mut tile = self.get_tile((x, y));
                    let tile_ref = tile.borrow();

                    if !tile_ref.is_path() {
                        continue;
                    }

                    let mut last_pos = None;

                    drop(tile_ref);

                    // let tile_loop = vec![];
                    let mut builder = LoopBuilder::new(tile.clone());
                    let mut is_loop = false;

                    loop {
                        let nb = self.get_connected_neighbor(tile.clone(), last_pos);

                        if let Some((neighbor, lp)) = nb {
                            tile = neighbor;
                            last_pos = Some(lp);

                            let tile_index = tile.borrow().index();
                            has_evaluated[tile_index.0][tile_index.1] = true;
                            if tile_index == (x, y) {
                                is_loop = true;
                                break;
                            }

                            indices.push(tile.borrow().index());
                            if !tile.borrow().is_path() {
                                break;
                            }
                            builder.add(&tile);
                        } else {
                            break;
                        }
                    }

                    // Reset has eval if loop finding failed
                    if is_loop {
                        loops.push(builder.build());
                    } else {
                        for &(x, y) in indices.iter() {
                            has_evaluated[x][y] = false;
                        }
                    }
                }
            }
        }

        loops
    }

    fn get_connected_neighbor(
        &mut self,
        tile: Rc<RefCell<Tile>>,
        without: Option<Position>,
    ) -> Option<(Rc<RefCell<Tile>>, Position)> {
        for &pos in POSITIONS.iter() {
            if let Some(igored_pos) = without {
                if pos == igored_pos.invert() {
                    continue;
                }
            }

            let index = tile.borrow().at(pos);

            if let Some(neighbor_index) = index {
                let neighbor = self.get_tile(neighbor_index);
                if tile.borrow().connected_to(&neighbor) {
                    return Some((neighbor, pos));
                }
            }
        }

        None
    }

    /// last_tile: the last tile in chain
    /// last_pos: last position to get the last tile in chain
    /// TODO: refactor
    fn next_chain_tile(
        &mut self,
        last_tile: &Rc<RefCell<Tile>>,
        last_pos: Option<Position>,
    ) -> Option<(Rc<RefCell<Tile>>, Position)> {
        // Assume last_tile has either 1 or 2 openings
        let last_tile_ref = last_tile.borrow();
        let mut openings = last_tile_ref.openings();

        if openings.len() == 1 {
            // Last tile is either start or end of chain

            if let Some(_) = last_pos {
                // This is not the first tile in the chain

                // End of chain
                return None;
            } else {
                // This wiill be `last_pos` in the next iteration
                let &next_tile_pos = openings.first().unwrap();
                let next_tile_index = last_tile_ref.at_unchecked(next_tile_pos);

                let next_tile = self.get_tile(next_tile_index);

                return if next_tile.borrow().can_be_chained() {
                    Some((next_tile, next_tile_pos))
                } else {
                    None
                };
            }
        } else {
            // Last tile is a path, openings must contain 2 pos
            // Guaranteed to have last pos because last_tile is path
            if let Some(pos) = last_pos {
                if *openings.get(0).unwrap() == pos.invert() {
                    openings.remove(0);
                } else {
                    openings.remove(1);
                }

                let &next_tile_pos = openings.first().unwrap();

                if let Some(next_tile_index) = last_tile_ref.at(next_tile_pos) {
                    let next_tile = self.get_tile(next_tile_index);

                    if next_tile.borrow().can_be_chained() {
                        return Some((next_tile, next_tile_pos));
                    }
                }

                return None;
            } else {
                // Last tile is path, and it is the first tile in the chain
                if last_tile_ref.is_edge_path_chain_end() {
                    let next_tile_pos = last_tile_ref.get_edge_path_chain_pos();
                    let next_tile_index = last_tile_ref.at_unchecked(next_tile_pos);

                    let next_tile = self.get_tile(next_tile_index);

                    if next_tile.borrow().can_be_chained() {
                        return Some((next_tile, next_tile_pos));
                    }
                }
                return None;
            }
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..3 {
            writeln!(
                f,
                "+{}+{}+{}+",
                h_line(!self.tiles[i][0].borrow().is_open(Position::Top)),
                h_line(!self.tiles[i][1].borrow().is_open(Position::Top)),
                h_line(!self.tiles[i][2].borrow().is_open(Position::Top))
            )?;
            writeln!(
                f,
                "{} {} {} {} {} {} {}",
                v_line(!self.tiles[i][0].borrow().is_open(Position::Left)),
                " ",
                v_line(!self.tiles[i][1].borrow().is_open(Position::Left)),
                " ",
                v_line(!self.tiles[i][2].borrow().is_open(Position::Left)),
                " ",
                v_line(!self.tiles[i][2].borrow().is_open(Position::Right)),
            )?;
        }
        writeln!(
            f,
            "+{}+{}+{}+",
            h_line(!self.tiles[2][0].borrow().is_open(Position::Bottom)),
            h_line(!self.tiles[2][1].borrow().is_open(Position::Bottom)),
            h_line(!self.tiles[2][2].borrow().is_open(Position::Bottom))
        )?;

        Ok(())
    }
}

fn h_line(cond: bool) -> String {
    if cond {
        "---".into()
    } else {
        "   ".into()
    }
}

fn v_line(cond: bool) -> String {
    if cond {
        "|".into()
    } else {
        " ".into()
    }
}

// pub struct Board {
//     h_lines: Matrix<bool>,
//     v_lines: Matrix<bool>,
//     square: Matrix<i32>,
//     turn: i32,
// }

// impl Board {
//     pub fn new() -> Self {
//         Self {
//             h_lines: vec![vec![false; 3]; 4],
//             v_lines: vec![vec![false; 4]; 3],
//             square: vec![vec![0; 3]; 3],
//             turn: 1,
//         }
//     }

//     pub fn place_h(&mut self, x: usize, y: usize) {
//         self.h_lines[x][y] = true;
//         self.check_h(x, y);
//         self.alternate();
//     }

//     pub fn place_v(&mut self, x: usize, y: usize) {
//         self.v_lines[x][y] = true;
//         self.check_v(x, y);
//         self.alternate();
//     }

//     pub fn utility(&self) -> i32 {
//         todo!(
//             "Use utility =
//                 S - O
//                 + sum(len(HLC) - 4) + sum(len(SC))
//                 - sum(4 - len(OLC)) + sum(len(LL) - 8)
//                 - 4 * SL - sum(len(OSC))"
//         )
//     }

//     pub fn get_chains_and_loops(&self) {
//         let mut flags = vec![vec![false; 3]; 3];
//         let mut chains: Vec<Vec<MatrixIndex>> = vec![];

//         for i in 0..3 {
//             for j in 0..3 {
//                 if !flags[i][j] {
//                     flags[i][j] = true;

//                     let mut tile = self.tile_at(i, j);

//                     if tile.is_single_open_ended() {
//                         if tile.single_open_to_outside() {
//                             continue;
//                         }
//                         let dir = tile.get_single_open_end_dir().unwrap();
//                         let index = tile.at(dir);
//                         let first_tile = tile;
//                         tile = self.tile_at(index.0, index.1);

//                         // We get a chain of at least length 2
//                         if tile.is_double_open_ended() {
//                             // let chain = vec![first_tile, tile];

//                             while tile.is_double_open_ended() {}

//                             // chains.push(chain);
//                         }
//                     }
//                 }
//             }
//         }

//         todo!("Use expansion to discover chains and loops")
//     }

//     pub fn tile_at(&self, x: usize, y: usize) -> Tile {
//         let top = self.h_lines[x][y];
//         let bottom = self.h_lines[x][y + 1];
//         let left = self.v_lines[x][y];
//         let right = self.v_lines[x][y + 1];

//         Tile::new(x, y, left, top, right, bottom)
//     }

//     fn alternate(&mut self) {
//         self.turn = if self.turn == 1 { 2 } else { 1 };
//     }

//     fn check_h(&mut self, x: usize, y: usize) {
//         // Check upper square
//         if x != 0 {
//             let top = self.h_lines[x - 1][y];
//             let left = self.v_lines[x - 1][y];
//             let right = self.v_lines[x - 1][y + 1];

//             if top && left && right {
//                 self.square[x - 1][y] = self.turn;
//             }
//         }

//         // Check lower square
//         if x != 3 {
//             let bottom = self.h_lines[x + 1][y];
//             let left = self.v_lines[x][y];
//             let right = self.v_lines[x][y + 1];

//             if bottom && left && right {
//                 self.square[x][y] = self.turn;
//             }
//         }
//     }

//     fn check_v(&mut self, x: usize, y: usize) {
//         // Check leftside square
//         if y != 0 {
//             let top = self.h_lines[x][y - 1];
//             let bottom = self.h_lines[x + 1][y - 1];
//             let left = self.v_lines[x][y - 1];

//             if top && left && bottom {
//                 self.square[x][y - 1] = self.turn;
//             }
//         }

//         // Check rightside square
//         if y != 3 {
//             let top = self.h_lines[x][y];
//             let bottom = self.h_lines[x + 1][y];
//             let right = self.v_lines[x][y + 1];

//             if top && right && bottom {
//                 self.square[x][y] = self.turn;
//             }
//         }
//     }
// }

// impl Display for Board {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         for i in 0..2 {
//             writeln!(
//                 f,
//                 "+{}+{}+{}+",
//                 h_line(self.h_lines[i][0]),
//                 h_line(self.h_lines[i][1]),
//                 h_line(self.h_lines[i][2])
//             )?;
//             writeln!(
//                 f,
//                 "{} {} {} {} {} {} {}",
//                 v_line(self.v_lines[i][0]),
//                 sq(self.square[i][0]),
//                 v_line(self.v_lines[i][1]),
//                 sq(self.square[i][1]),
//                 v_line(self.v_lines[i][2]),
//                 sq(self.square[i][2]),
//                 v_line(self.v_lines[i][3]),
//             )?;
//         }
//         writeln!(
//             f,
//             "+{}+{}+{}+",
//             h_line(self.h_lines[3][0]),
//             h_line(self.h_lines[3][1]),
//             h_line(self.h_lines[3][2])
//         )?;

//         Ok(())
//     }
// }

// fn h_line(cond: bool) -> String {
//     if cond {
//         "---".into()
//     } else {
//         "   ".into()
//     }
// }

// fn v_line(cond: bool) -> String {
//     if cond {
//         "|".into()
//     } else {
//         " ".into()
//     }
// }

// fn sq(v: i32) -> String {
//     if v == 0 {
//         " ".into()
//     } else if v == 1 {
//         "x".into()
//     } else {
//         "o".into()
//     }
// }

// pub struct Tile {
//     x: usize,
//     y: usize,
//     left: bool,
//     right: bool,
//     bottom: bool,
//     top: bool,
// }

// impl Tile {
//     pub fn new(x: usize, y: usize, left: bool, top: bool, right: bool, bottom: bool) -> Self {
//         Self {
//             x,
//             y,
//             left,
//             right,
//             bottom,
//             top,
//         }
//     }

//     pub fn is_single_open_ended(&self) -> bool {
//         self.sum() == 1
//     }

//     pub fn is_double_open_ended(&self) -> bool {
//         self.sum() == 2
//     }

//     pub fn x(&self) -> usize {
//         self.x
//     }

//     pub fn y(&self) -> usize {
//         self.y
//     }

//     pub fn get_single_open_end_dir(&self) -> Option<Direction> {
//         if self.is_single_open_ended() {
//             if self.left {
//                 return Some(Direction::Left);
//             }

//             if self.right {
//                 return Some(Direction::Right);
//             }

//             if self.top {
//                 return Some(Direction::Up);
//             }

//             if self.bottom {
//                 return Some(Direction::Down);
//             }
//         }

//         None
//     }

//     pub fn get_double_open_end_dir(&self) -> Option<(Direction, Direction)> {
//         if self.is_double_open_ended() {
//             let mut v = vec![];

//             if self.left {
//                 v.push(Direction::Left);
//             }

//             if self.right {
//                 v.push(Direction::Right);
//             }

//             if self.top {
//                 v.push(Direction::Up);
//             }

//             if self.bottom {
//                 v.push(Direction::Down);
//             }

//             return Some((v[0], v[1]));
//         }

//         None
//     }

//     pub fn single_open_to_outside(&self) -> bool {
//         let dir = self.get_single_open_end_dir().unwrap();
//         match dir {
//             Direction::Left => self.at_left(),
//             Direction::Right => self.at_right(),
//             Direction::Up => self.at_top(),
//             Direction::Down => self.at_bottom(),
//         }
//     }

//     pub fn at(&self, dir: Direction) -> MatrixIndex {
//         let diff = dir.to_diff();
//         (self.x + diff.0 as usize, self.y + diff.0 as usize)
//     }

//     pub fn index(&self) -> MatrixIndex {
//         todo!()
//     }

//     fn sum(&self) -> i32 {
//         self.left as i32 + self.right as i32 + self.top as i32 + self.bottom as i32
//     }

//     fn at_top(&self) -> bool {
//         self.x == 0
//     }

//     fn at_bottom(&self) -> bool {
//         self.x == 2
//     }

//     fn at_left(&self) -> bool {
//         self.y == 0
//     }

//     fn at_right(&self) -> bool {
//         self.y == 2
//     }
// }

// #[derive(Debug, Clone, Copy, PartialEq)]
// pub enum Direction {
//     Up,
//     Down,
//     Left,
//     Right,
// }

// impl Direction {
//     pub fn to_diff(&self) -> (i32, i32) {
//         match *self {
//             Self::Up => (-1, 0),
//             Self::Down => (1, 0),
//             Self::Left => (0, -1),
//             Self::Right => (0, 1),
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use crate::tile::{Position, BOTTOM_RIGHT, CENTER, MIDDLE_LEFT, TOP_CENTER, TOP_LEFT};

    use super::Board;

    #[test]
    fn print() {
        println!("{}", Board::new());
        println!();
        println!();

        let mut board = Board::new();
        board.mark((0, 0), Position::Top);
        board.mark((0, 0), Position::Left);
        board.mark((0, 0), Position::Right);
        board.mark((0, 0), Position::Bottom);
        println!("{}", board);
        println!();
        println!();

        board.mark((0, 1), Position::Right);
        board.mark((0, 2), Position::Top);
        board.mark((0, 2), Position::Right);
        board.mark((0, 2), Position::Bottom);
        println!("{}", board);
    }

    #[test]
    fn chain() {
        let mut board = Board::new();
        board.mark((0, 0), Position::Top);
        board.mark((0, 0), Position::Bottom);
        board.mark((0, 1), Position::Bottom);
        board.mark((0, 1), Position::Right);
        board.mark((0, 2), Position::Right);
        board.mark((1, 0), Position::Left);
        board.mark((1, 1), Position::Bottom);
        board.mark((1, 2), Position::Bottom);
        board.mark((1, 2), Position::Right);
        board.mark((2, 0), Position::Right);
        board.mark((2, 0), Position::Left);
        board.mark((2, 2), Position::Bottom);
        println!("{}", board);
        println!();
        println!();

        println!("Chains = {}", board.get_chains().len());
    }

    #[test]
    fn loops() {
        let mut board = Board::new();
        board.mark(TOP_LEFT, Position::Top);
        board.mark(TOP_LEFT, Position::Left);
        board.mark(TOP_CENTER, Position::Top);
        board.mark(TOP_CENTER, Position::Right);
        board.mark(MIDDLE_LEFT, Position::Left);
        board.mark(MIDDLE_LEFT, Position::Bottom);
        board.mark(CENTER, Position::Bottom);
        board.mark(CENTER, Position::Right);
        // clutter
        board.mark(BOTTOM_RIGHT, Position::Right);
        board.mark(BOTTOM_RIGHT, Position::Bottom);
        println!("{}", board);
        println!();
        println!();

        println!("Loops = {}", board.get_loops().len());
    }
}
