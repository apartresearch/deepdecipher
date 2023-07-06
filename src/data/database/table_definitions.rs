const MODEL_TABLE: &str = r#"
CREATE TABLE model (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    name                    TEXT NOT NULL UNIQUE,
    num_layers              INTEGER NOT NULL,
    neurons_per_layer       INTEGER NOT NULL,
    activation_function     TEXT NOT NULL,
    num_total_parameters    INTEGER NOT NULL,
    dataset                 TEXT NOT NULL
    CHECK (num_layers >= 0 AND neurons_per_layer >= 0 AND num_total_parameters >= 0)
  ) STRICT;
"#;

const SERVICE_TABLE: &str = r#"
CREATE TABLE service (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    name                    TEXT NOT NULL UNIQUE,
    provider                TEXT NOT NULL,
    provider_args           BLOB NOT NULL
  ) STRICT;
"#;

const MODEL_SERVICE_TABLE: &str = r#"
CREATE TABLE model_service (
    model_id                   INTEGER NOT NULL,
    service_id                 INTEGER NOT NULL,
    FOREIGN KEY(model_id) REFERENCES model(id),
    FOREIGN KEY(service_id) REFERENCES service(id)
  ) STRICT;
"#;

const DATA_OBJECT_TABLE: &str = r#"
CREATE TABLE data_object (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    name                    TEXT NOT NULL UNIQUE,
    type                    TEXT NOT NULL,
    type_args               BLOB NOT NULL
  ) STRICT;
"#;

const MODEL_DATA_OBJECT_TABLE: &str = r#"
CREATE TABLE model_data_object (
    model_id                INTEGER NOT NULL,
    data_object_id          INTEGER NOT NULL,
    FOREIGN KEY(model_id) REFERENCES model(id),
    FOREIGN KEY(data_object_id) REFERENCES data_object(id)
)
"#;

const MODEL_DATA_TABLE: &str = r#"
CREATE TABLE model_data (
    model_id                INTEGER NOT NULL,
    data_object_id          INTEGER NOT NULL,
    data                    BLOB NOT NULL,
    PRIMARY KEY(model_id, data_object_id),
    FOREIGN KEY(model_id) REFERENCES model(id),
    FOREIGN KEY(data_object_id) REFERENCES data_object(id)
  ) STRICT;
"#;

const LAYER_DATA_TABLE: &str = r#"
CREATE TABLE layer_data (
    model_id                INTEGER NOT NULL,
    data_object_id          INTEGER NOT NULL,
    layer_index             INTEGER NOT NULL,
    data                    BLOB NOT NULL,
    PRIMARY KEY(model_id, data_object_id, layer_index),
    FOREIGN KEY(model_id) REFERENCES model(id),
    FOREIGN KEY(data_object_id) REFERENCES data_object(id)
    CHECK (layer_index >= 0)
  ) STRICT;
"#;

const NEURON_DATA_TABLE: &str = r#"
CREATE TABLE neuron_data (
    model_id                INTEGER NOT NULL,
    data_object_id          INTEGER NOT NULL,
    layer_index             INTEGER NOT NULL,
    neuron_index            INTEGER NOT NULL,
    data                    BLOB NOT NULL,
    PRIMARY KEY(model_id, data_object_id, layer_index, neuron_index),
    FOREIGN KEY(model_id) REFERENCES model(id),
    FOREIGN KEY(data_object_id) REFERENCES data_object(id)
    CHECK (layer_index >= 0 AND neuron_index >= 0)
  ) STRICT;
"#;

pub const TABLES: [&str; 8] = [
    MODEL_TABLE,
    SERVICE_TABLE,
    MODEL_SERVICE_TABLE,
    DATA_OBJECT_TABLE,
    MODEL_DATA_OBJECT_TABLE,
    MODEL_DATA_TABLE,
    LAYER_DATA_TABLE,
    NEURON_DATA_TABLE,
];
