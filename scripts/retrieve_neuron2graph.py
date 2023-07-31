from os import path
import sys

from deepdecipher import Database, ModelMetadata, ServiceProvider

if len(sys.argv) < 3:
    raise RuntimeError(
        "Please specify a database file as the first argument and data path as the second argument."
    )

database_path = sys.argv[1]
data_path = sys.argv[2]

if path.isfile(database_path):
    database = Database.open(database_path)
else:
    database = Database.initialize(database_path)

model = database.model("solu-6l")
if model is None:
    model_metadata = ModelMetadata("solu-6l", 6, 3072, "solu", 42467328, "The Pile")
    model = database.add_model(model_metadata)
assert model is not None

data_object = database.data_object("neuron2graph")
if data_object is not None and model.has_data_object(data_object):
    print("Deleting existing neuron2graph data from model.")
    model.delete_data_object(data_object)


print("Adding neuron2graph neuron graphs to model.")
model.add_neuron2graph_graphs(data_path)

service = database.service("neuron2graph")
if service is None:
    print("Adding neuron2graph service.")
    service = database.add_service("neuron2graph", ServiceProvider.neuron2graph())
