use std::fmt::Display;

type Matrix<T> = Vec<Vec<T>>;

pub struct Board {
    h_lines: Matrix<bool>,
    v_lines: Matrix<bool>,
    square: Matrix<i32>,
    turn: i32,
}

impl Board {
    pub fn new() -> Self {
        Self {
            h_lines: vec![vec![false; 3]; 4],
            v_lines: vec![vec![false; 4]; 3],
            square: vec![vec![0; 3]; 3],
            turn: 1,
        }
    }

    pub fn place_h(&mut self, x: usize, y: usize) {
        self.h_lines[x][y] = true;
        self.check_h(x, y);
        self.alternate();
    }

    pub fn place_v(&mut self, x: usize, y: usize) {
        self.v_lines[x][y] = true;
        self.check_v(x, y);
        self.alternate();
    }

    pub fn utility(&self) -> i32 {
        todo!(
            "Use utility =
                S - O
                + sum(len(HLC) - 4) + sum(len(SC))
                - sum(4 - len(OLC)) + sum(len(LL) - 8)
                - 4 * SL - sum(len(OSC))"
        )
    }

    pub fn get_chains_and_loops(&self) {
        let flags = vec![vec![true; 3]; 3];
        todo!("Use expansion to discover chains and loops")
    }

    pub fn tile_at(&self, x: usize, y: usize) -> Tile {
        let top = self.h_lines[x][y];
        let bottom = self.h_lines[x][y + 1];
        let left = self.v_lines[x][y];
        let right = self.v_lines[x][y + 1];

        Tile::new(left, top, right, bottom)
    }

    fn alternate(&mut self) {
        self.turn = if self.turn == 1 { 2 } else { 1 };
    }

    fn check_h(&mut self, x: usize, y: usize) {
        // Check upper square
        if x != 0 {
            let top = self.h_lines[x - 1][y];
            let left = self.v_lines[x - 1][y];
            let right = self.v_lines[x - 1][y + 1];

            if top && left && right {
                self.square[x - 1][y] = self.turn;
            }
        }

        // Check lower square
        if x != 3 {
            let bottom = self.h_lines[x + 1][y];
            let left = self.v_lines[x][y];
            let right = self.v_lines[x][y + 1];

            if bottom && left && right {
                self.square[x][y] = self.turn;
            }
        }
    }

    fn check_v(&mut self, x: usize, y: usize) {
        // Check leftside square
        if y != 0 {
            let top = self.h_lines[x][y - 1];
            let bottom = self.h_lines[x + 1][y - 1];
            let left = self.v_lines[x][y - 1];

            if top && left && bottom {
                self.square[x][y - 1] = self.turn;
            }
        }

        // Check rightside square
        if y != 3 {
            let top = self.h_lines[x][y];
            let bottom = self.h_lines[x + 1][y];
            let right = self.v_lines[x][y + 1];

            if top && right && bottom {
                self.square[x][y] = self.turn;
            }
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..2 {
            writeln!(
                f,
                "+{}+{}+{}+",
                h_line(self.h_lines[i][0]),
                h_line(self.h_lines[i][1]),
                h_line(self.h_lines[i][2])
            )?;
            writeln!(
                f,
                "{} {} {} {} {} {} {}",
                v_line(self.v_lines[i][0]),
                sq(self.square[i][0]),
                v_line(self.v_lines[i][1]),
                sq(self.square[i][1]),
                v_line(self.v_lines[i][2]),
                sq(self.square[i][2]),
                v_line(self.v_lines[i][3]),
            )?;
        }
        writeln!(
            f,
            "+{}+{}+{}+",
            h_line(self.h_lines[3][0]),
            h_line(self.h_lines[3][1]),
            h_line(self.h_lines[3][2])
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

fn sq(v: i32) -> String {
    if v == 0 {
        " ".into()
    } else if v == 1 {
        "x".into()
    } else {
        "o".into()
    }
}

pub struct Tile {
    left: bool,
    right: bool,
    bottom: bool,
    top: bool,
}

impl Tile {
    pub fn new(left: bool, top: bool, right: bool, bottom: bool) -> Self {
        Self {
            left,
            right,
            bottom,
            top,
        }
    }
}
