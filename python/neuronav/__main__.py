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

database.start_server()
