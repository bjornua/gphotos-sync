use clap::{App, ArgMatches, SubCommand};
// use std::path;
use crate::config;

pub fn get_subcommand() -> App<'static, 'static> {
    SubCommand::with_name("authenticate").about("Authenticate with Google")
}

pub async fn main(_matches: &ArgMatches<'_>) {
    let config = match config::get_or_create("./gphotos-sync.cbor") {
        Ok(config) => config,
        Err(e) => {
            println!("Configuration file error: {:?}", e);
            return;
        }
    };
    if config.credentials.is_some() {
        println!("Already authenticated.");
        return;
    }

    println!("Opening browser. Follow the instructions to authenticate gphotos-sync.");
    let credentials = match crate::gauth::oauth().await {
        Ok(credentials) => credentials,
        Err(error) => {
            println!("{:?}", error);
            return;
        }
    };

    let new_config = config::Config {
        credentials: Some(credentials),
        ..config
    };
    match config::save("./gphotos-sync.cbor", &new_config) {
        Ok(()) => (),
        Err(error) => {
            println!("{:?}", error);
            return;
        }
    };
    println!("Done authenticating. You can now run `gphotos-sync upload`");
}
