#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub refresh_token: Option<String>,
    pub uploaded_files: Vec<String>,
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
    serde_json::from_reader(file).map_err(GetError::SerdeError)
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
        uploaded_files: Vec::with_capacity(0),
    }
}
pub fn get_or_create(path: &str) -> Result<Config, GetError> {
    match get(path) {
        Ok(config) => return Ok(config),
        Err(GetError::NotFound) => Ok(create()),
        e => e,
    }
}