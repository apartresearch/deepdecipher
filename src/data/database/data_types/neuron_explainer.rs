use anyhow::{bail, Context, Result};
use async_trait::async_trait;

use crate::data::{
    database::ModelHandle, neuron_explainer_page::NeuronExplainerPage, DataObjectHandle,
};

use super::{
    data_object::{DataValidationError, ModelDataObject},
    DataTypeDiscriminants,
};

pub struct NeuronExplainer {
    model: ModelHandle,
    data_object: DataObjectHandle,
}

#[async_trait]
impl ModelDataObject for NeuronExplainer {
    async fn new(model: ModelHandle, data_object: DataObjectHandle) -> Result<Option<Self>> {
        match data_object.data_type().into() {
            DataTypeDiscriminants::NeuronExplainer => Ok(Some(Self { model, data_object })),
            _ => bail!("Invalid type for neuron explainer data object.",),
        }
    }

    fn data_type() -> DataTypeDiscriminants {
        DataTypeDiscriminants::NeuronExplainer
    }

    fn model_handle(&self) -> &ModelHandle {
        &self.model
    }

    async fn validate(&self) -> anyhow::Result<Result<(), DataValidationError>> {
        let missing_items: Vec<_> = self
            .model
            .missing_neuron_items(&self.data_object)
            .await?
            .collect();
        Ok(if missing_items.is_empty() {
            Ok(())
        } else {
            Err(DataValidationError::MissingItems { missing_items })
        })
    }
}

impl NeuronExplainer {
    pub async fn neuron_page(
        &self,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<Option<NeuronExplainerPage>> {
        let model_name = self.model.name();
        let raw_data = self.model
            .neuron_data( &self.data_object, layer_index, neuron_index)
            .await.with_context(|| {
                format!("Failed to get neuron explainer neuron data for neuron l{layer_index}n{neuron_index} in model '{model_name}'.")
            })?;
        raw_data.map(|raw_data| NeuronExplainerPage::from_binary(raw_data.as_slice())
            .with_context(|| {
                format!("Failed to deserialize neuron explainer neuron data for neuron l{layer_index}n{neuron_index} in model '{model_name}'.")
            })).transpose()
    }
}
