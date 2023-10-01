import sys

import sqlite3 as sql

if len(sys.argv) > 1:
    database_path = sys.argv[1]
else:
    raise "No database path provided."

cursor = sql.connect(database_path).cursor()
cursor.execute("ALTER TABLE data_object RENAME TO data_type")
cursor.execute("ALTER TABLE model_data_object RENAME TO model_data_type")
cursor.execute(
    "ALTER TABLE model_data_type RENAME COLUMN data_object_id TO data_type_id"
)
cursor.execute("ALTER TABLE model_data RENAME COLUMN data_object_id TO data_type_id")
cursor.execute("ALTER TABLE layer_data RENAME COLUMN data_object_id TO data_type_id")
cursor.execute("ALTER TABLE neuron_data RENAME COLUMN data_object_id TO data_type_id")
