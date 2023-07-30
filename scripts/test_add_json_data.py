from os import path
import sys
import json

from deepdecipher import Database, DataType, Index, ServiceProvider

if len(sys.argv) < 2:
    raise RuntimeError("Please specify a database file as the first argument.")

database_path = sys.argv[1]

if path.isfile(database_path):
    database = Database.open(database_path)
else:
    database = Database.initialize(database_path)

model = database.model("solu-1l")
data_object = database.data_object("json-test")
if data_object is not None:
    print("Deleting existing json-test data from model...")
    data_object.delete()
data_object = database.add_data_object("json-test", DataType.json())

if not model.has_data_object(data_object):
    model.add_data_object(data_object)

model_layer_size = model.metadata().layer_size
for neuron_index in range(model_layer_size):
    print(f"Adding json-test data to model {neuron_index}/{model_layer_size}", end="\r")
    json_data = {"index_square": neuron_index**2, "index_cube": neuron_index**3}
    model.add_json_data(
        data_object, Index.neuron(0, neuron_index), json.dumps(json_data)
    )
print("Added json-test data to model.                           ")

service = database.service("power-service")
if service is None:
    print("Adding power-service service.")
    service = database.add_service("power-service", ServiceProvider.json(data_object))
