#![warn(unused_extern_crates)]

use clap::{App, AppSettings};

mod commands;
mod lib;

#[tokio::main]
async fn main() -> () {
    let app = App::new("sd-photo-uploader")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version("0.1")
        .author("Bjørn Arnholtz. <bjorn.arnholtz@gmail.com>")
        .about("Uploads photos from a certain directory to google photos")
        .subcommand(commands::authenticate::get_subcommand())
        .subcommand(commands::upload::get_subcommand())
        .subcommand(commands::watch::get_subcommand());
    let matches = app.get_matches();

    match matches.subcommand() {
        ("upload", Some(args)) => commands::upload::command(args).await,
        ("authenticate", Some(args)) => commands::authenticate::command(args).await,
        ("watch", Some(args)) => commands::watch::command(args).await,
        (_, _) => unimplemented!(),
    }
}
