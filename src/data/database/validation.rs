use anyhow::{bail, Context};

use crate::{data::NeuronIndex, Index};

use super::{DataObjectHandle, ModelHandle};

impl ModelHandle {
    pub async fn missing_model_items(
        &self,
        data_object: &DataObjectHandle,
    ) -> anyhow::Result<impl ExactSizeIterator<Item = Index>> {
        const COUNT_MODEL_DATA: &str = r#"
        SELECT
            COUNT(*),
        FROM model_data
        WHERE model_id = ?1 AND data_object_id = ?2;
        "#;

        let params = (self.id(), data_object.id());

        let num_rows: u32 = self
            .database()
            .connection
            .call(move |connection| {
                let mut statement = connection.prepare(COUNT_MODEL_DATA)?;
                statement.query_row(params, |row| row.get(0))
            })
            .await
            .with_context(|| {
                format!(
                    "Failed to count model data for data object '{}' for model '{}'.",
                    self.name(),
                    data_object.name()
                )
            })?;
        match num_rows {
            0 => Ok(vec![Index::Model].into_iter()),
            1 => Ok(vec![].into_iter()),
            _ => bail!(
                "Model '{}' has multiple data objects named '{}'. This should be impossible.",
                self.name(),
                data_object.name()
            ),
        }
    }

    pub async fn missing_layer_items(
        &self,
        data_object: &DataObjectHandle,
    ) -> anyhow::Result<impl Iterator<Item = Index>> {
        const COUNT_LAYER_DATA: &str = r#"
        SELECT
            layer_index,
        FROM layer_data
        WHERE model_id = ?1 AND data_object_id = ?2;
        "#;

        let params = (self.id(), data_object.id());

        let existing_layers = self
            .database()
            .connection
            .call(move |connection| {
                connection
                    .prepare(COUNT_LAYER_DATA)?
                    .query_map(params, |row| row.get::<_, usize>(0))?
                    .collect::<Result<Vec<_>, _>>()
            })
            .await
            .with_context(|| {
                format!(
                    "Failed to count layer data for data object '{}' for model '{}'.",
                    self.name(),
                    data_object.name()
                )
            })?;
        let mut layer_item_exists = vec![false; self.metadata().num_layers as usize];
        for layer_index in existing_layers {
            layer_item_exists[layer_index] = true;
        }
        Ok(layer_item_exists
            .into_iter()
            .enumerate()
            .filter_map(|(index, exists)| {
                if exists {
                    None
                } else {
                    Some(Index::Layer(index as u32))
                }
            }))
    }

    pub async fn missing_neuron_items(
        &self,
        data_object: &DataObjectHandle,
    ) -> anyhow::Result<impl Iterator<Item = Index>> {
        const COUNT_NEURON_DATA: &str = r#"
        SELECT
            layer_index,
            neuron_index,
        FROM neuron_data
        WHERE model_id = ?1 AND data_object_id = ?2;
        "#;

        let layer_size = self.metadata().layer_size;

        let params = (self.id(), data_object.id());

        let existing_neurons = self
            .database()
            .connection
            .call(move |connection| {
                connection
                    .prepare(COUNT_NEURON_DATA)?
                    .query_map(params, |row| {
                        let layer_index: u32 = row.get(0)?;
                        let neuron_index: u32 = row.get(1)?;
                        Ok(NeuronIndex {
                            layer: layer_index,
                            neuron: neuron_index,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()
            })
            .await
            .with_context(|| {
                format!(
                    "Failed to count neuron data for data object '{}' for model '{}'.",
                    self.name(),
                    data_object.name()
                )
            })?;
        let mut neuron_item_exists = vec![false; self.metadata().num_total_neurons as usize];
        for neuron_index in existing_neurons {
            neuron_item_exists[neuron_index.flat_index(layer_size)] = true;
        }
        Ok(neuron_item_exists.into_iter().enumerate().flat_map(
            move |(flat_neuron_index, exists)| {
                if exists {
                    None
                } else {
                    Some(Index::from_flat_neuron_index(layer_size, flat_neuron_index))
                }
            },
        ))
    }

    pub async fn missing_items(
        &self,
        data_object: &DataObjectHandle,
    ) -> anyhow::Result<impl Iterator<Item = Index>> {
        Ok(self
            .missing_model_items(data_object)
            .await?
            .chain(self.missing_layer_items(data_object).await?)
            .chain(self.missing_neuron_items(data_object).await?))
    }
}
