from os import path
import sys

import deepdecipher
from deepdecipher import Database, ServiceProvider, ModelMetadata

if len(sys.argv) < 3:
    raise RuntimeError(
        "Please specify a database file as the first argument and model name as the second argument."
    )

database_path = sys.argv[1]
model_name = sys.argv[2]

deepdecipher.log_init("logs")

if path.isfile(database_path):
    database = Database.open(database_path)
else:
    database = Database.initialize(database_path)

model = database.model(model_name)
if model is None:
    model_metadata = ModelMetadata(
        "gpt2-small", 12, 3072, "gelu", 84934656, "Open Web Text"
    )
    model = database.add_model(model_metadata)
print("Adding neuron_explainer data for model...")
model.add_neuron_explainer_small()

service = database.service("neuron_explainer")
if service is None:
    print("Adding neuron_explainer service.")
    service = database.add_service(
        "neuron_explainer", ServiceProvider.neuron_explainer()
    )
