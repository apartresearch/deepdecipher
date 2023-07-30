from os import path
import sys

from deepdecipher import Database, ModelMetadata, ServiceProvider

if len(sys.argv) < 2:
    raise RuntimeError("Please specify a database file as the first argument.")

database_path = sys.argv[1]

if path.isfile(database_path):
    database = Database.open(sys.argv[1])
else:
    database = Database.initialize(sys.argv[1])

model = database.model("solu-6l")
if model is None:
    model_metadata = ModelMetadata("solu-6l", 6, 3072, "solu", 42467328, "The Pile")
    model = database.add_model(model_metadata)
assert model is not None

data_object = database.data_object("neuron_store")
if data_object is not None:
    print("Deleting existing neuron store data for model.")
    model.delete_data_object(data_object)

print("Adding neuron store data for model.")
model.add_neuron_store("data\\solu-6l\\neuron2graph-search\\neuron_store.json", 0.4)

service = database.service("neuron2graph-search")
if service is None:
    print("Adding neuron2graph-search service.")
    service = database.add_service(
        "neuron2graph-search", ServiceProvider.neuron2graph_search()
    )
