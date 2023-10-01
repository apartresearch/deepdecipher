use anyhow::{bail, Context, Result};
use async_trait::async_trait;

use crate::data::{neuron2graph::Graph, DataTypeHandle, ModelHandle};

use super::{data_type::DataValidationError, DataTypeDiscriminants, ModelDataType};

pub struct Neuron2Graph {
    model: ModelHandle,
    data_type: DataTypeHandle,
}

#[async_trait]
impl ModelDataType for Neuron2Graph {
    async fn new(model: ModelHandle, data_type: DataTypeHandle) -> Result<Option<Self>> {
        match data_type.data_type().into() {
            DataTypeDiscriminants::Neuron2Graph => Ok(Some(Self { model, data_type })),
            _ => bail!("Invalid type for Neuron2Graph data object."),
        }
    }

    fn data_type() -> DataTypeDiscriminants {
        DataTypeDiscriminants::Neuron2Graph
    }

    fn model_handle(&self) -> &ModelHandle {
        &self.model
    }

    async fn validate(&self) -> anyhow::Result<Result<(), DataValidationError>> {
        let missing_items: Vec<_> = self
            .model
            .missing_neuron_items(&self.data_type)
            .await?
            .collect();
        Ok(if missing_items.is_empty() {
            Ok(())
        } else {
            Err(DataValidationError::MissingItems { missing_items })
        })
    }
}

impl Neuron2Graph {
    pub async fn neuron_graph(&self, layer_index: u32, neuron_index: u32) -> Result<Graph> {
        let model_name = self.model.name();
        let raw_data = self.model
            .neuron_data(&self.data_type, layer_index, neuron_index)
            .await?
            .with_context(|| {
                format!("Database has no neuron2graph data for neuron l{layer_index}n{neuron_index} in model '{model_name}'")
            })?;
        Graph::from_binary(raw_data).with_context(|| format!("Failed to unpack neuron2graph graph for neuron l{layer_index}n{neuron_index} in model '{model_name}'."))
    }
}
