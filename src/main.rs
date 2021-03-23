use shakmaty::{Chess, Position, Setup, Color, Piece, Role, Move};
use shakmaty::fen::{board_fen, epd};
use std::cmp::max;
use smallvec::{smallvec, SmallVec};


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

    white_score - black_score
}

fn negamax(game :&Chess, depth :usize) -> SmallVec<[(Option<Move>, i64); MAX_DEPTH]> {
    if depth == 0 {
        // println!("{}", game.board());
        return smallvec![(None, evaluate(game))];
    }

    let mut value = i64::MIN;
    let mut stack = smallvec![];

    // generate all the legal moves
    for mv in game.legal_moves() {
        let mut new_game = game.clone();
        new_game.play_unchecked(&mv);

        let new_stack = negamax(&new_game, depth - 1);
        let new_value = new_stack.last().unwrap().1 * -1;

        if new_value > value {
            stack = new_stack;
            stack.push( (Some(mv.clone()), new_value) );
            value = new_value;
        }
    }

    // println!("{}: {:?}", depth, stack);

    stack
}

fn main() {
    // create a new board
    let mut game = Chess::default();

    let res = negamax(&game, 4);

    println!("{:?}", res);
}
