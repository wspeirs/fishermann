import tensorflow as tf
import numpy as np

x_data = []
y_data = []

# read in the data file
with open('../data/rand_gen_1m.values', 'r') as fp:
    for line in fp.readlines():
        score, values = line.strip().split(':')

        y_data.append(float(score))  # convert to a float
        x_data.append([float(x) for x in values.split(' ')])  # convert to an array of floats

print("DONE READING DATA")

# break up into training and testing data
train_len = int(len(y_data) * 0.90)  # use 90% of the data for training
y_train_data = np.array(y_data[:train_len])
x_train_data = np.array(x_data[:train_len])
y_test_data = np.array(y_data[train_len:])
x_test_data = np.array(x_data[train_len:])

# this _should_ always be 65, but get it programmatically
x_data_width = len(x_data[0])

# create a model by defining the layers
model = tf.keras.models.Sequential()

# start with an input layer with a shape the same size as a single array, by as many arrays as we have (left blank)
model.add(tf.keras.layers.InputLayer(input_shape=(x_data_width,)))

# go through and add dense layers starting at x_data_width^2 until we get to 1
cur_nodes = x_data_width**2
while True:
    # add a dense layer after our input that's twice the size
    model.add(tf.keras.layers.Dense(cur_nodes, activation='relu'))

    cur_nodes = int(cur_nodes / 4)

    if cur_nodes <= 1:
        model.add(tf.keras.layers.Dense(cur_nodes, activation='relu'))
        break

# create a loss function
loss_fn = tf.keras.losses.LogCosh()

# compile the model with the loss function
model.compile(optimizer=tf.keras.optimizers.SGD(0.8), loss=loss_fn, metrics=['accuracy'])

# fit our training data to the model
model.fit(x_train_data, y_train_data, validation_data=(x_test_data, y_test_data), shuffle=True, epochs=5)

# evaluate how well we did with test data
model.evaluate(x_test_data, y_test_data, verbose=1)


