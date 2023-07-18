use crate::data::{
    data_types::DataType, neuron_store::NeuronStoreRaw, DataObjectHandle, Database, ModelHandle,
    NeuronStore,
};

use anyhow::{bail, Context, Result};

pub async fn store_similar_neurons(
    model_handle: &mut ModelHandle,
    data_object_handle: &DataObjectHandle,
    neuron_store: NeuronStore,
    threshold: f32,
) -> Result<()> {
    let model_name = model_handle.name().to_owned();
    let model_name = model_name.as_str();
    for neuron_index in model_handle.metadata().neuron_indices() {
        let similar_neurons = neuron_store.similar_neurons(neuron_index, threshold);
        let data = postcard::to_allocvec(similar_neurons.as_slice()).with_context(|| format!("Failed to serialize similar neuron vector for neuron {neuron_index} in model {model_name}."))?;
        model_handle.add_neuron_data(data_object_handle, neuron_index.layer, neuron_index.neuron, data).await.with_context(||format!("Failed to add similar neuron vector for neuron {neuron_index} in model {model_name} to database."))?;
    }
    Ok(())
}

pub async fn store_neuron_store(
    database: &Database,
    model_handle: &mut ModelHandle,
    neuron_store: NeuronStore,
    similarity_threshold: f32,
) -> Result<()> {
    let data_object = if let Some(data_object) = database.data_object("neuron_store").await? {
        data_object
    } else {
        database
            .add_data_object(
                "neuron_store",
                DataType::NeuronStore {
                    similarity_threshold,
                },
            )
            .await?
    };

    let model_name = model_handle.name().to_owned();
    let model_name = model_name.as_str();

    if model_handle.has_data_object(&data_object).await? {
        bail!("Model '{model_name}' already has a neuron store data object.",)
    } else {
        model_handle
            .add_data_object(&data_object)
            .await
            .context("Failed to add neuron store data object to model.")?
    }

    model_handle
        .add_model_data(
            &data_object,
            postcard::to_allocvec(&neuron_store).with_context(|| {
                format!("Failed to serialize neuron store for model '{model_name}'.",)
            })?,
        )
        .await
        .with_context(|| {
            format!("Failed to add neuron store data for model '{model_name}' to database.",)
        })?;

    Ok(())
}

pub async fn store_raw_neuron_store(
    database: &Database,
    model_handle: &mut ModelHandle,
    neuron_store: NeuronStoreRaw,
    similarity_threshold: f32,
) -> Result<()> {
    store_neuron_store(
        database,
        model_handle,
        NeuronStore::from_raw(neuron_store)?,
        similarity_threshold,
    )
    .await
}
