use fasthash::{metro::crc::Hasher64_1, FastHasher};
use std::hash::Hasher;
use std::io::Read;

pub type HashDigest = u64;
pub type Hashes = std::collections::BTreeSet<HashDigest>;

#[derive(Debug)]
pub enum HashFileError {
    OpenError(std::io::Error),
    ReadError(std::io::Error),
}

pub fn hash_file<P: AsRef<std::path::Path>>(path: P) -> Result<(usize, HashDigest), HashFileError> {
    let mut file = std::fs::File::open(path.as_ref()).map_err(HashFileError::OpenError)?;
    let mut buffer = [0; 65536];
    let mut read_bytes_total = 0;
    let mut hasher = Hasher64_1::new();
    loop {
        let read_bytes = file.read(&mut buffer).map_err(HashFileError::ReadError)?;
        read_bytes_total += read_bytes;
        if read_bytes == 0 {
            return Ok((read_bytes_total, hasher.finish()));
        };
        hasher.write(&buffer[0..read_bytes]);
    }
}
