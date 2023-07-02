use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use neuronav::data::{
    data_types::DataType, database::Database, NeuroscopeLayerPage, NeuroscopeModelPage,
    NeuroscopeNeuronPage,
};

const DATA_PATH: &str = "data.db";

fn main() -> Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(8)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        let database_path = PathBuf::from(DATA_PATH);
        let database = if database_path.exists() {
            fs::remove_file(database_path)?;
            Database::initialize(DATA_PATH).await?
        } else {
            Database::initialize(DATA_PATH).await?
        };

        let model_name = "solu-1l";
        let model_path = PathBuf::from("data").join(model_name);

        let model = if let Some(model) = database.model(model_name.to_owned()).await? {
            model
        } else {
            let path: std::path::PathBuf = model_path.join("metadata.json");
            let text = fs::read_to_string(path).context("Failed to read metadata")?;
            let metadata = serde_json::from_str(&text)?;
            database.add_model(metadata).await?
        };

        database
            .add_data_object("neuroscope", DataType::Neuroscope)
            .await?;
        model.add_data_object(&database, "neuroscope").await?;

        let neuroscope_path = model_path.join("neuroscope");
        print!("Loading model page...\r");
        let model_page = NeuroscopeModelPage::from_file(neuroscope_path.join("model.postcard"))?;
        print!("Model page loaded. Writing to database...\r");
        model
            .add_model_data(&database, "neuroscope", model_page.to_binary()?)
            .await?;
        println!("Model page written to database.                 ");

        let mut load_time = Duration::from_secs(0);
        let mut write_time = Duration::from_secs(0);
        print!("Writing layers and neuron pages to database. Layer 0/8. Neuron 0/2048.\r");
        for layer_index in 0..1 {
            let layer_page = NeuroscopeLayerPage::from_file(
                neuroscope_path.join(format!("l{layer_index}.postcard")),
            )?;
            model
                .add_layer_data(
                    &database,
                    "neuroscope",
                    layer_index,
                    layer_page.to_binary()?,
                )
                .await?;
            print!("Writing layers and neuron pages to database. Layer {}/8. Neuron 0/2048.                                                                               \r", layer_index+1);

            for neuron_index in 0..256 {
                let start_time = Instant::now();
                let neuron_page = NeuroscopeNeuronPage::from_file(neuroscope_path.join(format!(
                    "l{layer_index}n{neuron_index}.postcard",
                )))?;
                load_time += start_time.elapsed();

                let start_time = Instant::now();
                model
                    .add_neuron_data(
                        &database,
                        "neuroscope",
                        layer_index,
                        neuron_index,
                        neuron_page.to_binary()?,
                    )
                    .await?;
                write_time += start_time.elapsed();
                let avg_load_time = load_time.as_secs_f32()/((layer_index*2048+neuron_index+1) as f32);
                let avg_write_time = write_time.as_secs_f32()/((layer_index*2048+neuron_index+1) as f32);
                print!("Writing layers and neuron pages to database. Layer {}/8. Neuron {}/2048. Load time avg: {avg_load_time}. Write time avg: {avg_write_time}    \r", layer_index, neuron_index+1);
            }
        }

        println!("Written all layer and neuron pages to database.                                                                          ");

        Ok(())
    })
}
