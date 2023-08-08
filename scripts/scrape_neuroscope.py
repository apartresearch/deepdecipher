from os import path
import sys

from deepdecipher import Database, ServiceProvider, ModelMetadata

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
if model is None:
    print("Adding neuroscope model.")
    metadata = ModelMetadata.from_neuroscope(model_name)
    model = database.add_model(metadata)
model.scrape_neuroscope_model()

service = database.service("neuroscope")
if service is None:
    print("Adding neuroscope service.")
    service = database.add_service("neuroscope", ServiceProvider.neuroscope())
