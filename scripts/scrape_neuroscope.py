from os import path
import sys

from neuronav import Database

if len(sys.argv) < 2:
    raise RuntimeError("Please specify a database file as the first argument.")

database_path = sys.argv[1]

if path.isfile(database_path):
    database = Database.open(sys.argv[1])
else:
    database = Database.initialize(sys.argv[1])

model = database.model("solu-1l")
if model is not None:
    model.delete()
database.scrape_neuroscope_model("solu-1l")
# database.scrape_neuroscope_model("solu-6l")
