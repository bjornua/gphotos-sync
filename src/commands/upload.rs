use crate::config;
use crate::upload;
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

#[derive(Debug)]
enum MainError {
    LoadConfig(config::LoadError),
    UploadError(upload::UploadError),
    SaveConfig(config::SaveError),
}

pub async fn main(matches: &ArgMatches<'_>) {
    if let Err(e) = main_inner(matches).await {
        println!("Error: {:?}", e);
    };
}

async fn main_inner(matches: &ArgMatches<'_>) -> Result<(), MainError> {
    let directory = matches.value_of_os("DIRECTORY").unwrap().to_os_string();

    let mut cfg = config::load("./gphotos-sync.cbor").map_err(MainError::LoadConfig)?;

    let files = crate::iterdir::findfiles_with_ext(directory).filter_map(Result::ok);

    upload::upload_many(&mut cfg.credentials, &mut cfg.uploaded_files, files)
        .await
        .map_err(MainError::UploadError)?;

    config::save("./gphotos-sync.cbor", &cfg).map_err(MainError::SaveConfig)?;
    return Ok(());
}
