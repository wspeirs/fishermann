mod uci;

use shakmaty::{Chess, Position, Setup, Color, Piece, Role, Move, CastlingMode, Square, Rank, File};
use shakmaty::fen::{board_fen, epd, Fen};

use smallvec::{smallvec, SmallVec};
use crate::uci::{Uci, Analysis};
use std::process::Command;
use std::collections::HashMap;
use vampirc_uci::UciMove;
use vampirc_uci::uci::UciTimeControl::MoveTime;
use std::fmt::Display;


const MAX_LINES :usize = 3;
const DEPTH :u8 = 10;


fn uci2move(uci_mv :UciMove, game :&Chess) -> Move {
    let from_square = Square::from_coords(File::from_char(uci_mv.from.file).unwrap(), Rank::new((uci_mv.from.rank-1) as u32));
    let to_square = Square::from_coords(File::from_char(uci_mv.to.file).unwrap(), Rank::new((uci_mv.to.rank-1) as u32));

    // println!("UCI: {} -> {} MOVE: {} -> {}\t{}", uci_mv.from, uci_mv.to, from_square, to_square, epd(game));

    Move::Normal {
        role: game.board().piece_at(from_square).unwrap().role,
        from: from_square,
        capture: game.board().piece_at(to_square).map(|p| p.role),
        to: to_square,
        promotion: uci_mv.promotion.map(|p| Role::from_char(p.as_char().unwrap()).unwrap())
    }
}

fn moves2string<I>(moves :I) -> String
    where I: Iterator,
    <I as Iterator>::Item: Display + Clone
{
    let ret = moves.map(|mv| {
        mv.to_string()
    }).collect::<Vec<_>>();

    ret.join(", ")
}


fn main() {
    let mut board = Chess::default();
    let mut stockfish_cmd = Command::new("/usr/local/bin/stockfish");
    let mut engine = Uci::start_engine(&mut stockfish_cmd);

    // we want to track the top 5 moves
    engine.set_option("MultiPV", MAX_LINES.to_string().as_str());

    // set to a large hash table
    engine.set_option("Hash", "512");

    // make the most common move
    // board.play_unchecked(&Move::Normal {
    //     role: Role::Pawn,
    //     from: Square::C2,
    //     capture: None,
    //     to: Square::C4,
    //     promotion: None
    // });

    // let mut move_count :HashMap<Move, i32> = HashMap::new();
    let mut resp_scores = Vec::new();

    let mut moves = Vec::new();

    // go through all the possible moves
    let legal_moves = board.legal_moves().into_iter().collect::<Vec<_>>();
    for white_first_move in &legal_moves {
        let mut first_mv_board = board.clone();

        // println!("WHITE MV: {}", &white_first_move);
        first_mv_board.play_unchecked(&white_first_move);
        moves.push(white_first_move.clone());

        // we make black's move based upon the "side" of the board white played
        let black_first_mv = if white_first_move.to().file() < File::E {
            Move::Normal {
                role: Role::Pawn,
                from: Square::F7,
                capture: None,
                to: Square::F5,
                promotion: None
            }
        } else {
            Move::Normal {
                role: Role::Pawn,
                from: Square::C7,
                capture: None,
                to: Square::C5,
                promotion: None
            }
        };

        // println!("BLACK MV: {}", &black_first_mv);
        first_mv_board.play_unchecked(&black_first_mv);
        moves.push(black_first_mv.clone());

        // again go through all the moves for white
        for white_second_mv in first_mv_board.legal_moves() {
            let mut second_mv_board = board.clone();

            // println!("WHITE MV: {}", &white_second_mv);
            second_mv_board.play_unchecked(&white_second_mv);
            moves.push(white_second_mv.clone());

            let black_second_mv = if second_mv_board.board().piece_at(Square::F5).is_some() {
                Move::Normal {
                    role: Role::Knight,
                    from: Square::G8,
                    capture: None,
                    to: Square::F6,
                    promotion: None
                }
            } else {
                Move::Normal {
                    role: Role::Knight,
                    from: Square::B8,
                    capture: None,
                    to: Square::C6,
                    promotion: None
                }
            };

            // println!("BLACK MV: {}", &black_second_mv);
            second_mv_board.play_unchecked(&black_second_mv);
            moves.push(black_second_mv.clone());

            // have the engine analyze the move
            let analysis = engine.analyze(epd(&first_mv_board), DEPTH);

            // only want to look at moves to the full depth
            let mut responses = analysis.iter()
                .filter(|a| {
                    if let Analysis::PossibleMove(pmv) = a {
                        pmv.depth == DEPTH // && pmv.score > 0
                    } else {
                        false
                    }
                })
                .map(|a| {
                    let pmv = a.as_possible_move();

                    (pmv.score, uci2move(pmv.moves[0], &first_mv_board))
                })
                .collect::<SmallVec<[(i32, Move); 5]>>();

            // record all the responses for a move
            // mv_resp.push( (white_first_move.clone(), responses.clone()) );

            for (score, res_mv) in &responses {
                resp_scores.push(*score);

                moves.push(res_mv.clone());
                println!("\t{}: {}", score, moves2string(moves.iter()));
                moves.pop();
            }

            moves.pop();
            moves.pop();

            // record a count for each move
            // for (score, res_mv) in responses.into_iter() {
            //     *move_count.entry(res_mv).or_default() += score;
            // }
        }

        moves.clear();
    }

    // go through all the response scores
    let white_adv_count = resp_scores.iter().filter(|s| **s > 0).count();
    let black_adv_count = resp_scores.iter().filter(|s| **s < 0).count();

    println!("WHITE ADV: {} + BLACK ADV: {} = {}", white_adv_count, black_adv_count, resp_scores.len());

    let avg_white_adv = resp_scores.iter().filter(|s| **s > 0).sum::<i32>() as f64 / resp_scores.len() as f64;
    let avg_black_adv = resp_scores.iter().filter(|s| **s < 0).sum::<i32>() as f64 / resp_scores.len() as f64;

    println!("AVG WHITE ADV: {}\tAVG BLACK ADV: {}", avg_white_adv, avg_black_adv);

/*

    // sort by count
    let mut mv_count_vec = move_count.into_iter().map(|(mv, count)| (count, mv)).collect::<Vec<_>>();
    mv_count_vec.sort_unstable_by_key(|(c, m)| *c);

    loop {
        // get the most common move
        let (score, most_common_move) = mv_count_vec.pop().unwrap();

        println!("COMMON MOVE: {} - {}", score, most_common_move);

        // go through and "reset" all response to _only_ the most common if it exists
        for mvr in mv_resp.iter_mut() {
            if let Some(i) = mvr.1.iter().position(|(s, m)| *m == most_common_move) {
                mvr.1 = smallvec![mvr.1[i].clone()];
            }
        }

        // if we've set everything to a single move, then break out of the loop
        if mv_resp.iter().all(|(mv, resp)| resp.len() == 1) {
            break
        }
    }

    // go through and print the moves and responses
    // mv_resp.sort_unstable_by_key(|(mv, resp)| resp[0]);

    for (mv, resp) in mv_resp.iter() {
        println!("{}: {} - {}", mv, resp[0].0, resp[0].1);
    }

 */
}
