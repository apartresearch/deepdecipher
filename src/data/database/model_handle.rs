use crate::data::{LayerMetadata, Metadata};

use super::{data_types::ModelDataObject, Database};

use anyhow::{Context, Result};
use rusqlite::OptionalExtension;

#[derive(Clone)]
pub struct ModelHandle {
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
        let model = ModelHandle {
            metadata,
            database: database.clone(),
        };

        model.add_service("metadata").await?;

        Ok(model)
    }

    pub(super) async fn new(database: Database, model_name: String) -> Result<Option<Self>> {
        const GET_MODEL: &str = r#"
        SELECT
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

                let num_layers: u32 = row.get(1)?;
                let neurons_per_layer = row.get(2)?;
                let layers = vec![
                    LayerMetadata {
                        num_neurons: neurons_per_layer
                    };
                    num_layers as usize
                ];

                Ok(Some(Metadata {
                    name: row.get(0)?,
                    layers,
                    activation_function: row.get(3)?,
                    num_total_neurons: num_layers * neurons_per_layer,
                    num_total_parameters: row.get(4)?,
                    dataset: row.get(5)?,
                }))
            })
            .await?;

        Ok(metadata.map(|metadata| ModelHandle { metadata, database }))
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    pub async fn add_service(&self, service_name: &str) -> Result<()> {
        const ADD_MODEL_SERVICE: &str = r#"
        INSERT INTO model_service (
            model,
            service
        ) VALUES (
            ?1,
            ?2
        );
        "#;

        let params = (self.name().to_owned(), service_name.to_owned());

        self.database
            .connection
            .call(|connection| connection.execute(ADD_MODEL_SERVICE, params))
            .await?;

        Ok(())
    }

    pub async fn add_data_object(&self, data_object_name: impl AsRef<str>) -> Result<()> {
        const ADD_DATA_OBJECT: &str = r#"
        INSERT INTO model_data_object (
            model,
            data_object
        ) VALUES (
            ?1,
            ?2
        );
        "#;

        let params = (self.name().to_owned(), data_object_name.as_ref().to_owned());

        self.database
            .connection
            .call(|connection| connection.execute(ADD_DATA_OBJECT, params).map(drop))
            .await
            .context("Failed to add data object to model.")
    }

    pub async fn has_data_object(&self, data_object_name: impl AsRef<str>) -> Result<bool> {
        const CHECK_DATA_OBJECT: &str = r#"
        SELECT 
            model
        FROM model_data_object
        WHERE model = ?1 AND data_object = ?2;
        "#;

        let data_object_name = data_object_name.as_ref();

        let params = (self.name().to_owned(), data_object_name.to_owned());
        self.database
            .connection
            .call(|connection| connection.prepare(CHECK_DATA_OBJECT)?.exists(params))
            .await
            .with_context(|| {
                format!(
                    "Failed to check whether model '{}' has data object '{data_object_name}'",
                    self.name()
                )
            })
    }

    pub async fn data_object<D>(&self, data_object_name: impl AsRef<str>) -> Result<Option<D>>
    where
        D: ModelDataObject,
    {
        let data_object = self.database.data_object(data_object_name).await?;
        if let Some(data_object) = data_object {
            self.database
                .model_data_object(self, &data_object)
                .await
                .map(Some)
        } else {
            Ok(None)
        }
    }

    pub async fn add_model_data(&self, data_object: impl AsRef<str>, data: Vec<u8>) -> Result<()> {
        const ADD_MODEL_DATA: &str = r#"
        INSERT INTO model_data (
            model,
            data_object,
            data
        ) VALUES (
            ?1,
            ?2,
            ?3
        );
        "#;

        let params = (
            self.name().to_owned(),
            data_object.as_ref().to_owned(),
            data,
        );

        self.database
            .connection
            .call(|connection| connection.execute(ADD_MODEL_DATA, params).map(drop))
            .await
            .context("Failed to add model data.")
    }

    pub async fn add_layer_data(
        &self,
        data_object: impl AsRef<str>,
        layer_index: u32,
        data: Vec<u8>,
    ) -> Result<()> {
        const ADD_LAYER_DATA: &str = r#"
        INSERT INTO layer_data (
            model,
            data_object,
            layer_index,
            data
        ) VALUES (
            ?1,
            ?2,
            ?3,
            ?4
        );
        "#;

        let params = (
            self.name().to_owned(),
            data_object.as_ref().to_owned(),
            layer_index,
            data,
        );

        self.database
            .connection
            .call(|connection| connection.execute(ADD_LAYER_DATA, params).map(drop))
            .await
            .context("Failed to add layer data.")
    }

    pub async fn add_neuron_data(
        &self,
        data_object: impl AsRef<str>,
        layer_index: u32,
        neuron_index: u32,
        data: Vec<u8>,
    ) -> Result<()> {
        const ADD_NEURON_DATA: &str = r#"
        INSERT INTO neuron_data (
            model,
            data_object,
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

        let params = (
            self.name().to_owned(),
            data_object.as_ref().to_owned(),
            layer_index,
            neuron_index,
            data,
        );

        self.database
            .connection
            .call(|connection| connection.execute(ADD_NEURON_DATA, params).map(drop))
            .await
            .context("Failed to add neuron data.")
    }

    pub async fn get_model_data(&self, data_object: impl AsRef<str>) -> Result<Option<Vec<u8>>> {
        const GET_MODEL_DATA: &str = r#"
        SELECT
            data
        FROM model_data
        WHERE model = ?1 AND data_object = ?2;
        "#;

        let params = (self.name().to_owned(), data_object.as_ref().to_owned());

        self.database
            .connection
            .call(|connection| {
                let mut statement = connection.prepare(GET_MODEL_DATA)?;
                statement.query_row(params, |row| row.get(0)).optional()
            })
            .await
            .with_context(|| {
                format!(
                    "Failed to get model data for model '{}' for data object '{}'.",
                    self.name(),
                    data_object.as_ref()
                )
            })
    }

    pub async fn get_layer_data(
        &self,
        data_object: impl AsRef<str>,
        layer_index: u32,
    ) -> Result<Option<Vec<u8>>> {
        const GET_LAYER_DATA: &str = r#"
        SELECT
            data
        FROM layer_data
        WHERE model = ?1 AND data_object = ?2 AND layer_index = ?3;
        "#;

        let params = (
            self.name().to_owned(),
            data_object.as_ref().to_owned(),
            layer_index,
        );
        self.database
            .connection
            .call(|connection| {
                let mut statement = connection.prepare(GET_LAYER_DATA)?;
                statement.query_row(params, |row| row.get(0)).optional()
            })
            .await
            .context("Failed to get layer data")
    }

    pub async fn get_neuron_data(
        &self,
        data_object: impl AsRef<str>,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<Option<Vec<u8>>> {
        const GET_NEURON_DATA: &str = r#"
        SELECT
            data
        FROM neuron_data
        WHERE model = ?1 AND data_object = ?2 AND layer_index = ?3 AND neuron_index = ?4;
        "#;

        let params = (
            self.name().to_owned(),
            data_object.as_ref().to_owned(),
            layer_index,
            neuron_index,
        );

        self.database
            .connection
            .call(|connection| {
                let mut statement = connection.prepare(GET_NEURON_DATA)?;
                statement.query_row(params, |row| row.get(0)).optional()
            })
            .await
            .context("Failed to get neuron data")
    }
}
