use std::cmp::max;
use std::time::Instant;

use shakmaty::{Chess, Position, Setup, Color, Piece, Role, Move, CastlingMode};
use shakmaty::fen::{board_fen, epd, Fen};

use smallvec::{smallvec, SmallVec};

use rayon::prelude::*;


const MAX_DEPTH :usize = 96;

const KING_VALUES :[i64; 64] =
    [ -30,-40,-40,-50,-50,-40,-40,-30,
      -30,-40,-40,-50,-50,-40,-40,-30,
      -30,-40,-40,-50,-50,-40,-40,-30,
      -30,-40,-40,-50,-50,-40,-40,-30,
      -20,-30,-30,-40,-40,-30,-30,-20,
      -10,-20,-20,-20,-20,-20,-20,-10,
       20, 20,  1,  1,  1,  1, 20, 20,
       20, 30, 10,  1,  1, 10, 30, 20 ];

const QUEEN_VALUES :[i64; 64] =
    [ -20,-10,-10, -5, -5,-10,-10,-20,
      -10,  1,  1,  1,  1,  1,  1,-10,
      -10,  1,  5,  5,  5,  5,  1,-10,
       -5,  1,  5,  5,  5,  5,  1, -5,
        1,  1,  5,  5,  5,  5,  1, -5,
      -10,  5,  5,  5,  5,  5,  1,-10,
      -10,  1,  5,  1,  1,  1,  1,-10,
      -20,-10,-10, -5, -5,-10,-10,-20 ];

const ROOK_VALUES :[i64; 64] =
    [  1,  1,  1,  1,  1,  1,  1,  1,
       5, 10, 10, 10, 10, 10, 10,  5,
      -5,  1,  1,  1,  1,  1,  1, -5,
      -5,  1,  1,  1,  1,  1,  1, -5,
      -5,  1,  1,  1,  1,  1,  1, -5,
      -5,  1,  1,  1,  1,  1,  1, -5,
      -5,  1,  1,  1,  1,  1,  1, -5,
       1,  1,  1,  5,  5,  1,  1,  1 ];

const BISHOP_VALUES :[i64; 64] =
    [ -20,-10,-10,-10,-10,-10,-10,-20,
      -10,  1,  1,  1,  1,  1,  1,-10,
      -10,  1,  5, 10, 10,  5,  1,-10,
      -10,  5,  5, 10, 10,  5,  5,-10,
      -10,  1, 10, 10, 10, 10,  1,-10,
      -10, 10, 10, 10, 10, 10, 10,-10,
      -10,  5,  1,  1,  1,  1,  5,-10,
      -20,-10,-10,-10,-10,-10,-10,-20 ];

const KNIGHT_VALUES :[i64; 64] =
    [ -50,-40,-30,-30,-30,-30,-40,-50,
      -40,-20,  1,  1,  1,  1,-20,-40,
      -30,  1, 10, 15, 15, 10,  1,-30,
      -30,  5, 15, 20, 20, 15,  5,-30,
      -30,  1, 15, 20, 20, 15,  1,-30,
      -30,  5, 10, 15, 15, 10,  5,-30,
      -40,-20,  1,  5,  5,  1,-20,-40,
      -50,-40,-30,-30,-30,-30,-40,-50 ];

const PAWN_VALUES :[i64; 64] =
    [  1,  1,  1,  1,  1,  1,  1,  1,
      50, 50, 50, 50, 50, 50, 50, 50,
      10, 10, 20, 30, 30, 20, 10, 10,
       5,  5, 10, 25, 25, 10,  5,  5,
       1,  1,  1, 20, 20,  1,  1,  1,
       5, -5,-10,  1,  1,-10, -5,  5,
       5, 10, 10,-20,-20, 10, 10,  5,
       1,  1,  1,  1,  1,  1,  1,  1 ];

#[inline]
fn get_value(square :usize, piece :&Piece) -> i64 {
    match piece.role {
        Role::Pawn => PAWN_VALUES[square],
        Role::Knight => KNIGHT_VALUES[square],
        Role::Bishop => BISHOP_VALUES[square],
        Role::Rook => ROOK_VALUES[square],
        Role::Queen => QUEEN_VALUES[square],
        Role::King => KING_VALUES[square]
    }
}

/// Given a game, evaluate the board
/// The evaluation is white_score - black_score
fn evaluate(game :&Chess) -> i64 {
    let board = game.board();

    let mut white_score = 0_i64;
    let mut black_score = 0_i64;

    // go through the pieces on the white squares
    for square in board.by_color(Color::White) {
        white_score += get_value(square as usize, &board.piece_at(square).unwrap())
    }

    // then through the black squares, flipping the square
    for square in board.by_color(Color::Black) {
        black_score += get_value(square.flip_vertical() as usize, &board.piece_at(square).unwrap())
    }

    if game.turn() == Color::White {
        (white_score + 10) - black_score
    } else {
        white_score - (black_score + 10)
    }
}

// fn negamax(game :&Chess, depth :usize) -> SmallVec<[(Option<Move>, i64); MAX_DEPTH]> {
    // get all the moves
    // let mut results = negamax_basic(game, 2);
    //
    // // sort the moves (is this the correct order?!?
    // results.sort_unstable_by_key(|(_op_mv, score)| score);
    //
    // // now go through each move, increasing the depth
    // for d in 3..depth {
    //     let mut value = i64::MIN;
    //     let mut stack = smallvec![];
    //
    //     for mv in results.iter().map(|(mv, _score)| mv) {
    //
    //     }
    // }

    // return negamax_ab(game, depth, &mut i64::MIN, i64::MAX);
// }

fn negamax_ab(game :&Chess, depth :usize, alpha :&mut i64, beta :i64) -> (i64, SmallVec<[Move; MAX_DEPTH]>) {
    if depth == 0 {
        return (evaluate(&game), smallvec![]);
    }

    if game.is_checkmate() {
        return (if game.turn() == Color::White {
            i64::MIN
        } else {
            i64::MAX
        }, smallvec![]);
    }

    let mut value = i64::MIN;
    let mut stack = smallvec![];

    // generate all the legal moves
    for mv in game.legal_moves() {
        let mut new_game = game.clone();
        new_game.play_unchecked(&mv);

        let mut new_alpha = beta.saturating_neg();
        let new_beta = alpha.saturating_neg();

        // make the recursive call
        let (new_value, new_stack) = negamax_ab(&new_game, depth - 1, &mut new_alpha, new_beta);

        if new_value > value {
            stack = new_stack;
            stack.push(mv.clone());
            value = new_value;
        }

        *alpha = max(*alpha, value);

        if *alpha >= beta {
            break
        }
    }

    // we might not have actually gotten _any_ legal moves here
    // so we just return a "very bad" value
    if stack.is_empty() {
        // println!("STACK EMPTY!!!");
        let val = if game.turn() == Color::White {
            i64::MIN
        } else {
            i64::MAX
        };

        (val, smallvec![])
    } else {
        (value, stack)
    }
}

fn negamax_basic(game :&Chess, depth :usize) -> (i64, SmallVec<[Move; MAX_DEPTH]>) {
    if depth == 0 {
        return (evaluate(&game), smallvec![]);
    }

    if game.is_checkmate() {
        return (if game.turn() == Color::White {
            i64::MIN
        } else {
            i64::MAX
        }, smallvec![]);
    }

    let mut value = i64::MIN;
    let mut stack = smallvec![];

    // generate all the legal moves
    for mv in game.legal_moves() {
        let mut new_game = game.clone();
        new_game.play_unchecked(&mv);

        // make the recursive call
        let (new_value, new_stack) = negamax_basic(&new_game, depth - 1);

        // println!("D{} ({}) {}: {}", depth, mv, new_value, moves2string(&new_stack));

        if new_value == value {
            let mut print_stack = new_stack.clone();
            print_stack.push(mv.clone());
            println!("FOUND EQUAL: {} {} == {} {}", value, moves2string(&stack), new_value, moves2string(&print_stack));
        }

        if new_value > value {
            stack = new_stack;
            stack.push(mv.clone());
            value = new_value;
        }
    }

    // we might not have actually gotten _any_ legal moves here
    // so we just return a "very bad" value
    if stack.is_empty() {
        println!("STACK EMPTY!!!");
        let val = if game.turn() == Color::White {
            i64::MIN
        } else {
            i64::MAX
        };

        (val, smallvec![])
    } else {
        (value, stack)
    }
}

fn moves2string(moves:&SmallVec<[Move; MAX_DEPTH]>) -> String {
    let ret = moves.iter().rev().map(|mv| {
        mv.to_string()
    }).collect::<Vec<_>>();

    format!("{} ({})", ret.join(", "), moves.len())
}

fn main() {
    let depth = 20;
    let fen = "8/8/k7/p7/2K5/1P6/8/8 w - - 0 1";
    println!("DEPTH: {} FEN: {}", depth, fen);

    let setup :Fen = fen.parse().expect("Error parsing FEN");

    // let game :Chess = setup.position(CastlingMode::Standard).expect("Error setting up game");
    // let start = Instant::now();
    // let (score, moves) = negamax_basic(&game, depth);
    // println!("{}s:\t{}: {}", start.elapsed().as_secs_f64(), score, moves2string(&moves));

    // let game :Chess = setup.position(CastlingMode::Standard).expect("Error setting up game");
    let game = Chess::default();
    let start = Instant::now();
    let (score, moves) = negamax_ab(&game, depth, &mut i64::MIN, i64::MAX);
    println!("{}s:\t{}: {}", start.elapsed().as_secs_f64(), score, moves2string(&moves));

    // let game = Chess::default();
    // let start = Instant::now();
    // let res = parallel_negamax(&game, depth);
    // println!("{}s: {:?}", start.elapsed().as_secs_f64(), res);

}
