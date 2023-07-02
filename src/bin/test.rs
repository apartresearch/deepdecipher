use anyhow::Result;
use flate2::{Compress, Compression};
use neuronav::data::{database::Database, NeuroscopeNeuronPage};
use rand::Rng;
use snap::raw::Encoder;

pub fn main() -> Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(8)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        let database = Database::open("data.db").await?;
        database.delete_model("gelu-1l".to_owned()).await?;
        Ok(())
    })

    /*let start = std::time::Instant::now();
    let page = NeuroscopeNeuronPage::from_file("data/solu-1l/neuroscope/l0n0.postcard").unwrap();
    println!("Load time: {:?}", start.elapsed());
    let start = std::time::Instant::now();
    let data = postcard::to_allocvec(&page)?;
    println!("Serialize time: {:?}", start.elapsed());
    println!("Raw data size: {}", data.len());
    let start = std::time::Instant::now();
    let snap = Encoder::new().compress_vec(data.as_slice())?;
    println!("Snap time: {:?}", start.elapsed());
    println!("Snap data size: {}", snap.len());

    let level = 3;
    let start = std::time::Instant::now();
    let zstd = zstd::bulk::compress(data.as_slice(), level)?;
    println!("zstd({level}) time: {:?}", start.elapsed());
    println!("zstd({level}) data size: {}", zstd.len());

    let start = std::time::Instant::now();
    let decoded_page = postcard::from_bytes::<NeuroscopeNeuronPage>(data.as_slice())?;
    println!("Deserialize time: {:?}", start.elapsed());

    let start = std::time::Instant::now();
    let unsnap = snap::raw::Decoder::new().decompress_vec(snap.as_slice())?;
    println!("Unsnap time: {:?}", start.elapsed());

    let start = std::time::Instant::now();
    let unzstd = zstd::stream::decode_all(zstd.as_slice())?;
    println!("Unzstd time: {:?}", start.elapsed());
    Ok(())*/
}
