use rand::prelude::*;

use crate::game::{Entity, Game, Move};
use crate::util::mdist;

pub struct Solver {
    verbose: bool,
    num_try: usize,
}

impl Solver {
    pub fn new(verbose: bool, num_try: usize) -> Self {
        Self { verbose, num_try }
    }

    // 場所の良さ
    fn cell_weight(game: &Game, i: usize, j: usize) -> i32 {
        let ds = vec![
            mdist((i, j), (0, 0)),
            mdist((i, j), (0, game.width - 1)),
            mdist((i, j), (game.height - 1, 0)),
            mdist((i, j), (game.height - 1, game.width - 1)),
        ];
        let d = *ds.iter().min().unwrap();
        if d == 0 {
            5
        } else if d == 1 {
            1
        } else {
            2
        }
    }

    pub fn play_random_mut(&self, game: &mut Game) {
        let next = game.next;
        let prev = -game.next;
        let mvs = game.moves(next);
        if mvs.len() == 0 {
            // pass
            game.next = prev;
        } else {
            let mut rng = thread_rng();
            let mv = *mvs
                .choose_weighted(&mut rng, |&Move(_, (i, j))| {
                    Solver::cell_weight(&game, i, j)
                })
                .unwrap();
            let _ = game.play_mut(&mv);
        }
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
    pub fn estimate_prob(&self, game: &Game) -> f64 {
        let mut win = 0;
        for _ in 0..self.num_try {
            if game.next == self.playroll_random(&game) {
                win += 1;
            }
        }
        win as f64 / self.num_try as f64
    }

    pub fn run(&self, game: &Game) -> Option<Game> {
        if game.is_finish() {
            return None;
        }
        let mut maxp = -1.0;
        let mut goodgame = None;
        for &mv in game.moves(game.next).iter() {
            let g = game.play(&mv).ok().unwrap();
            let mut minp = 2.0;
            for &mv in g.moves(g.next).iter() {
                let h = g.play(&mv).ok().unwrap();
                let prob = self.estimate_prob(&h);
                if minp > prob {
                    minp = prob;
                }
            }
            if self.verbose {
                println!("---");
                println!("Move: {:?}", &mv);
                g.write();
                println!("Prob: {:.3}", minp);
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
