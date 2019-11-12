use crate::config;
use clap::{App, Arg, ArgMatches, SubCommand};
use fasthash::{metro::crc::Hasher64_1, FastHasher};
use std::hash::Hasher;
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

fn hash_file<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<(usize, config::HashDigest), ReadFileError> {
    let mut file = std::fs::File::open(path.as_ref()).map_err(ReadFileError::OpenError)?;
    let mut buffer = [0; 65536];
    // 128KB buffer
    // let mut buffer = [0; 131_072];
    let mut read_bytes_total = 0;
    let mut hasher = Hasher64_1::new();
    loop {
        let read_bytes = file.read(&mut buffer).map_err(ReadFileError::ReadError)?;
        read_bytes_total += read_bytes;
        if read_bytes == 0 {
            return Ok((read_bytes_total, hasher.finish()));
        };
        hasher.write(&buffer[0..read_bytes]);
    }
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
    let mut files_skipped = 0;
    let mut files_skipped_size = 0;
    let mut files_skipped_duration = std::time::Duration::new(0, 0);
    for f in files {
        let time_begin = std::time::Instant::now();
        let (hash_file_size, hash) = match hash_file(&f) {
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
            files_skipped_duration += time_begin.elapsed();
            files_skipped_size += hash_file_size;
            files_skipped += 1;
            continue;
        }
        println!("Uploading file: {:?}", f);

        cfg.uploaded_files.insert(hash);
    }
    println!(
        "Skipped {:} files. Skipped total size: {:.2} MB. Speed: {:.2} MB/s",
        files_skipped,
        (files_skipped_size as f64) / 1_000_000f64,
        (files_skipped_size as f64 / 1_000_000f64) / files_skipped_duration.as_secs_f64()
    );
    match config::save("./sd-card-uploader.json", &cfg) {
        Ok(()) => (),
        Err(e) => {
            println!("Error saving configuration file: {:?}", e);
            return;
        }
    };
}
