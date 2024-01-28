use anyhow::{bail, Context, Result};
use async_trait::async_trait;

use super::{
    data_type::{DataValidationError, ModelDataType},
    DataTypeDiscriminants,
};
use crate::data::{
    data_objects::{DataObject, NeuronExplainerPage},
    database::ModelHandle,
    DataTypeHandle,
};

pub struct NeuronExplainer {
    model: ModelHandle,
    data_type: DataTypeHandle,
}

#[async_trait]
impl ModelDataType for NeuronExplainer {
    async fn new(model: ModelHandle, data_type: DataTypeHandle) -> Result<Option<Self>> {
        match data_type.data_type().into() {
            DataTypeDiscriminants::NeuronExplainer => Ok(Some(Self { model, data_type })),
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

impl NeuronExplainer {
    pub async fn neuron_page(
        &self,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<Option<NeuronExplainerPage>> {
        let model_name = self.model.name();
        let raw_data = self
            .model
            .neuron_data(&self.data_type, layer_index, neuron_index)
            .await
            .with_context(|| {
                format!(
                    "Failed to get neuron explainer neuron data for neuron \
                     l{layer_index}n{neuron_index} in model '{model_name}'."
                )
            })?;
        raw_data
            .map(|raw_data| {
                NeuronExplainerPage::from_binary(raw_data.as_slice()).with_context(|| {
                    format!(
                        "Failed to deserialize neuron explainer neuron data for neuron \
                         l{layer_index}n{neuron_index} in model '{model_name}'."
                    )
                })
            })
            .transpose()
    }
}
