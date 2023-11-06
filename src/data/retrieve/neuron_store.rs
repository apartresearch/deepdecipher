use std::path::Path;

use crate::data::{
    data_types::DataType, neuron_store::NeuronStoreRaw, DataTypeHandle, Database, ModelHandle,
    NeuronStore,
};

use anyhow::{bail, Context, Result};

pub async fn store_similar_neurons(
    model_handle: &mut ModelHandle,
    data_type_handle: &DataTypeHandle,
    neuron_store: NeuronStore,
    similarity_threshold: f32,
) -> Result<()> {
    let model_name = model_handle.name().to_owned();
    let model_name = model_name.as_str();
    print!("Calculating neuron similarities...");
    let neuron_relatedness = neuron_store.neuron_similarity();
    println!("\rNeuron similarities calculated.    ");

    let num_neurons = model_handle.metadata().num_total_neurons;
    let mut num_completed = 0;
    print!("Adding neuron similarities to database: {num_completed}/{num_neurons}",);
    for neuron_index in model_handle.metadata().neuron_indices() {
        let similar_neurons =
            neuron_relatedness.similar_neurons(neuron_index, similarity_threshold);
        let data = similar_neurons.to_binary().with_context(|| 
            format!("Failed to serialize similar neuron vector for neuron {neuron_index} in model {model_name}.")
        )?;
        model_handle.add_neuron_data(data_type_handle, neuron_index.layer, neuron_index.neuron, data).await.with_context(||
            format!("Failed to add similar neuron vector for neuron {neuron_index} in model {model_name} to database.")
        )?;

        num_completed += 1;
        print!("\rAdding neuron similarities to database: {num_completed}/{num_neurons}",);
    }
    println!("\rAdding neuron similarities to database: {num_completed}/{num_neurons}                     ",);
    Ok(())
}

pub async fn store_neuron_store(
    database: &Database,
    model_handle: &mut ModelHandle,
    neuron_store: NeuronStore,
    similarity_threshold: f32,
) -> Result<()> {
    let data_type = if let Some(data_type) = database.data_type("neuron_store").await? {
        data_type
    } else {
        database
            .add_data_type(
                "neuron_store",
                DataType::NeuronStore {
                    similarity_threshold,
                },
            )
            .await?
    };

    let model_name = model_handle.name().to_owned();
    let model_name = model_name.as_str();

    if model_handle.has_data_type(&data_type).await? {
        bail!("Model '{model_name}' already has a neuron store data object.",)
    } else {
        model_handle
            .add_data_type(&data_type)
            .await
            .context("Failed to add neuron store data object to model.")?
    }

    model_handle
        .add_model_data(&data_type, neuron_store.to_binary()?)
        .await
        .with_context(|| {
            format!("Failed to add neuron store data for model '{model_name}' to database.",)
        })?;

    store_similar_neurons(model_handle, &data_type, neuron_store, similarity_threshold)
        .await
        .context("Failed to store similar neurons.")
}

pub async fn store_raw_neuron_store(
    database: &Database,
    model_handle: &mut ModelHandle,
    neuron_store: NeuronStoreRaw,
    similarity_threshold: f32,
) -> Result<()> {
    let metadata = model_handle.metadata();
    let neuron_store =
        NeuronStore::from_raw(neuron_store, metadata.num_layers, metadata.layer_size)
            .context("Failed to convert raw neuron store")?;
    store_neuron_store(database, model_handle, neuron_store, similarity_threshold).await
}

pub async fn retrieve_neuron_store(
    model_handle: &mut ModelHandle,
    neuron_store_path: impl AsRef<Path>,
    similarity_threshold: f32,
) -> Result<()> {
    let neuron_store_path = neuron_store_path.as_ref();
    let neuron_store = NeuronStoreRaw::load(neuron_store_path)
        .with_context(|| format!("Failed to load neuron store from '{neuron_store_path:?}'."))?;
    let database = model_handle.database().clone();
    store_raw_neuron_store(&database, model_handle, neuron_store, similarity_threshold)
        .await
        .with_context(|| {
            format!(
                "Failed to store neuron store for model '{model_name}'.",
                model_name = model_handle.name()
            )
        })
}
