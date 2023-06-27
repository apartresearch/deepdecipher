use std::path::{Path, PathBuf};

use anyhow::{ensure, Result};

use crate::data::{NeuroscopeLayerPage, NeuroscopeModelPage, NeuroscopeNeuronPage};

use super::{data_object::DataObjectTrait, Metadata};

pub struct Neuroscope {
    path: PathBuf,
    model_data: Option<NeuroscopeModelPage>,
    layer_data: Vec<Option<NeuroscopeLayerPage>>,
    neuron_data: Vec<Vec<Option<NeuroscopeNeuronPage>>>,
}

impl Neuroscope {
    pub fn new(metadata: &Metadata, model_path: impl AsRef<Path>) -> Self {
        let path = model_path.as_ref().join("neuroscope");
        let model_data = None;
        let layer_data = vec![None; metadata.layers.len()];
        let neuron_data = metadata
            .layers
            .iter()
            .map(|layer| vec![None; layer.num_neurons as usize])
            .collect();
        Self {
            path,
            model_data,
            layer_data,
            neuron_data,
        }
    }

    pub fn model_page(&mut self) -> Result<Option<&NeuroscopeModelPage>> {
        if self.model_data.is_none() {
            let path = self.path.join("model.postcard");
            self.model_data = Some(NeuroscopeModelPage::from_file(path)?);
        }
        Ok(self.model_data.as_ref())
    }

    pub fn layer_page(&mut self, layer_index: u32) -> Result<Option<&NeuroscopeLayerPage>> {
        ensure!(
            layer_index < self.layer_data.len() as u32,
            "Invalid layer index. Layer index is {layer_index}, but model only has {} layers.",
            self.layer_data.len()
        );
        if self.layer_data[layer_index as usize].is_none() {
            let path = self.path.join(format!("l{layer_index}.postcard"));
            self.layer_data[layer_index as usize] = Some(NeuroscopeLayerPage::from_file(path)?);
        }
        Ok(self.layer_data[layer_index as usize].as_ref())
    }

    pub fn neuron_page(
        &mut self,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<Option<&NeuroscopeNeuronPage>> {
        ensure!(
            layer_index < self.neuron_data.len() as u32,
            "Invalid layer index. Layer index is {layer_index}, but model only has {} layers.",
            self.neuron_data.len()
        );
        ensure!(
            neuron_index < self.neuron_data[layer_index as usize].len() as u32,
            "Invalid neuron index. Neuron index is {neuron_index}, but layer only has {} neurons.",
            self.neuron_data[layer_index as usize].len()
        );
        if self.neuron_data[layer_index as usize][neuron_index as usize].is_none() {
            let path = self
                .path
                .join(format!("l{layer_index}n{neuron_index}.postcard"));
            self.neuron_data[layer_index as usize][neuron_index as usize] =
                Some(NeuroscopeNeuronPage::from_file(path)?);
        }
        Ok(self.neuron_data[layer_index as usize][neuron_index as usize].as_ref())
    }
}

impl DataObjectTrait for Neuroscope {}
