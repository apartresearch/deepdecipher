use crate::{data::Metadata, Index};

use super::{
    data_types::ModelDataObject, service_handle::ServiceHandle, DataObjectHandle, Database,
    Operation,
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
    fn create_inner(database: Database, metadata: Metadata) -> impl Operation<Self> {
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

        let params = (
            metadata.name.clone(),
            metadata.num_layers,
            metadata.layer_size,
            metadata.activation_function.clone(),
            metadata.num_total_parameters,
            metadata.dataset.clone(),
        );

        |transaction| {
            let id = transaction.prepare(ADD_MODEL)?.insert(params)?;
            let model = ModelHandle {
                id,
                metadata,
                database,
            };
            Ok(model)
        }
    }

    pub(super) async fn create(mut database: Database, metadata: Metadata) -> Result<Self> {
        database
            .execute(Self::create_inner(database.clone(), metadata))
            .await
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
                let layer_size = row.get(3)?;

                Ok(Some((
                    row.get(0)?,
                    Metadata {
                        name: row.get(1)?,
                        num_layers,
                        layer_size,
                        activation_function: row.get(4)?,
                        num_total_neurons: num_layers * layer_size,
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

    fn delete_inner(&self) -> impl Operation<()> {
        const DELETE_MODEL_REFERENCES: &str = r#"
        DELETE FROM $TABLE
        WHERE model_id = ?1;
        "#;
        const DELETE_MODEL: &str = r#"
        DELETE FROM model
        WHERE id = ?1;
        "#;
        const REFERENCE_TABLES: [&str; 4] = [
            "model_data_object",
            "model_data",
            "layer_data",
            "neuron_data",
        ];

        let params = (self.id,);
        move |transaction| {
            for table in REFERENCE_TABLES.iter() {
                let mut statement = transaction
                    .prepare(DELETE_MODEL_REFERENCES.replace("$TABLE", table).as_str())?;
                statement.execute(params)?;
            }
            transaction.prepare(DELETE_MODEL)?.execute(params)?;
            Ok(())
        }
    }

    pub async fn delete(mut self) -> Result<()> {
        let name = self.name().to_owned();

        self.database
            .execute(self.delete_inner())
            .await
            .with_context(|| format!("Problem deleting model '{name}'."))
    }

    pub async fn missing_data_objects(&self, service: &ServiceHandle) -> Result<Vec<String>> {
        let mut missing_data_objects = vec![];
        for data_object in service.required_data_objects().await? {
            if !self.has_data_object(&data_object).await? {
                missing_data_objects.push(data_object.name().to_owned());
            }
        }
        Ok(missing_data_objects)
    }

    fn add_data_object_inner(&self, data_object: &DataObjectHandle) -> impl Operation<()> {
        const ADD_DATA_OBJECT: &str = r#"
        INSERT INTO model_data_object (
            model_id,
            data_object_id
        ) VALUES (
            ?1,
            ?2
        );
        "#;
        let params = (self.id(), data_object.id());

        move |transaction| {
            transaction.prepare(ADD_DATA_OBJECT)?.insert(params)?;
            Ok(())
        }
    }

    pub async fn add_data_object(&mut self, data_object: &DataObjectHandle) -> Result<()> {
        let data_object_name = data_object.name().to_owned();
        let model_name = self.name().to_owned();

        self.database
            .execute(self.add_data_object_inner(data_object))
            .await
            .with_context(|| {
                format!("Failed to add data object '{data_object_name}' to model '{model_name}'.")
            })
    }

    fn delete_data_object_inner(&self, data_object: &DataObjectHandle) -> impl Operation<()> {
        const DELETE_DATA: &str = r#"
        DELETE FROM $DATABASE
        WHERE model_id = ?1 AND data_object_id = ?2"#;
        const REFERENCE_TABLES: [&str; 4] = [
            "model_data",
            "layer_data",
            "neuron_data",
            "model_data_object",
        ];

        let params = (self.id, data_object.id());
        move |transaction| {
            for table in REFERENCE_TABLES.iter() {
                let mut statement =
                    transaction.prepare(DELETE_DATA.replace("$DATABASE", table).as_str())?;
                statement.execute(params)?;
            }
            Ok(())
        }
    }

    pub async fn delete_data_object(&mut self, data_object: &DataObjectHandle) -> Result<()> {
        self.database
            .execute(self.delete_data_object_inner(data_object))
            .await
            .with_context(|| {
                format!(
                    "Problem deleting data object '{data_object_name}' from model '{name}.",
                    data_object_name = data_object.name(),
                    name = self.name()
                )
            })
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

    fn add_model_data_inner(
        &self,
        data_object: &DataObjectHandle,
        data: Vec<u8>,
    ) -> impl Operation<()> {
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
        move |transaction| {
            transaction.prepare(ADD_MODEL_DATA)?.insert(params)?;
            Ok(())
        }
    }

    pub async fn add_model_data(
        &mut self,
        data_object: &DataObjectHandle,
        data: Vec<u8>,
    ) -> Result<()> {
        let model_name = self.name().to_owned();

        self.database
            .execute(self.add_model_data_inner(data_object, data))
            .await
            .with_context(|| format!("Failed to add model data to model '{model_name}'."))
    }

    fn add_layer_data_inner(
        &self,
        data_object: &DataObjectHandle,
        layer_index: u32,
        data: Vec<u8>,
    ) -> impl Operation<()> {
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

        move |transaction| {
            transaction.prepare(ADD_LAYER_DATA)?.insert(params)?;
            Ok(())
        }
    }

    pub async fn add_layer_data(
        &mut self,
        data_object: &DataObjectHandle,
        layer_index: u32,
        data: Vec<u8>,
    ) -> Result<()> {
        self.database
            .execute(self.add_layer_data_inner(data_object, layer_index, data))
            .await
            .context("Failed to add layer data.")
    }

    fn add_neuron_data_inner(
        &self,
        data_object: &DataObjectHandle,
        layer_index: u32,
        neuron_index: u32,
        data: Vec<u8>,
    ) -> impl Operation<()> {
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

        move |transaction| {
            transaction.prepare(ADD_NEURON_DATA)?.insert(params)?;
            Ok(())
        }
    }

    pub async fn add_neuron_data(
        &mut self,
        data_object: &DataObjectHandle,
        layer_index: u32,
        neuron_index: u32,
        data: Vec<u8>,
    ) -> Result<()> {
        self.database
            .execute(self.add_neuron_data_inner(data_object, layer_index, neuron_index, data))
            .await
            .context("Failed to add neuron data.")
    }

    pub async fn add_data(
        &mut self,
        data_object: &DataObjectHandle,
        index: Index,
        data: Vec<u8>,
    ) -> Result<()> {
        match index {
            Index::Model => self.add_model_data(data_object, data).await,
            Index::Layer(layer_index) => self.add_layer_data(data_object, layer_index, data).await,
            Index::Neuron(layer_index, neuron_index) => {
                self.add_neuron_data(data_object, layer_index, neuron_index, data)
                    .await
            }
        }
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

    pub async fn data(
        &self,
        data_object: &DataObjectHandle,
        index: Index,
    ) -> Result<Option<Vec<u8>>> {
        match index {
            Index::Model => self.model_data(data_object).await,
            Index::Layer(layer_index) => self.layer_data(data_object, layer_index).await,
            Index::Neuron(layer_index, neuron_index) => {
                self.neuron_data(data_object, layer_index, neuron_index)
                    .await
            }
        }
    }
}
