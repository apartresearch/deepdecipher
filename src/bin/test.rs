use flate2::{Compress, Compression};
use rand::Rng;

pub fn main() {
    let mut data: Vec<u8> = vec![0; 10000];
    let mut rng = rand::thread_rng();
    rng.fill(data.as_mut_slice());

    let mut compressor = Compress::new(Compression::default(), false);
    let mut compressed_data = Vec::with_capacity(100000);

    match compressor.compress_vec(
        data.as_slice(),
        &mut compressed_data,
        flate2::FlushCompress::None,
    ) {
        Ok(status) => println!("Compressed data: {status:?}"),
        Err(e) => println!("Error: {:?}", e),
    }
}
