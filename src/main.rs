/*
Goal:
    Empty pictures from sd-card and upload to google images without user interaction

Features:
    Generate list of file to process
    Detect images
    Detect sd-card is inserted
    Delete successfuly uploaded files
    Send notification (email, IM, whatever) when certain events happens
        * Transfer started (label, count, size, eta)
        * Transfer completed (suceeded, failed, size, elapsed, speed, thumbs, errors)
        * SD-Card Inserted
        * Failure
    * Multiple destinations
        * Google
        * FTP / SFTP / SCP

    Checksums?

    Duplicate detection (skip alredy uploaded files)
    Read configuration file from mounted file-system

*/

extern crate clap;
extern crate futures;
extern crate hyper;
extern crate open;
extern crate url;

use clap::{App, AppSettings};

mod commands;
mod config;
mod gphotos;
mod iterdir;

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

    match matches.subcommand() {
        ("upload", Some(args)) => commands::upload::main(args),
        ("authenticate", Some(args)) => commands::authenticate::main(args),
        (_, _) => unimplemented!(),
    }
}
