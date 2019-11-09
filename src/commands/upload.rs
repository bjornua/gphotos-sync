use crate::config;
use clap::{App, Arg, ArgMatches, SubCommand};
use std::io::Read;

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
enum ReadFileError {
    OpenError(std::io::Error),
    ReadError(std::io::Error),
}
fn read_file<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<u8>, ReadFileError> {
    let mut content = Vec::<u8>::new();
    std::fs::File::open(path.as_ref())
        .map_err(ReadFileError::OpenError)?
        .read_to_end(&mut content)
        .map_err(ReadFileError::ReadError)?;
    return Ok(content);
}

fn read_and_hash_file<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<(Vec<u8>, config::HashDigest), ReadFileError> {
    let content = crate::utils::slowlog(50, "read file", || read_file(path))?;
    let hash = crate::utils::slowlog(50, "hashed file", || config::sha224str(&content));
    Ok((content, hash))
}

pub async fn main(matches: &ArgMatches<'_>) {
    let mut cfg = match config::get_or_create("./sd-card-uploader.json") {
        Ok(cfg) => cfg,
        Err(e) => {
            println!("Error reading configuration file: {:?}", e);
            return;
        }
    };
    let _refresh_token = match &cfg.refresh_token {
        Some(t) => t,
        None => {
            println!("You are not authenticated. Please run `sd-card-uploader authenticate`.");
            return;
        }
    };
    let directory = matches.value_of_os("DIRECTORY").unwrap().to_os_string();

    let files = crate::iterdir::findfiles(directory, EXTENSIONS)
        .filter_map(Result::ok)
        .map(|m| m.dir_entry.path());
    // Change this to hash as we read the file
    for f in files {
        let (_contents, hash) = match read_and_hash_file(&f) {
            Ok(r) => r,
            Err(err) => {
                println!(
                    "An error happened while processing file: {:?}: {:?}",
                    f, err
                );
                return;
            }
        };
        if cfg.uploaded_files.contains(&hash) {
            continue;
        }
        println!("Uploading file: {:?}", f);

        cfg.uploaded_files.insert(hash);
    }
    match config::save("./sd-card-uploader.json", &cfg) {
        Ok(()) => (),
        Err(e) => {
            println!("Error saving configuration file: {:?}", e);
            return;
        }
    };
}
