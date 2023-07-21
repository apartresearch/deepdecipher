from os import path
import sys

from neuronav import Database

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
if model is not None:
    model.delete()
database.scrape_neuroscope_model(model_name)
