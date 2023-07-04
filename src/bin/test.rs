use anyhow::Result;
use neuronav::data::NeuroscopeNeuronPage;
use snap::raw::Encoder;

pub fn main() -> Result<()> {
    let start = std::time::Instant::now();
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
    let _decoded_page = postcard::from_bytes::<NeuroscopeNeuronPage>(data.as_slice())?;
    println!("Deserialize time: {:?}", start.elapsed());

    let start = std::time::Instant::now();
    let _unsnap = snap::raw::Decoder::new().decompress_vec(snap.as_slice())?;
    println!("Unsnap time: {:?}", start.elapsed());

    let start = std::time::Instant::now();
    let _unzstd = zstd::stream::decode_all(zstd.as_slice())?;
    println!("Unzstd time: {:?}", start.elapsed());
    Ok(())
}
