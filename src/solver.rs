use rand::prelude::*;

use crate::game::{Entity, Game, Move};
use crate::util::{cdist, mdist, HyperFloat};

pub struct Solver {
    verbose: bool,
    num_try: usize,
    try_smooth: usize,
    coeff_montecarlo:f64,
    coeff_position:f64,
    coeff_moves:f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pos {
    Corner, // 角
    X,      // 角の斜め隣
    C,      // 角の横
    Edge,
    Other,
}
use Pos::*;

impl Pos {
    fn from(i: usize, j: usize, height: usize, width: usize) -> Self {
        let md = *vec![
            mdist((i, j), (0, 0)),
            mdist((i, j), (0, width - 1)),
            mdist((i, j), (height - 1, 0)),
            mdist((i, j), (height - 1, width - 1)),
        ]
        .iter()
        .min()
        .unwrap();
        let cd = *vec![
            cdist((i, j), (0, 0)),
            cdist((i, j), (0, width - 1)),
            cdist((i, j), (height - 1, 0)),
            cdist((i, j), (height - 1, width - 1)),
        ]
        .iter()
        .min()
        .unwrap();
        if md == 0 {
            Corner
        } else if md == 2 && cd == 1 {
            X
        } else if md == 1 && cd == 1 {
            C
        } else if i == 0 || i == height - 1 || j == 0 || j == width - 1 {
            Edge
        } else {
            Other
        }
    }
}

impl Solver {
    pub fn new(verbose: bool, num_try: usize, try_smooth: usize, coeff_montecarlo: f64, coeff_position: f64, coeff_moves: f64) -> Self {
        Self { verbose, num_try, try_smooth, coeff_montecarlo, coeff_position, coeff_moves }
    }

    // 場所を選ぶ確率
    fn cell_prob(game: &Game, i: usize, j: usize) -> i32 {
        match Pos::from(i, j, game.height, game.width) {
            Corner => 3,
            X => 1,
            C => 1,
            Edge => 2,
            Other => 2,
        }
    }

    // 場所の良さ
    fn cell_goodness(game: &Game, i: usize, j: usize) -> f64 {
        match Pos::from(i, j, game.height, game.width) {
            Corner => 6.0,
            X => -2.0,
            C => -1.0,
            Edge => 0.1,
            Other => 0.05,
        }
    }

    // その手を選ぶ確率
    fn move_prob(game: &Game, mv: Move) -> i32 {
        match mv {
            Move::Pass => 1,
            Move::Put(_, (i, j)) => Solver::cell_prob(&game, i, j),
        }
    }

    /// ランダムに一手選んで打つ
    pub fn play_random_mut(&self, game: &mut Game) {
        let next = game.next;
        let mvs = game.moves(next);
        let mut rng = thread_rng();
        let mv = *mvs
            .choose_weighted(&mut rng, |&mv| Solver::move_prob(&game, mv))
            .unwrap();
        game.play_mut(mv);
    }

    /// ランダムにプレイして決着をつける
    pub fn playroll_random(&self, game: &Game) -> Entity {
        let mut game = game.clone();
        loop {
            if game.is_finish() {
                break;
            }
            self.play_random_mut(&mut game);
        }
        let (o, x) = game.count();
        if o > x {
            Entity::O
        } else if o < x {
            Entity::X
        } else {
            Entity::Empty
        }
    }

    /// 勝つ確率の推定
    pub fn estimate_prob(&self, game: &Game, debug: bool) -> f64 {
        // モンテカルロで勝つ割合
        let mut win = 0;
        for _ in 0..self.num_try {
            if game.next == self.playroll_random(&game) {
                win += 1;
            }
        }
        let win_ratio = ((win + self.try_smooth) as f64 / (self.num_try + 2 * self.try_smooth) as f64).ln();

        // 場所の良さ
        let mut goodness_self = 0_f64;
        let mut goodness_enemy = 0_f64;
        for i in 0..game.height {
            for j in 0..game.width {
                if game.data[i][j] == game.next {
                    goodness_self += Solver::cell_goodness(&game, i, j);
                } else if game.data[i][j] == -game.next {
                    goodness_enemy += Solver::cell_goodness(&game, i, j);
                }
            }
        }
        let goodness_position = goodness_self - goodness_enemy;

        // 打てる場所の数
        let num_moves_self = game.moves(game.next).len();
        let num_moves_enemy = game.moves(-game.next).len();
        let moves_ratio = (num_moves_self as f64).ln() - (num_moves_enemy as f64).ln();

        let value =
            win_ratio * self.coeff_montecarlo
            + goodness_position * self.coeff_position
            + moves_ratio * self.coeff_moves;
        if debug {
            println!("winning: {} / {}", win, self.num_try);
            println!("good_pos: {} / {}", goodness_self, goodness_enemy);
            println!("num_moves: {} / {}", num_moves_self, num_moves_enemy);
            println!(
                "win: {:?}, good_pos: {:?}, moves: {:?} => value: {}",
                win_ratio, goodness_position, moves_ratio, value
            );
        }
        value
    }

    pub fn run(&self, game: &Game) -> Option<Game> {
        if game.is_finish() {
            return None;
        }
        let mut maxp = HyperFloat::MinInf;
        let mut goodgame = None;
        for &mv in game.moves(game.next).iter() {
            let g = game.play(mv);
            let mut h_min = g.clone();
            let mut minp = HyperFloat::Inf;
            for &mv in g.moves(g.next).iter() {
                let h = g.play(mv);
                let p = HyperFloat::Real(self.estimate_prob(&h, false));
                if minp > p {
                    minp = p;
                    h_min = h.clone();
                }
            }
            if self.verbose {
                println!("---");
                println!("Move: {:?}", &mv);
                g.write();
                println!("=>");
                h_min.write();
                self.estimate_prob(&h_min, true);
                println!("Prob: {:?}", minp);
                println!("---");
            }
            if maxp < minp {
                maxp = minp;
                goodgame = Some(g);
            }
        }
        goodgame
    }
}
