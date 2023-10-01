use std::path::{Path, PathBuf};

use crate::data::{
    data_types::DataType, neuron2graph::Graph, DataTypeHandle, ModelHandle, NeuronIndex,
};

use anyhow::{bail, Context, Result};
use tokio::fs;

fn neuron_path(root: impl AsRef<Path>, neuron_index: NeuronIndex) -> PathBuf {
    let NeuronIndex { layer, neuron } = neuron_index;
    root.as_ref()
        .join(format!("layer_{layer}"))
        .join(format!("{layer}_{neuron}"))
        .join("graph")
}

async fn retrieve_neuron2graph_neuron(
    model_handle: &mut ModelHandle,
    data_type: &DataTypeHandle,
    root: impl AsRef<Path>,
    neuron_index: NeuronIndex,
) -> Result<bool> {
    let path = neuron_path(root, neuron_index);
    let graph = match fs::read_to_string(path).await.map(|graph| Graph { graph }) {
        Ok(graph) => graph,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                return Ok(false);
            } else {
                return Err(err)
                    .with_context(|| format!("Failed to read neuron2graph graph file for neuron {neuron_index} in model '{}'.", model_handle.name()));
            }
        }
    };
    model_handle
        .add_neuron_data(
            data_type,
            neuron_index.layer,
            neuron_index.neuron,
            graph.to_binary()?,
        )
        .await
        .map(|_| true)
}

pub async fn retrieve_neuron2graph(
    model_handle: &mut ModelHandle,
    path: impl AsRef<Path>,
) -> Result<()> {
    let database = model_handle.database();

    let data_type = if let Some(data_type) = database.data_type("neuron2graph").await? {
        data_type
    } else {
        database
            .add_data_type("neuron2graph", DataType::Neuron2Graph)
            .await?
    };

    if model_handle.has_data_type(&data_type).await? {
        bail!(
            "Model '{}' already has a neuron2graph data object.",
            model_handle.name()
        )
    } else {
        model_handle
            .add_data_type(&data_type)
            .await
            .with_context(|| {
                format!(
                    "Failed to add neuron2graph data object to model '{}'.",
                    model_handle.name(),
                )
            })?
    }

    let num_total_neurons = model_handle.metadata().num_total_neurons;
    let layer_size = model_handle.metadata().layer_size;
    println!(
        "Retrieving neuron2graph data for model '{}'...",
        model_handle.name()
    );
    print!("Storing neuron graphs: 0/{num_total_neurons}");
    let mut num_missing = 0;
    for neuron_index in model_handle.metadata().neuron_indices() {
        if !retrieve_neuron2graph_neuron(model_handle, &data_type, path.as_ref(), neuron_index)
            .await?
        {
            num_missing += 1
        }

        print!(
            "\rStoring neuron graphs: {}/{num_total_neurons}",
            neuron_index.flat_index(layer_size)
        );
    }
    println!("\rStored all {num_total_neurons} neuron graphs. {num_missing} were missing.                   ");
    Ok(())
}
