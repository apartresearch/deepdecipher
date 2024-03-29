use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use regex::Regex;
use tokio::fs;

use crate::data::{
    data_objects::{DataObject, Graph},
    data_types::DataType,
    DataTypeHandle, ModelHandle, NeuronIndex,
};

fn neuron_path(root: impl AsRef<Path>, neuron_index: NeuronIndex) -> PathBuf {
    let NeuronIndex { layer, neuron } = neuron_index;
    root.as_ref()
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
    let graph: Graph = match fs::read_to_string(path).await {
        Ok(graph_string) => {
            let regex = Regex::new(r#"\[label=([^"]\S*)"#)
                .context("Failed to compile regex. This should never happen.")?;
            let graph_str = regex.replace_all(graph_string.as_str(), r#"[label="$1""#);
            let graph = match graphviz_rust::parse(graph_str.as_ref()) {
                Ok(graph) => graph,
                Err(parse_error) => {
                    bail!(
                        "Failed to parse graph for neuron {neuron_index} in model '{}'. Error: \
                         '{parse_error}'",
                        model_handle.name()
                    )
                }
            };
            Graph::from_dot(graph).with_context(|| {
                format!(
                    "Succesfully parsed graph, but graph is not a valid neuron2graph grpah. \
                     Neuron {neuron_index} in model '{}'.",
                    model_handle.name()
                )
            })?
        }
        Err(read_err) => {
            if read_err.kind() == std::io::ErrorKind::NotFound {
                return Ok(false);
            } else {
                return Err(read_err).with_context(|| {
                    format!(
                        "Failed to read neuron2graph graph file for neuron {neuron_index} in \
                         model '{}'.",
                        model_handle.name()
                    )
                });
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
    let path = path.as_ref();

    if !path.is_dir() {
        bail!("Path '{}' is not a directory.", path.display())
    }

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
        if !retrieve_neuron2graph_neuron(model_handle, &data_type, path, neuron_index).await? {
            num_missing += 1
        }

        print!(
            "\rStoring neuron graphs: {}/{num_total_neurons}",
            neuron_index.flat_index(layer_size)
        );
    }
    println!(
        "\rStored all {num_total_neurons} neuron graphs. {num_missing} were missing.              "
    );
    Ok(())
}
