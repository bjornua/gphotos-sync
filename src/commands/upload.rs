use crate::config;
use crate::gphotos;
use crate::gauth;
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
    HashFileError(std::path::PathBuf, hash::HashFileError)
}

pub async fn main(matches: &ArgMatches<'_>)  {
    if let Err(e) = main_inner(matches).await {
        println!("Error: {:?}", e);
    };
}

 async fn main_inner(matches: &ArgMatches<'_>) -> Result<(), MainError> {
    let mut cfg = config::get_or_create("./gphotos-sync.cbor").map_err(MainError::ReadConfiguration)?;
    let mut credentials = (&cfg.credentials).clone().ok_or(MainError::NotAuthenticated)?.clone();
    let directory = matches.value_of_os("DIRECTORY").unwrap().to_os_string();

    let files = crate::iterdir::findfiles(directory, EXTENSIONS)
        .filter_map(Result::ok)
        .map(|m| m.dir_entry.path());

    let mut files_skipped: usize = 0;
    let mut files_skipped_size = 0;
    let mut files_skipped_duration = std::time::Duration::new(0, 0);
    for path in files {
        let time_begin = std::time::Instant::now();
        let (hash_file_size, hash) = hash::hash_file(&path).map_err(|e| MainError::HashFileError(path.clone(), e))?;
        if cfg.uploaded_files.contains(&hash) {
            files_skipped_duration += time_begin.elapsed();
            files_skipped_size += hash_file_size;
            files_skipped += 1;
            continue;
        }
        credentials = gauth::refresh_credentials_if_needed(credentials).await.map_err(MainError::RefreshCredentials)?;
        cfg.uploaded_files.insert(hash);
        match gphotos::upload_file(&credentials.access_token, &path).await {
            Ok(gphotos::UploadFileOk { upload_token }) => upload_token,
            Err(err) => {
                println!(
                    "An error happened while uploading file: {:?}: {:?}",
                    path, err
                );
                continue;
            }
        };
    }
    println!(
        "Skipped {:} files. Skipped total size: {:.2} MB. Speed: {:.2} MB/s",
        files_skipped,
        (files_skipped_size as f64) / 1_000_000f64,
        (files_skipped_size as f64 / 1_000_000f64) / files_skipped_duration.as_secs_f64()
    );

    match config::save("./gphotos-sync.cbor", &cfg) {
        Ok(()) => (),
        Err(e) => {
            println!("Error saving configuration file: {:?}", e);
            return Ok(());
        }
    };
    return Ok(())
}
