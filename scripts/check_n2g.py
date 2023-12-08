from os import path
import sys

import sqlite3 as sql

if len(sys.argv) < 2:
    raise RuntimeError("Please specify a database file as the first argument.")

database_path = sys.argv[1]

cursor = sql.connect(database_path).cursor()

n2g_id = cursor.execute(
    "SELECT id FROM data_type WHERE name='neuron2graph'"
).fetchall()[0][0]
neuron_store_id = cursor.execute(
    "SELECT id FROM data_type WHERE name='neuron_store'"
).fetchall()[0][0]

count_query = """
SELECT 
    name,
    (num_layers * neurons_per_layer) AS num_neurons,
    COUNT(CASE WHEN data_type_id=? THEN 1 END),
    COUNT(CASE WHEN data_type_id=? THEN 1 END)
FROM model JOIN neuron_data
ON model.id=neuron_data.model_id 
GROUP BY model_id
"""

counts = cursor.execute(count_query, (n2g_id, neuron_store_id)).fetchall()
good_emoji = "\N{white heavy check mark}"
bad_emoji = "\N{cross mark}"

for name, num_neurons, n2g_count, neuron_store_count in counts:
    n2g_percent = f"{n2g_count / num_neurons * 100:.0f}%"
    neuron_store_percent = f"{neuron_store_count / num_neurons * 100:.0f}%"
    col_width = 13
    if num_neurons == n2g_count and num_neurons == neuron_store_count:
        print(f"{good_emoji} {name}: ")
    else:
        print(f"{bad_emoji} {name}: ")
    n2g_ratio = f"{n2g_count}/{num_neurons}"
    neuron_store_ratio = f"{neuron_store_count}/{num_neurons}"
    if num_neurons == n2g_count:
        print(
            f"  {good_emoji} {'N2G:':<{5}} {n2g_ratio:>{col_width}} {n2g_percent:>{5}} "
        )
    else:
        print(
            f"  {bad_emoji} {'N2G:':<{5}} {n2g_ratio:>{col_width}} {n2g_percent:>{5}} "
        )
    if num_neurons == neuron_store_count:
        print(
            f"  {good_emoji} {'NS:':<{5}} {neuron_store_ratio:>{col_width}} {neuron_store_percent:>{5}} "
        )
    else:
        print(
            f"  {bad_emoji} {'NS:':<{5}} {neuron_store_ratio:>{col_width}} {neuron_store_percent:>{5}} "
        )
