use async_trait::async_trait;

use anyhow::Result;
use strum::EnumString;

use crate::data::database::Database;

use super::Neuroscope;

#[async_trait]
pub trait DataTypeTrait: Clone + Send + Sync {
    fn name(&self) -> String;

    async fn model_data(&self, database: &Database, model_name: &str) -> Result<serde_json::Value>;
}

#[derive(Clone, EnumString)]
pub enum DataType {
    Neuroscope,
}

impl DataType {
    pub fn neuroscope(&self) -> Neuroscope {
        Neuroscope
    }
}
