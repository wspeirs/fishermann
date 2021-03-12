import tensorflow as tf
import numpy as np

x_data = []
y_data = []

# read in the data file
with open('../data/rand_gen_fen.txt', 'r') as fp:
    for line in fp.readlines():
        score, values = line.split(':')

