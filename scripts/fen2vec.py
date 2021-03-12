import chess
import sys

PIECE2VALUE = {
    None: 0,
    'None': 0,
    'p': -100,
    'n': -400,
    'b': -400,
    'r': -600,
    'q': -1200,
    'k': -10000,
    'P': 100,
    'N': 400,
    'B': 400,
    'R': 600,
    'Q': 1200,
    'K': 10000
}

SQUARES = [chess.Square(s) for s in range(64)]

# go through the file passed on the cmd line
if len(sys.argv) < 2:
    print("Must pass file to parse on the command line")
    exit(-1)

file = sys.argv[1]

with open(file, 'r') as fp:
    for line in fp.readlines():
        score, fen = line.split(":")

        # print the score
        print("{}:".format(score), end='')

        board = chess.Board(fen.strip())

        # print a value for who's turn it is
        if board.turn == chess.WHITE:
            print('10000 ', end='')
        else:
            print('-10000 ', end='')

        for square in SQUARES:
            piece = board.piece_at(square)
            print("{} ".format(PIECE2VALUE[str(piece)]), end='')

        print()
