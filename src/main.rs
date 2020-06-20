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
        #[structopt(long, default_value = "200")]
        /// number of random simulations
        num_try: usize,
        /// 'o' or 'x'
        next: char,
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
        } => {
            let game = Game::read(Entity::from_char(next));
            let solver = Solver::new(verbose, num_try);
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
            let mv = Move(next, (x, y));
            if game.is_finish() {
                let (o, x) = game.count();
                if o > x {
                    println!("Game Over; O Win");
                } else if o < x {
                    println!("Game Over; X Win");
                } else {
                    println!("Game Over; Draw");
                }
            } else if game.is_valid_move(&mv) {
                if let Ok(_) = game.play_mut(&mv) {
                    game.write();
                } else {
                    println!("Invalid Move");
                }
            } else {
                println!("Invalid Move");
            }
        }
    }
}
