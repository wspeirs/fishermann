import chess
import sys

# Stockfish values can be found here: https://github.com/official-stockfish/Stockfish/blob/master/src/types.h#L189
PIECE2VALUE = {
    None: 0,
    'None': 0,
    'p': -126,
    'n': -781,
    'b': -825,
    'r': -1276,
    'q': -2538,
    'k': -32000,
    'P': 126,
    'N': 781,
    'B': 825,
    'R': 1276,
    'Q': 2538,
    'K': 32000
}

SIDE_TO_PLAY = {
    chess.WHITE: 50000,
    chess.BLACK: -50000
}

SQUARES = [chess.Square(s) for s in range(64)]

# go through the file passed on the cmd line
if len(sys.argv) < 2:
    print("Must pass file to parse on the command line")
    exit(-1)

file = sys.argv[1]

seen_fens = set()

with open(file, 'r') as fp:
    for line in fp.readlines():
        score, fen = line.split(":")

        if fen in seen_fens:
            continue

        # add to the list of things seen
        seen_fens.add(fen)

        # print the score
        print("{}:".format(score), end='')

        board = chess.Board(fen.strip())

        # print a value for who's turn it is
        print('{} '.format(SIDE_TO_PLAY[board.turn]), end='')

        for square in SQUARES:
            piece = board.piece_at(square)
            print("{} ".format(PIECE2VALUE[str(piece)]), end='')

        print()
