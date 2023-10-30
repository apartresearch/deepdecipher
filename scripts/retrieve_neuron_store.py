from os import path
import sys

from deepdecipher import Database, ModelMetadata, ServiceProvider

if len(sys.argv) < 4:
    raise RuntimeError(
        "Please specify a database file as the first argument, the neuron store path as the second argument, and the model name as the third argument."
    )

database_path = sys.argv[1]
neuron_store_path = sys.argv[2]
model_name = sys.argv[3]

if path.isfile(database_path):
    database = Database.open(sys.argv[1])
else:
    database = Database.initialize(sys.argv[1])

model = database.model(model_name)
if model is None:
    model_metadata = ModelMetadata.from_neuroscope(model_name)
    model = database.add_model(model_metadata)
assert model is not None

data_type = database.data_type("neuron_store")
if data_type is not None:
    print("Deleting existing neuron store data for model.")
    model.delete_data_type(data_type)

print("Adding neuron store data for model.")
model.add_neuron_store(neuron_store_path, 0.4)

service = database.service("neuron2graph-search")
if service is None:
    print("Adding neuron2graph-search service.")
    service = database.add_service(
        "neuron2graph-search", ServiceProvider.neuron2graph_search()
    )
