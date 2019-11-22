use crate::config;
use crate::gauth;
use crate::gphotos;
use crate::hash;
use clap::{App, Arg, ArgMatches, SubCommand};

pub fn get_subcommand() -> App<'static, 'static> {
    SubCommand::with_name("upload")
        .about("Upload photos to Google")
        .arg(
            Arg::with_name("DIRECTORY")
                .index(1)
                .required(true)
                .multiple(false),
        )
}

const EXTENSIONS: &'static [&'static str] = &["jpg", "JPG", "png", "PNG"];

#[derive(Debug)]
enum MainError {
    ReadConfiguration(config::GetError),
    NotAuthenticated,
    RefreshCredentials(gauth::RefreshCredentialsError),
    HashFile(std::path::PathBuf, hash::HashFileError),
    UploadFile(gphotos::UploadFileError),
    BatchCreate(gphotos::BatchCreateError),
    SaveConfig(config::SaveError),
}

pub async fn main(matches: &ArgMatches<'_>) {
    if let Err(e) = main_inner(matches).await {
        println!("Error: {:?}", e);
    };
}

async fn main_inner(matches: &ArgMatches<'_>) -> Result<(), MainError> {
    let mut cfg =
        config::get_or_create("./gphotos-sync.cbor").map_err(MainError::ReadConfiguration)?;
    let mut credentials = (&cfg.credentials)
        .clone()
        .ok_or(MainError::NotAuthenticated)?
        .clone();
    let directory = matches.value_of_os("DIRECTORY").unwrap().to_os_string();

    let files = crate::iterdir::findfiles(directory, EXTENSIONS)
        .filter_map(Result::ok)
        .map(|m| m.dir_entry.path());
    let mut upload_tokens: Vec<String> = Vec::new();
    for path in files {
        let (_hash_file_size, hash) =
            hash::hash_file(&path).map_err(|e| MainError::HashFile(path.clone(), e))?;
        if cfg.uploaded_files.contains(&hash) {
            continue;
        }
        credentials = gauth::refresh_credentials_if_needed(credentials)
            .await
            .map_err(MainError::RefreshCredentials)?;

        match gphotos::upload_file(&credentials.access_token, &path).await {
            Ok(t) => {
                upload_tokens.push(t);
            }
            Err(gphotos::UploadFileError::Duplicate) => {}
            Err(e) => {
                return Err(MainError::UploadFile(e));
            }
        };
        cfg.uploaded_files.insert(hash);
    }

    credentials = gauth::refresh_credentials_if_needed(credentials)
        .await
        .map_err(MainError::RefreshCredentials)?;

    dbg!(
        gphotos::batch_create(&credentials.access_token, upload_tokens)
            .await
            .map_err(MainError::BatchCreate)?
    );

    config::save("./gphotos-sync.cbor", &cfg).map_err(MainError::SaveConfig)?;
    return Ok(());
}
