use crate::lib::gauth::Credentials;
use crate::lib::hash::Hashes;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub credentials: Credentials,
    pub uploaded_files: Hashes,
}

#[derive(Debug)]
pub enum LoadError {
    OpenError(std::io::Error),
    SerdeError(serde_cbor::Error),
    NotFound(std::path::PathBuf),
}

pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Config, LoadError> {
    let path = path.as_ref();
    let file_result = std::fs::File::open(path);
    let file = file_result.map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => LoadError::NotFound(path.to_path_buf()),
        _ => LoadError::OpenError(e),
    })?;
    serde_cbor::from_reader(file).map_err(LoadError::SerdeError)
}

#[derive(Debug)]
pub enum SaveError {
    OpenFileError(std::io::Error),
    SerdeError(serde_cbor::Error),
}

pub fn save<P: AsRef<std::path::Path>>(path: P, config: &Config) -> Result<(), SaveError> {
    let file = std::fs::File::create(path).map_err(SaveError::OpenFileError)?;
    serde_cbor::to_writer(file, config).map_err(SaveError::SerdeError)?;
    return Ok(());
}
pub fn create(credentials: Credentials) -> Config {
    Config {
        credentials,
        uploaded_files: Hashes::new(),
    }
}
