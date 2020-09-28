use crate::lib::config;
use crate::lib::gauth;

use clap::{App, ArgMatches, SubCommand};

pub fn get_subcommand() -> App<'static, 'static> {
    SubCommand::with_name("authenticate").about("Authenticate with Google")
}

pub async fn command(_matches: &ArgMatches<'_>) {
    println!("Opening browser. Follow the instructions to authenticate gphotos-sync.");
    let credentials = match gauth::oauth().await {
        Ok(credentials) => credentials,
        Err(error) => {
            println!("{:?}", error);
            return;
        }
    };

    let new_config = config::create(credentials);
    match config::save("./gphotos-sync.cbor", &new_config) {
        Ok(()) => (),
        Err(error) => {
            println!("{:?}", error);
            return;
        }
    };
    println!("Done authenticating. You can now run `gphotos-sync upload`");
}
