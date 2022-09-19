use std::{ops::Deref, rc::Rc};

use crate::{
    board::{Game, Player},
    tile::{Position, TileIndex},
};

pub struct Agent {
    game: Rc<Game>,
    turn: Player,
}

const ENABLE_DEBUG: bool = false;

type Action = (TileIndex, Position);
const NULL_ACTION: Action = ((3, 3), Position::Right);

impl Agent {
    pub fn new(game: Rc<Game>, turn: Player) -> Agent {
        Self { game, turn }
    }

    pub fn ab_search(&mut self) -> (Action, i32) {
        let alpha = i32::MIN;
        let beta = i32::MAX;
        self.max(self.game.deref().clone(), alpha, beta)
    }

    fn max(&mut self, mut game: Game, mut alpha: i32, beta: i32) -> (Action, i32) {
        if self.turn != game.player_to_play() {
            panic!()
        }

        if game.ended() {
            if ENABLE_DEBUG {
                let k = game.utility(self.turn);
                Self::print_mv(&game, NULL_ACTION, k);
            }

            return (NULL_ACTION, game.utility(self.turn));
        }

        let mut action = NULL_ACTION;

        let mut v = i32::MIN;
        for (index, pos) in game.available_moves() {
            let mut new_state = game.clone();
            new_state.play(index, pos);

            let f = if game.player_to_play() == new_state.player_to_play() {
                Self::max
            } else {
                Self::min
            };

            let (_, val) = f(self, new_state, alpha, beta);

            if val > v {
                action = (index, pos);
                v = val;
            }

            if v >= beta {
                if ENABLE_DEBUG {
                    Self::print_mv(&game, (index, pos), v);
                }
                return ((index, pos), v);
            }

            if v > alpha {
                alpha = v;
            }
        }

        if ENABLE_DEBUG {
            Self::print_mv(&game, action, v);
        }

        (action, v)
    }

    fn min(&mut self, mut game: Game, alpha: i32, mut beta: i32) -> (Action, i32) {
        if self.turn == game.player_to_play() {
            panic!()
        }

        if game.ended() {
            if ENABLE_DEBUG {
                let k = game.utility(self.turn);
                Self::print_mv(&game, NULL_ACTION, k);
            }

            return (NULL_ACTION, game.utility(self.turn));
        }

        let mut action = NULL_ACTION;
        let mut v = i32::MAX;
        for (index, pos) in game.available_moves() {
            let mut new_state = game.clone();
            new_state.play(index, pos);

            let f = if game.player_to_play() == new_state.player_to_play() {
                Self::min
            } else {
                Self::max
            };

            let (_, val) = f(self, new_state, alpha, beta);

            if val < v {
                action = (index, pos);
                v = val;
            }

            if v <= alpha {
                if ENABLE_DEBUG {
                    Self::print_mv(&game, (index, pos), v);
                }

                return ((index, pos), v);
            }

            if v < beta {
                beta = v;
            }
        }

        if ENABLE_DEBUG {
            Self::print_mv(&game, action, v);
        }

        (action, v)
    }

    fn print_mv(game: &Game, mv: Action, value: i32) {
        game.print_board_without_pad();
        println!("Move: {:?}", mv);
        println!("Value: {}", value);
        println!("Turn: {:?}", game.player_to_play());
        println!();
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{
        board::{Game, Player},
        tile::{
            Position, BOTTOM_CENTER, BOTTOM_LEFT, BOTTOM_RIGHT, CENTER, MIDDLE_LEFT, MIDDLE_RIGHT,
            TOP_CENTER, TOP_LEFT, TOP_RIGHT,
        },
    };

    use super::Agent;

    #[test]
    fn a() {
        let mut game = Game::new();
        game.play(TOP_LEFT, Position::Top);
        game.play(TOP_LEFT, Position::Bottom);
        game.play(TOP_CENTER, Position::Bottom);
        game.play(TOP_CENTER, Position::Right);
        game.play(TOP_RIGHT, Position::Right);
        game.play(MIDDLE_LEFT, Position::Left);
        game.play(CENTER, Position::Bottom);
        game.play(MIDDLE_RIGHT, Position::Bottom);
        game.play(MIDDLE_RIGHT, Position::Right);
        game.play(BOTTOM_LEFT, Position::Right);
        game.play(BOTTOM_LEFT, Position::Left);

        // Agent
        // game.play(TOP_LEFT, Position::Right);
        // game.play(TOP_LEFT, Position::Left);
        // game.play(TOP_CENTER, Position::Top);
        // game.play(BOTTOM_RIGHT, Position::Right);
        // game.play(TOP_RIGHT, Position::Top);
        // game.play(MIDDLE_RIGHT, Position::Top);
        // game.play(MIDDLE_RIGHT, Position::Left);
        // game.play(MIDDLE_LEFT, Position::Right);
        // game.play(MIDDLE_LEFT, Position::Bottom);
        // game.play(BOTTOM_LEFT, Position::Bottom);

        // game.play(BOTTOM_CENTER, Position::Bottom);
        // game.play(BOTTOM_RIGHT, Position::Bottom);

        game.print_board();

        let mut agent = Agent::new(Rc::new(game), Player::Even);
        println!("{:?}", agent.ab_search())
    }

    #[test]
    fn b() {
        let mut game = Game::new();
        game.play(TOP_LEFT, Position::Top);
        game.play(TOP_CENTER, Position::Top);
        game.play(TOP_RIGHT, Position::Top);

        game.play(TOP_LEFT, Position::Bottom);
        game.play(TOP_CENTER, Position::Bottom);
        game.play(TOP_RIGHT, Position::Bottom);

        game.play(BOTTOM_LEFT, Position::Top);
        game.play(BOTTOM_CENTER, Position::Top);
        game.play(BOTTOM_RIGHT, Position::Top);

        game.play(TOP_LEFT, Position::Right);
        game.play(MIDDLE_LEFT, Position::Right);
        game.play(BOTTOM_LEFT, Position::Right);

        game.play(BOTTOM_LEFT, Position::Bottom);
        game.play(BOTTOM_RIGHT, Position::Right);

        game.play(TOP_LEFT, Position::Left);
        game.play(MIDDLE_LEFT, Position::Left);
        game.play(BOTTOM_LEFT, Position::Left);

        game.play(TOP_RIGHT, Position::Right);
        game.play(TOP_RIGHT, Position::Left);

        // Agent
        // game.play(MIDDLE_RIGHT, Position::Right);
        // game.play(MIDDLE_RIGHT, Position::Left);
        // game.play(BOTTOM_CENTER, Position::Bottom);

        // game.play(TOP_RIGHT, Position::Left);
        // game.play(MIDDLE_RIGHT, Position::Left);
        // game.play(MIDDLE_RIGHT, Position::Right);

        game.print_board();
        println!("Turn : {:?}", game.player_to_play());
        let mut agent = Agent::new(Rc::new(game), Player::Even);
        println!("{:?}", agent.ab_search())
    }
}
