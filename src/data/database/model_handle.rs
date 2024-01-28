use anyhow::{Context, Result};
use rusqlite::OptionalExtension;

use super::{
    data_types::ModelDataType, service_handle::ServiceHandle, DataTypeHandle, Database, Operation,
};
use crate::{data::Metadata, Index};

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
        const REFERENCE_TABLES: [&str; 4] =
            ["model_data_type", "model_data", "layer_data", "neuron_data"];

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

    pub async fn missing_data_types(&self, service: &ServiceHandle) -> Result<Vec<String>> {
        let mut missing_data_types = vec![];
        for data_type in service.required_data_types().await? {
            if !self.has_data_type(&data_type).await? {
                missing_data_types.push(data_type.name().to_owned());
            }
        }
        Ok(missing_data_types)
    }

    fn add_data_type_inner(&self, data_type: &DataTypeHandle) -> impl Operation<()> {
        const ADD_DATA_TYPE: &str = r#"
        INSERT INTO model_data_type (
            model_id,
            data_type_id
        ) VALUES (
            ?1,
            ?2
        );
        "#;
        let params = (self.id(), data_type.id());

        move |transaction| {
            transaction.prepare(ADD_DATA_TYPE)?.insert(params)?;
            Ok(())
        }
    }

    pub async fn add_data_type(&mut self, data_type: &DataTypeHandle) -> Result<()> {
        let data_type_name = data_type.name().to_owned();
        let model_name = self.name().to_owned();

        self.database
            .execute(self.add_data_type_inner(data_type))
            .await
            .with_context(|| {
                format!("Failed to add data object '{data_type_name}' to model '{model_name}'.")
            })
    }

    fn delete_data_type_inner(&self, data_type: &DataTypeHandle) -> impl Operation<()> {
        const DELETE_DATA: &str = r#"
        DELETE FROM $DATABASE
        WHERE model_id = ?1 AND data_type_id = ?2"#;
        const REFERENCE_TABLES: [&str; 4] =
            ["model_data", "layer_data", "neuron_data", "model_data_type"];

        let params = (self.id, data_type.id());
        move |transaction| {
            for table in REFERENCE_TABLES.iter() {
                let mut statement =
                    transaction.prepare(DELETE_DATA.replace("$DATABASE", table).as_str())?;
                statement.execute(params)?;
            }
            Ok(())
        }
    }

    pub async fn delete_data_type(&mut self, data_type: &DataTypeHandle) -> Result<()> {
        self.database
            .execute(self.delete_data_type_inner(data_type))
            .await
            .with_context(|| {
                format!(
                    "Problem deleting data object '{data_type_name}' from model '{name}.",
                    data_type_name = data_type.name(),
                    name = self.name()
                )
            })
    }

    pub async fn has_data_type(&self, data_type: &DataTypeHandle) -> Result<bool> {
        const CHECK_DATA_TYPE: &str = r#"
        SELECT 
            model_id
        FROM model_data_type
        WHERE model_id = ?1 AND data_type_id = ?2;
        "#;

        let data_type_name = data_type.name();

        let params = (self.id(), data_type.id());

        self.database
            .connection
            .call(move |connection| connection.prepare(CHECK_DATA_TYPE)?.exists(params))
            .await
            .with_context(|| {
                format!(
                    "Failed to check whether model '{}' has data object '{data_type_name}'",
                    self.name()
                )
            })
    }

    pub async fn data_type<D>(&self, data_type: &DataTypeHandle) -> Result<D>
    where
        D: ModelDataType,
    {
        self.database.model_data_type(self, data_type).await
    }

    pub async fn available_services(&self) -> Result<Vec<ServiceHandle>> {
        let mut services = vec![];
        for service in ServiceHandle::all_services(&self.database)
            .await
            .context("Failed to get list of services.")?
        {
            if self
                .missing_data_types(&service)
                .await
                .with_context(|| {
                    format!(
                        "Failed to get list of missing data objects for model '{model_name}' and \
                         service '{service_name}'.",
                        model_name = self.name(),
                        service_name = service.name()
                    )
                })?
                .is_empty()
            {
                services.push(service);
            }
        }
        Ok(services)
    }

    fn add_model_data_inner(
        &self,
        data_type: &DataTypeHandle,
        data: Vec<u8>,
    ) -> impl Operation<()> {
        const ADD_MODEL_DATA: &str = r#"
        INSERT INTO model_data (
            model_id,
            data_type_id,
            data
        ) VALUES (
            ?1,
            ?2,
            ?3
        );
        "#;

        let params = (self.id(), data_type.id(), data);
        move |transaction| {
            transaction.prepare(ADD_MODEL_DATA)?.insert(params)?;
            Ok(())
        }
    }

    pub async fn add_model_data(
        &mut self,
        data_type: &DataTypeHandle,
        data: Vec<u8>,
    ) -> Result<()> {
        let model_name = self.name().to_owned();

        self.database
            .execute(self.add_model_data_inner(data_type, data))
            .await
            .with_context(|| format!("Failed to add model data to model '{model_name}'."))
    }

    fn add_layer_data_inner(
        &self,
        data_type: &DataTypeHandle,
        layer_index: u32,
        data: Vec<u8>,
    ) -> impl Operation<()> {
        const ADD_LAYER_DATA: &str = r#"
        INSERT INTO layer_data (
            model_id,
            data_type_id,
            layer_index,
            data
        ) VALUES (
            ?1,
            ?2,
            ?3,
            ?4
        );
        "#;

        let params = (self.id(), data_type.id(), layer_index, data);

        move |transaction| {
            transaction.prepare(ADD_LAYER_DATA)?.insert(params)?;
            Ok(())
        }
    }

    pub async fn add_layer_data(
        &mut self,
        data_type: &DataTypeHandle,
        layer_index: u32,
        data: Vec<u8>,
    ) -> Result<()> {
        self.database
            .execute(self.add_layer_data_inner(data_type, layer_index, data))
            .await
            .context("Failed to add layer data.")
    }

    fn add_neuron_data_inner(
        &self,
        data_type: &DataTypeHandle,
        layer_index: u32,
        neuron_index: u32,
        data: Vec<u8>,
    ) -> impl Operation<()> {
        const ADD_NEURON_DATA: &str = r#"
        INSERT INTO neuron_data (
            model_id,
            data_type_id,
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

        let params = (self.id(), data_type.id(), layer_index, neuron_index, data);

        move |transaction| {
            transaction.prepare(ADD_NEURON_DATA)?.insert(params)?;
            Ok(())
        }
    }

    pub async fn add_neuron_data(
        &mut self,
        data_type: &DataTypeHandle,
        layer_index: u32,
        neuron_index: u32,
        data: Vec<u8>,
    ) -> Result<()> {
        self.database
            .execute(self.add_neuron_data_inner(data_type, layer_index, neuron_index, data))
            .await
            .context("Failed to add neuron data.")
    }

    pub async fn add_data(
        &mut self,
        data_type: &DataTypeHandle,
        index: Index,
        data: Vec<u8>,
    ) -> Result<()> {
        match index {
            Index::Model => self.add_model_data(data_type, data).await,
            Index::Layer(layer_index) => self.add_layer_data(data_type, layer_index, data).await,
            Index::Neuron(layer_index, neuron_index) => {
                self.add_neuron_data(data_type, layer_index, neuron_index, data)
                    .await
            }
        }
    }

    pub async fn model_data(&self, data_type: &DataTypeHandle) -> Result<Option<Vec<u8>>> {
        const GET_MODEL_DATA: &str = r#"
        SELECT
            data
        FROM model_data
        WHERE model_id = ?1 AND data_type_id = ?2;
        "#;

        let params = (self.id(), data_type.id());

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
                    data_type.name()
                )
            })
    }

    pub async fn layer_data(
        &self,
        data_type: &DataTypeHandle,
        layer_index: u32,
    ) -> Result<Option<Vec<u8>>> {
        const GET_LAYER_DATA: &str = r#"
        SELECT
            data
        FROM layer_data
        WHERE model_id = ?1 AND data_type_id = ?2 AND layer_index = ?3;
        "#;

        let params = (self.id(), data_type.id(), layer_index);
        self.database
            .connection
            .call(move |connection| {
                let mut statement = connection.prepare(GET_LAYER_DATA)?;
                statement.query_row(params, |row| row.get(0)).optional()
            })
            .await
            .with_context(|| {
                format!(
                    "Failed to get layer data for layer {layer_index} data object '{}' for model \
                     '{}'.",
                    self.name(),
                    data_type.name()
                )
            })
    }

    pub async fn neuron_data(
        &self,
        data_type: &DataTypeHandle,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<Option<Vec<u8>>> {
        const GET_NEURON_DATA: &str = r#"
        SELECT
            data
        FROM neuron_data
        WHERE model_id = ?1 AND data_type_id = ?2 AND layer_index = ?3 AND neuron_index = ?4;
        "#;

        let params = (self.id(), data_type.id(), layer_index, neuron_index);

        self.database
            .connection
            .call(move |connection| {
                let mut statement = connection.prepare(GET_NEURON_DATA)?;
                statement.query_row(params, |row| row.get(0)).optional()
            })
            .await
            .with_context(|| {
                format!(
                    "Failed to get neuron data for neuron l{layer_index}n{neuron_index} for data \
                     object '{}' for model '{}'.",
                    data_type.name(),
                    self.name(),
                )
            })
    }

    pub async fn data(&self, data_type: &DataTypeHandle, index: Index) -> Result<Option<Vec<u8>>> {
        match index {
            Index::Model => self.model_data(data_type).await,
            Index::Layer(layer_index) => self.layer_data(data_type, layer_index).await,
            Index::Neuron(layer_index, neuron_index) => {
                self.neuron_data(data_type, layer_index, neuron_index).await
            }
        }
    }
}
