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

    pub fn acquisitions(&self) -> Matrix<bool> {
        self.tiles
            .iter()
            .map(|row| row.iter().map(|tile| tile.borrow().all_marked()).collect())
            .collect()
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

    pub fn free_edge_squares(&mut self) -> i32 {
        let mut sq = 0;

        for x in 0..3 {
            for y in 0..3 {
                let tile = self.get_tile((x, y));
                let tile_ref = tile.borrow();

                if tile_ref.is_end() {
                    let opening = tile_ref.openings()[0];
                    if !tile_ref.has_neighbor(opening) {
                        sq += 1;
                    }
                }
            }
        }

        sq
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

    pub fn safe_moves_count(&mut self) -> i32 {
        let mut safe_moves = 0;

        for x in 0..3 {
            for y in 0..3 {
                let index = (x, y);

                for &pos in POSITIONS.iter() {
                    safe_moves += 1;

                    if x == 1 && pos.is_vertical() {
                        continue;
                    }

                    if y == 1 && pos.is_horizontal() {
                        continue;
                    }

                    if self.will_make_end(index, pos) {
                        safe_moves -= 1;
                    }
                }
            }
        }

        safe_moves
    }

    fn will_make_end(&mut self, mark_index: TileIndex, mark_pos: Position) -> bool {
        let tile = self.get_tile(mark_index);
        let tile_ref = tile.borrow();

        if tile_ref.is_path() && tile_ref.is_open(mark_pos) {
            return true;
        }

        if tile_ref.has_neighbor(mark_pos) {
            let neighbor = self.get_tile(tile_ref.at_unchecked(mark_pos));
            let neighbor_ref = neighbor.borrow();

            if neighbor_ref.is_path() && neighbor_ref.is_open(mark_pos.invert()) {
                return true;
            }
        }

        false
    }
}

impl Clone for Board {
    fn clone(&self) -> Board {
        let mut tiles = Vec::new();
        for x in 0..3 {
            let mut row = Vec::new();

            for y in 0..3 {
                let tile = &self.tiles[x][y];
                row.push(Rc::new(RefCell::new(tile.borrow().clone())));
            }

            tiles.push(row);
        }

        Board { tiles }
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

pub struct Game {
    board: Board,
    turn: Player,
    squares: Matrix<Option<Player>>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            turn: Player::Odd,
            squares: vec![
                vec![None, None, None],
                vec![None, None, None],
                vec![None, None, None],
            ],
        }
    }

    pub fn play(&mut self, index: TileIndex, pos: Position) {
        self.board.mark(index, pos);

        let acquired_squares = self.board.acquisitions();
        for x in 0..3 {
            for y in 0..3 {
                if let None = self.squares[x][y] {
                    if acquired_squares[x][y] {
                        self.squares[x][y] = Some(self.turn);
                    }
                }
            }
        }

        self.switch();
    }

    // Calculate board setup utility value on certain player perspective
    pub fn utility(&mut self, player: Player) -> i32 {
        let chains = self.board.get_chains();
        let loops = self.board.get_loops();

        let mut chain_values = 0;
        let this_player_to_move = self.turn == player;

        let factor = -1_i32.pow(self.board.safe_moves_count() as u32 + 1) * {
            if this_player_to_move {
                1
            } else {
                -1
            }
        };

        for chain in chains.iter() {
            if !chain.is_long() {
                // 2-chain
                if chain.is_closed() {
                    chain_values += 2;
                } else {
                    chain_values -= 2;
                }
            } else {
                // long chain
                if chain.is_closed() {
                    chain_values += chain.len()
                } else if chain.is_half_open() {
                    chain_values += chain.len() - 4;
                } else {
                    chain_values -= 4 - chain.len();
                }
            }
        }

        let mut loop_values = 0;
        for _loop in loops.iter() {
            loop_values -= _loop.len();
        }

        self.acquired_squares(player) - self.acquired_squares(player.opponent())
            + (chain_values as i32 + loop_values as i32 + self.board.free_edge_squares()) * factor
    }

    fn acquired_squares(&self, player: Player) -> i32 {
        let mut s = 0;
        for row in self.squares.iter() {
            for &sq in row.iter() {
                if let Some(p) = sq {
                    if p == player {
                        s += 1;
                    }
                }
            }
        }

        s
    }

    fn switch(&mut self) {
        self.turn = self.turn.opponent();
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Player {
    Odd,
    Even,
}

impl Player {
    pub fn opponent(&self) -> Player {
        match *self {
            Self::Odd => Self::Even,
            Self::Even => Self::Odd,
        }
    }
}
