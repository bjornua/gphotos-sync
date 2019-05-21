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

mod iterdir;
use std::ffi::OsString;

fn main() -> () {
    let args = match get_args() {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };
    let files = crate::iterdir::findfiles(args.directory, args.extensions)
        .filter_map(Result::ok)
        .map(|m| m.dir_entry.path());
    println!("{:#?}", files.count())
}

struct Args {
    directory: OsString,
    extensions: Vec<OsString>,
}
fn get_args() -> Result<Args, &'static str> {
    let mut args = std::env::args_os().skip(1);
    let directory = match args.next() {
        Some(directory) => directory,
        None => return Err("Invalid directory"),
    };
    let extensions: Vec<_> = args.collect();

    return Ok(Args {
        directory,
        extensions,
    });
}
