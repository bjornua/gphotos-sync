extern crate arrayvec;
extern crate clap;
extern crate fasthash;
extern crate hex;
extern crate hyper;
extern crate open;
extern crate sha2;
extern crate url;
use clap::{App, AppSettings};

mod commands;
mod config;
mod gphotos;
mod iterdir;
mod utils;

use std::alloc::System;

// Don't use jemalloc as allocator. We optimize for binary size because
// this program doesn't do a lot of allocations
#[global_allocator]
static GLOBAL: System = System;

fn main() -> () {
    let app = App::new("sd-photo-uploader")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version("0.1")
        .author("Bjørn Arnholtz. <bjorn.arnholtz@gmail.com>")
        .about("Uploads photos from a certain directory to google photos")
        .subcommand(commands::authenticate::get_subcommand())
        .subcommand(commands::upload::get_subcommand());
    let matches = app.get_matches();

    let command_future = async {
        match matches.subcommand() {
            ("upload", Some(args)) => commands::upload::main(args).await,
            ("authenticate", Some(args)) => commands::authenticate::main(args).await,
            (_, _) => unimplemented!(),
        }
    };

    let response_code = tokio::runtime::current_thread::Runtime::new()
        .unwrap()
        .block_on(command_future);
}
