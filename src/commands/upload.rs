use crate::config;
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

pub fn main(matches: &ArgMatches) {
    let config = match config::get_or_create("./sd-card-uploader.json") {
        Ok(config) => config,
        Err(e) => {
            println!("Configuration file error: {:?}", e);
            return;
        }
    };
    let refresh_token = match config.refresh_token {
        Some(t) => t,
        None => {
            println!("Refresh token not found");
            return;
        }
    };

    let directory = matches.value_of_os("DIRECTORY").unwrap().to_os_string();

    let files = crate::iterdir::findfiles(directory, EXTENSIONS)
        .filter_map(Result::ok)
        .map(|m| m.dir_entry.path());
    println!("{:#?}", files.collect::<Vec<_>>())
}
