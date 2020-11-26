extern crate structopt;
use structopt::StructOpt;

mod game;
use game::{Entity, Game, Move};
mod solver;
use solver::Solver;
mod util;

#[derive(Debug, StructOpt)]
enum SubCommand {
    /// AI Solver
    Solve {
        #[structopt(short, long)]
        verbose: bool,
        #[structopt(long, default_value = "50")]
        /// number of random simulations
        num_try: usize,
        /// 'o' or 'x'
        next: char,
        #[structopt(long, default_value = "10")]
        try_smooth: usize,
        #[structopt(long, default_value = "1.0")]
        coeff_montecarlo: f64,
        #[structopt(long, default_value = "0.4")]
        coeff_position: f64,
        #[structopt(long, default_value = "1.9")]
        coeff_moves: f64,
    },
    /// Simulate Game Move
    Move {
        /// 'o' or 'x'
        next: char,
        /// row number (0-start)
        x: usize,
        /// column number (0-start)
        y: usize,
    },
}

fn main() {
    let com = SubCommand::from_args();
    match com {
        SubCommand::Solve {
            verbose,
            next,
            num_try,
            try_smooth,
            coeff_montecarlo,
            coeff_position,
            coeff_moves,
        } => {
            let game = Game::read(Entity::from_char(next));
            let solver = Solver::new(verbose, num_try, try_smooth, coeff_montecarlo, coeff_position, coeff_moves);
            if game.is_finish() {
                let (o, x) = game.count();
                if o > x {
                    println!("Game Over; O Win");
                } else if o < x {
                    println!("Game Over; X Win");
                } else {
                    println!("Game Over; Draw");
                }
            } else if let Some(goodgame) = solver.run(&game) {
                goodgame.write()
            } else {
                println!("Pass");
            }
        }
        SubCommand::Move { next, x, y } => {
            let next = Entity::from_char(next);
            let mut game = Game::read(next);
            let mv = Move::Put(next, (x, y));
            if game.is_finish() {
                let (o, x) = game.count();
                if o > x {
                    println!("Game Over; O Win");
                } else if o < x {
                    println!("Game Over; X Win");
                } else {
                    println!("Game Over; Draw");
                }
            } else if game.is_valid_move(mv) {
                game.play_mut(mv);
                game.write();
            } else {
                println!("Invalid Move");
            }
        }
    }
}
