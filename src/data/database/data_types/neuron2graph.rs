use anyhow::{bail, Context, Result};
use async_trait::async_trait;

use crate::data::{neuron2graph::Graph, DataObjectHandle, ModelHandle};

use super::{DataTypeDiscriminants, ModelDataObject};

pub struct Neuron2Graph {
    model: ModelHandle,
    data_object: DataObjectHandle,
}

#[async_trait]
impl ModelDataObject for Neuron2Graph {
    async fn new(model: &ModelHandle, datatype: DataTypeDiscriminants) -> Result<Option<Self>> {
        let data_object = model
            .database()
            .data_object("neuron2graph")
            .await?
            .context("No neuron2graph data object in database.")?;
        match datatype {
            DataTypeDiscriminants::Neuron2Graph => Ok(Some(Self {
                model: model.clone(),
                data_object,
            })),
            _ => bail!("Invalid type for Neuron2Graph data object."),
        }
    }

    fn data_type() -> DataTypeDiscriminants {
        DataTypeDiscriminants::Neuroscope
    }

    fn model_handle(&self) -> &ModelHandle {
        &self.model
    }
}

impl Neuron2Graph {
    pub async fn neuron_graph(&self, layer_index: u32, neuron_index: u32) -> Result<Graph> {
        let model_name = self.model.name();
        let raw_data = self.model
            .neuron_data(&self.data_object, layer_index, neuron_index)
            .await?
            .with_context(|| {
                format!("Database has no neuron2graph data for neuron l{layer_index}n{neuron_index} in model '{model_name}'")
            })?;
        Graph::from_binary(raw_data).with_context(|| format!("Failed to unpack neuron2graph graph for neuron l{layer_index}n{neuron_index} in model '{model_name}'."))
    }
}
