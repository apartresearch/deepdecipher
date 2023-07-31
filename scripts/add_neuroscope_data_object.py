from os import path
import sys

from deepdecipher import Database, ServiceProvider

if len(sys.argv) < 3:
    raise RuntimeError(
        "Please specify a database file as the first argument and model name as the second argument."
    )

database_path = sys.argv[1]
model_name = sys.argv[2]

if path.isfile(database_path):
    database = Database.open(database_path)
else:
    database = Database.initialize(database_path)

model = database.model(model_name)
data_object = database.data_object("neuroscope")
model.add_data_object(data_object)
