use fastrand;
use shakmaty::{Chess, Position};
use shakmaty::fen::{board_fen, epd};
use std::process::Command;
use crate::uci::{Uci, Analysis};

mod uci;

/*
 * Basic idea/algorithm:
 * 1) Start with a normal board
 * 2) Make a random move
 * 3) Print the board
 * 4) Get the score from Stockfish at a depth of 20
 * 5) Print out the score as determined by Stockfish
 * 6) Continue until the game is won
 * 7) Check how many board we've created... continue until we have thousands
 */

fn main() {
    let mut count :u64 = 0;
    let mut stockfish_cmd = Command::new("/usr/local/bin/stockfish");
    let mut analysis_engine = Uci::start_engine(&mut stockfish_cmd);

    while count < 10_000 {
        // create a new board
        let mut board = Chess::default();

        while !board.is_game_over() {
            let legal_moves = board.legal_moves();

            // randomly pick one
            let rand_move_idx = fastrand::usize(..legal_moves.len());
            let mv = &legal_moves[rand_move_idx];

            // make the move
            board = board.play(mv).expect("Got illegal move");

            let rx = analysis_engine.analyze(epd(&board), 20);
            let mut rx_iter = rx.iter();
            let mut last_analysis = rx_iter.next().unwrap();

            for analysis in rx_iter {
                if let Analysis::BestMove(_) = analysis {
                    if let Analysis::PossibleMove(pmv) = last_analysis {
                        println!("{}: {}", pmv.score, epd(&board));
                        break;
                    } else {
                        panic!("Last is not possible")
                    }
                }

                last_analysis = analysis;
            }

            // bump our count
            count += 1;
        }
    }
}
