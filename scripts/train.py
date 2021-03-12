import tensorflow as tf
import numpy as np

x_data = []
y_data = []

# read in the data file
with open('../data/rand_gen.values', 'r') as fp:
    for line in fp.readlines():
        score, values = line.strip().split(':')

        y_data.append(float(score))  # convert to a float
        x_data.append([float(x) for x in values.split(' ')])  # convert to an array of floats


# convert to numpy array
y_data = np.array(y_data)
x_data = np.array(x_data)

# this _should_ always be 65, but get it programmatically
x_data_width = len(x_data[0])

# create a model by defining the layers
model = tf.keras.models.Sequential()

# start with an input layer with a shape the same size as a single array, by as many arrays as we have (left blank)
model.add(tf.keras.layers.InputLayer(input_shape=(x_data_width,)))

# add a dense layer after our input that's twice the size
model.add(tf.keras.layers.Dense(x_data_width*2, activation='relu'))

# add another dense layer that's the same size
model.add(tf.keras.layers.Dense(x_data_width, activation='relu'))

# another dense layer that's floor of half the size
model.add(tf.keras.layers.Dense(int(x_data_width/2), activation='relu'))

# finally a single-output layer
model.add(tf.keras.layers.Dense(1))

# create a loss function
loss_fn = tf.keras.losses.BinaryCrossentropy()

model.compile(optimizer='adam', loss=loss_fn, metrics=['accuracy'])

model.fit(x_data, y_data, epochs=5)

