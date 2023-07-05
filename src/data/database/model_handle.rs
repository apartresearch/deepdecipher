use crate::data::{LayerMetadata, Metadata};

use super::{
    data_types::ModelDataObject, service_handle::ServiceHandle, DataObjectHandle, Database,
};

use anyhow::{Context, Result};
use rusqlite::OptionalExtension;

#[derive(Clone)]
pub struct ModelHandle {
    id: i64,
    metadata: Metadata,
    database: Database,
}

impl ModelHandle {
    pub(super) async fn create(database: &Database, metadata: Metadata) -> Result<Self> {
        const ADD_MODEL: &str = r#"
        INSERT INTO model (
            name,
            num_layers,
            neurons_per_layer,
            activation_function,
            num_total_parameters,
            dataset
        ) VALUES (
            ?1,
            ?2,
            ?3,
            ?4,
            ?5,
            ?6
        );
        "#;

        let metadata2 = metadata.clone();

        database
            .connection
            .call(move |connection| {
                connection.execute(
                    ADD_MODEL,
                    (
                        &metadata2.name,
                        &metadata2.layers.len(),
                        &metadata2.layers[0].num_neurons,
                        &metadata2.activation_function,
                        &metadata2.num_total_parameters,
                        &metadata2.dataset,
                    ),
                )
            })
            .await?;
        let id = database.latest_id("model").await?;
        let model = ModelHandle {
            id,
            metadata,
            database: database.clone(),
        };

        model
            .add_service(&database.service("metadata").await?.unwrap())
            .await?;

        Ok(model)
    }

    pub(super) async fn new(database: Database, model_name: String) -> Result<Option<Self>> {
        const GET_MODEL: &str = r#"
        SELECT
            id,
            name,
            num_layers,
            neurons_per_layer,
            activation_function,
            num_total_parameters,
            dataset
        FROM model
        WHERE name = ?1;
        "#;

        let params = (model_name.clone(),);
        let metadata = database
            .connection
            .call(|connection| {
                let mut statement = connection.prepare(GET_MODEL)?;
                let mut rows = statement.query(params)?;

                let row = if let Some(row) = rows.next()? {
                    row
                } else {
                    return Ok(None);
                };

                let num_layers: u32 = row.get(2)?;
                let neurons_per_layer = row.get(3)?;
                let layers = vec![
                    LayerMetadata {
                        num_neurons: neurons_per_layer
                    };
                    num_layers as usize
                ];

                Ok(Some((
                    row.get(0)?,
                    Metadata {
                        name: row.get(1)?,
                        layers,
                        activation_function: row.get(4)?,
                        num_total_neurons: num_layers * neurons_per_layer,
                        num_total_parameters: row.get(5)?,
                        dataset: row.get(6)?,
                    },
                )))
            })
            .await?;

        Ok(metadata.map(|(id, metadata)| ModelHandle {
            id,
            metadata,
            database,
        }))
    }

    pub(super) fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn database(&self) -> &Database {
        &self.database
    }

    pub async fn delete(self) -> Result<()> {
        const DELETE_MODEL_REFERENCES: &str = r#"
        DELETE FROM $DATABASE
        WHERE model_id = ?1;
        "#;
        const DELETE_MODEL: &str = r#"
        DELETE FROM model
        WHERE id = ?1;
        "#;
        const REFERENCE_TABLES: [&str; 5] = [
            "model_service",
            "model_data_object",
            "model_data",
            "layer_data",
            "neuron_data",
        ];

        let name = self.name();

        for table in REFERENCE_TABLES.iter() {
            let params = (self.id,);
            self.database
                .connection
                .call(move |connection| {
                    let mut statement = connection
                        .prepare(DELETE_MODEL_REFERENCES.replace("$DATABASE", table).as_str())?;
                    statement.execute(params)?;
                    Ok(())
                })
                .await?;
        }
        let params = (self.id,);
        self.database
            .connection
            .call(move |connection| {
                let mut statement = connection.prepare(DELETE_MODEL)?;
                statement.execute(params)?;
                Ok(())
            })
            .await
            .with_context(|| format!("Problem deleting model '{name}'."))
    }

    pub async fn add_service(&self, service: &ServiceHandle) -> Result<()> {
        const ADD_MODEL_SERVICE: &str = r#"
        INSERT INTO model_service (
            model_id,
            service_id
        ) VALUES (
            ?1,
            ?2
        );
        "#;

        let params = (self.id, service.id());

        self.database
            .connection
            .call(move |connection| connection.execute(ADD_MODEL_SERVICE, params))
            .await?;

        Ok(())
    }

    pub async fn add_data_object(&self, data_object: &DataObjectHandle) -> Result<()> {
        const ADD_DATA_OBJECT: &str = r#"
        INSERT INTO model_data_object (
            model_id,
            data_object_id
        ) VALUES (
            ?1,
            ?2
        );
        "#;

        let data_object_name = data_object.name();

        let params = (self.id(), data_object.id());

        self.database
            .connection
            .call(move |connection| connection.execute(ADD_DATA_OBJECT, params).map(drop))
            .await
            .with_context(|| format!("Failed to add data object '{data_object_name}' to model."))
    }

    pub async fn has_data_object(&self, data_object: &DataObjectHandle) -> Result<bool> {
        const CHECK_DATA_OBJECT: &str = r#"
        SELECT 
            model_id
        FROM model_data_object
        WHERE model_id = ?1 AND data_object_id = ?2;
        "#;

        let data_object_name = data_object.name();

        let params = (self.id(), data_object.id());

        self.database
            .connection
            .call(move |connection| connection.prepare(CHECK_DATA_OBJECT)?.exists(params))
            .await
            .with_context(|| {
                format!(
                    "Failed to check whether model '{}' has data object '{data_object_name}'",
                    self.name()
                )
            })
    }

    pub async fn data_object<D>(&self, data_object: &DataObjectHandle) -> Result<D>
    where
        D: ModelDataObject,
    {
        self.database.model_data_object(self, data_object).await
    }

    pub async fn add_model_data(
        &self,
        data_object: &DataObjectHandle,
        data: Vec<u8>,
    ) -> Result<()> {
        const ADD_MODEL_DATA: &str = r#"
        INSERT INTO model_data (
            model_id,
            data_object_id,
            data
        ) VALUES (
            ?1,
            ?2,
            ?3
        );
        "#;

        let params = (self.id(), data_object.id(), data);

        self.database
            .connection
            .call(move |connection| connection.execute(ADD_MODEL_DATA, params).map(drop))
            .await
            .context("Failed to add model data.")
    }

    pub async fn add_layer_data(
        &self,
        data_object: &DataObjectHandle,
        layer_index: u32,
        data: Vec<u8>,
    ) -> Result<()> {
        const ADD_LAYER_DATA: &str = r#"
        INSERT INTO layer_data (
            model_id,
            data_object_id,
            layer_index,
            data
        ) VALUES (
            ?1,
            ?2,
            ?3,
            ?4
        );
        "#;

        let params = (self.id(), data_object.id(), layer_index, data);

        self.database
            .connection
            .call(|connection| connection.execute(ADD_LAYER_DATA, params).map(drop))
            .await
            .context("Failed to add layer data.")
    }

    pub async fn add_neuron_data(
        &self,
        data_object: &DataObjectHandle,
        layer_index: u32,
        neuron_index: u32,
        data: Vec<u8>,
    ) -> Result<()> {
        const ADD_NEURON_DATA: &str = r#"
        INSERT INTO neuron_data (
            model_id,
            data_object_id,
            layer_index,
            neuron_index,
            data
        ) VALUES (
            ?1,
            ?2,
            ?3,
            ?4,
            ?5
        );
        "#;

        let params = (self.id(), data_object.id(), layer_index, neuron_index, data);

        self.database
            .connection
            .call(|connection| connection.execute(ADD_NEURON_DATA, params).map(drop))
            .await
            .context("Failed to add neuron data.")
    }

    pub async fn model_data(&self, data_object: &DataObjectHandle) -> Result<Option<Vec<u8>>> {
        const GET_MODEL_DATA: &str = r#"
        SELECT
            data
        FROM model_data
        WHERE model_id = ?1 AND data_object_id = ?2;
        "#;

        let params = (self.id(), data_object.id());

        self.database
            .connection
            .call(move |connection| {
                let mut statement = connection.prepare(GET_MODEL_DATA)?;
                statement.query_row(params, |row| row.get(0)).optional()
            })
            .await
            .with_context(|| {
                format!(
                    "Failed to get model data for data object '{}' for model '{}'.",
                    self.name(),
                    data_object.name()
                )
            })
    }

    pub async fn layer_data(
        &self,
        data_object: &DataObjectHandle,
        layer_index: u32,
    ) -> Result<Option<Vec<u8>>> {
        const GET_LAYER_DATA: &str = r#"
        SELECT
            data
        FROM layer_data
        WHERE model_id = ?1 AND data_object_id = ?2 AND layer_index = ?3;
        "#;

        let params = (self.id(), data_object.id(), layer_index);
        self.database
            .connection
            .call(move |connection| {
                let mut statement = connection.prepare(GET_LAYER_DATA)?;
                statement.query_row(params, |row| row.get(0)).optional()
            })
            .await
            .with_context(|| {
                format!(
                    "Failed to get layer data for layer {layer_index} data object '{}' for model '{}'.",
                    self.name(),
                    data_object.name()
                )
            })
    }

    pub async fn neuron_data(
        &self,
        data_object: &DataObjectHandle,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<Option<Vec<u8>>> {
        const GET_NEURON_DATA: &str = r#"
        SELECT
            data
        FROM neuron_data
        WHERE model_id = ?1 AND data_object_id = ?2 AND layer_index = ?3 AND neuron_index = ?4;
        "#;

        let params = (self.id(), data_object.id(), layer_index, neuron_index);

        self.database
            .connection
            .call(move |connection| {
                let mut statement = connection.prepare(GET_NEURON_DATA)?;
                statement.query_row(params, |row| row.get(0)).optional()
            })
            .await
            .with_context(|| {
                format!(
                    "Failed to get neuron data for neuron l{layer_index}n{neuron_index} for data object '{}' for model '{}'.",
                    data_object.name(),
                    self.name(),
                )
            })
    }
}
