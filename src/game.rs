use std::ops::Neg;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entity {
    O,
    X,
    Empty,
}
use Entity::*;

impl Entity {
    pub fn from_char(c: char) -> Self {
        match c {
            'o' | 'O' => O,
            'x' | 'X' => X,
            _ => Empty,
        }
    }
    pub fn into_char(&self) -> char {
        match self {
            O => 'o',
            X => 'x',
            _ => '.',
        }
    }
}

impl Neg for Entity {
    type Output = Entity;
    fn neg(self) -> Self {
        match self {
            O => X,
            X => O,
            Empty => Empty,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Put(Entity, (usize, usize)),
    Pass,
}

#[derive(Debug, Clone)]
pub struct Game {
    pub next: Entity,
    pub height: usize,
    pub width: usize,
    pub data: Vec<Vec<Entity>>,
}

impl Game {
    pub fn read(next: Entity) -> Self {
        let stdin = std::io::stdin();
        let mut buffer: Vec<Vec<char>> = vec![];
        loop {
            let mut line = String::new();
            match stdin.read_line(&mut line) {
                Ok(x) if x > 0 => {
                    buffer.push(line.trim().chars().collect());
                }
                _ => break,
            }
        }
        let height = buffer.len();
        let width = buffer[0].len();
        let data: Vec<Vec<_>> = buffer
            .iter()
            .map(|line| line.iter().cloned().map(Entity::from_char).collect())
            .collect();
        Self {
            next,
            height,
            width,
            data,
        }
    }

    pub fn write(&self) {
        for line in self.data.iter() {
            println!("{}", line.iter().map(Entity::into_char).collect::<String>());
        }
    }

    const DEGS: [(isize, isize); 8] = [
        (1, 1),
        (1, 0),
        (1, -1),
        (0, 1),
        (0, -1),
        (-1, 1),
        (-1, 0),
        (-1, -1),
    ];

    /// mv したときにひっくり返せる (dx, dy) 方向のライン上のコマ
    fn reversal_line(&self, mv: Move, dx: isize, dy: isize) -> Option<Vec<(usize, usize)>> {
        match mv {
            Move::Pass => None,
            Move::Put(next, (x, y)) => {
                if self.data[x][y] != Empty {
                    return None;
                }
                let mut x = x;
                let mut y = y;
                let mut line = vec![];
                loop {
                    let ix = x as isize + dx;
                    let iy = y as isize + dy;
                    if ix < 0 || iy < 0 {
                        return None;
                    }
                    x = ix as usize;
                    y = iy as usize;
                    if x >= self.height || y >= self.width {
                        return None;
                    }
                    match self.data[x][y] {
                        Empty => return None,
                        c if c == next => {
                            if line.len() > 0 {
                                return Some(line);
                            } else {
                                return None;
                            }
                        }
                        _ => {
                            line.push((x, y));
                        }
                    }
                }
            }
        }
    }

    /// 次おける場所全て
    pub fn puttables(&self, next: Entity) -> Vec<(usize, usize)> {
        let mut r = vec![];
        for i in 0..self.height {
            for j in 0..self.width {
                let mut ok = false;
                for &(dx, dy) in Game::DEGS.iter() {
                    if self
                        .reversal_line(Move::Put(next, (i, j)), dx, dy)
                        .is_some()
                    {
                        ok = true;
                        break;
                    }
                }
                if ok {
                    r.push((i, j));
                }
            }
        }
        r
    }

    /// 次に打てる手の全て
    pub fn moves(&self, next: Entity) -> Vec<Move> {
        let ps = self.puttables(next);
        if ps.is_empty() {
            vec![Move::Pass]
        } else {
            ps.iter().map(|&(i, j)| Move::Put(next, (i, j))).collect()
        }
    }

    /// NOTE: mv should be already validated.
    pub fn play_mut(&mut self, mv: Move) {
        match mv {
            Move::Pass => {
                self.next = -self.next;
            }
            Move::Put(_, (i, j)) => {
                for &(dx, dy) in Game::DEGS.iter() {
                    if let Some(line) = self.reversal_line(Move::Put(self.next, (i, j)), dx, dy) {
                        for &(u, v) in line.iter() {
                            self.data[u][v] = self.next;
                        }
                    }
                }
                self.data[i][j] = self.next;
                self.next = -self.next;
            }
        }
    }

    /// NOTE: mv should be validated
    pub fn play(&self, mv: Move) -> Game {
        let mut g = self.clone();
        g.play_mut(mv);
        g
    }

    pub fn is_valid_move(&self, mv: Move) -> bool {
        let ps = self.puttables(self.next);
        match mv {
            Move::Pass => ps.is_empty(),
            Move::Put(next, (i, j)) => self.next == next && ps.iter().any(|&pos| pos == (i, j)),
        }
    }

    /// どちらも打てない盤面なら終了
    pub fn is_finish(&self) -> bool {
        self.puttables(O).is_empty() && self.puttables(X).is_empty()
    }

    /// (O, X) の数
    pub fn count(&self) -> (usize, usize) {
        let mut o = 0;
        let mut x = 0;
        for line in self.data.iter() {
            for c in line.iter() {
                match c {
                    O => o += 1,
                    X => x += 1,
                    _ => {}
                }
            }
        }
        (o, x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reversal_line() {
        let game = Game {
            next: O,
            height: 3,
            width: 4,
            data: vec![
                vec![Empty, Empty, X, O],
                vec![Empty, Empty, X, Empty],
                vec![Empty, X, X, O],
            ],
            // ..xo
            // ..x.
            // .xxo
        };
        assert_eq!(game.reversal_line(Move::Put(O, (0, 0)), 0, 1), None);
        assert_eq!(
            game.reversal_line(Move::Put(O, (0, 1)), 0, 1),
            Some(vec![(0, 2)])
        );
        assert_eq!(
            game.reversal_line(Move::Put(O, (0, 1)), 1, 1),
            Some(vec![(1, 2)])
        );
        assert_eq!(game.reversal_line(Move::Put(O, (0, 2)), 0, 1), None);
        assert_eq!(game.reversal_line(Move::Put(O, (1, 1)), 1, 1), None);
        assert_eq!(game.reversal_line(Move::Put(O, (1, 1)), 1, -1), None);
        assert_eq!(game.reversal_line(Move::Put(O, (1, 3)), 0, 1), None);
        assert_eq!(game.reversal_line(Move::Put(O, (1, 3)), -1, 0), None);
        assert_eq!(game.reversal_line(Move::Put(O, (1, 3)), 1, 0), None);
        assert_eq!(
            game.reversal_line(Move::Put(O, (2, 0)), 0, 1),
            Some(vec![(2, 1), (2, 2)])
        );
    }

    #[test]
    fn test_moves() {
        let game = Game {
            next: O,
            height: 3,
            width: 4,
            data: vec![
                vec![Empty, Empty, X, O],
                vec![Empty, Empty, X, Empty],
                vec![Empty, X, X, O],
            ],
            // ..xo
            // ..x.
            // .xxo
        };
        assert_eq!(game.moves(O), vec![(0, 1), (2, 0),]);
        assert_eq!(game.moves(X), vec![]);
    }

    #[test]
    fn test_play() {
        let mut game = Game {
            next: O,
            height: 3,
            width: 4,
            data: vec![
                vec![Empty, Empty, X, O],
                vec![Empty, Empty, X, Empty],
                vec![Empty, X, X, O],
            ],
            // ..xo
            // ..x.
            // .xxo
        };
        let result = game.play_mut(Move::Put(O, (0, 1)));
        assert_eq!(result, Ok(()));
        assert_eq!(
            game.data,
            vec![
                vec![Empty, O, O, O],
                vec![Empty, Empty, O, Empty],
                vec![Empty, X, X, O],
            ]
        )
    }
}
