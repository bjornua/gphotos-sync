use arrayvec;
// use sha2::Digest;
use std::hash::Hasher;
use fasthash::{metro::crc::Hasher128_1, FastHasher, HasherExt};

pub type HashDigest = arrayvec::ArrayString<[u8; 16]>;
pub fn sha224str(s: &[u8]) -> HashDigest {
    // let hash_bytes = sha2::Sha224::digest(s);
    let mut hasher = Hasher128_1::new();
    hasher.write(s);
    let hash = hasher.finish_ext();
    // return fasthash::metro::Hash128_2::new()(&hex::encode(&hash_bytes[0..8])).unwrap();
    arrayvec::ArrayString::from(&hex::encode(&hash.to_ne_bytes())[0..16]).unwrap()
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub refresh_token: Option<String>,
    pub uploaded_files: std::collections::BTreeSet<HashDigest>,
}

#[derive(Debug)]
pub enum GetError {
    OpenError(std::io::Error),
    SerdeError(serde_json::Error),
    NotFound,
}

fn get<P: AsRef<std::path::Path>>(path: P) -> Result<Config, GetError> {
    let file_result = std::fs::File::open(path);
    let file = file_result.map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => GetError::NotFound,
        _ => GetError::OpenError(e),
    })?;
    crate::utils::slowlog(0, "Serde parse", || {
        serde_json::from_reader(file).map_err(GetError::SerdeError)
    })
}

#[derive(Debug)]
pub enum SaveError {
    OpenFileError(std::io::Error),
    SerdeError(serde_json::Error),
}

pub fn save<P: AsRef<std::path::Path>>(path: P, config: &Config) -> Result<(), SaveError> {
    let file = std::fs::File::create(path).map_err(SaveError::OpenFileError)?;
    serde_json::to_writer_pretty(file, config).map_err(SaveError::SerdeError)?;
    return Ok(());
}
fn create() -> Config {
    Config {
        refresh_token: None,
        uploaded_files: std::collections::BTreeSet::new(),
    }
}
pub fn get_or_create(path: &str) -> Result<Config, GetError> {
    match get(path) {
        Ok(config) => return Ok(config),
        Err(GetError::NotFound) => Ok(create()),
        e => e,
    }
}
