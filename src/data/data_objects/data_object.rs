use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{Metadata, Neuroscope};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DataDescriptor {
    Neuroscope,
}

impl DataDescriptor {
    pub fn load(
        self,
        metadata: &Metadata,
        model_path: impl AsRef<Path>,
    ) -> Box<dyn DataObjectTrait> {
        match self {
            DataDescriptor::Neuroscope => Box::new(Neuroscope::new(metadata, model_path)),
        }
    }
}

pub trait DataObjectTrait: 'static + Send + Sync {}

pub enum DataObject {
    Neuroscope,
    /*Neuron2Graph,
    NeuronStore,*/
}
