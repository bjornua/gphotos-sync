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
mod gphotos;
mod iterdir;
mod process;

fn main() -> () {
    let matches = clap::App::new("sd-photo-uploader")
        .version("0.1")
        .author("Bjørn Arnholtz. <bjorn.arnholtz@gmail.com>")
        .about("Uploads photos from a certain directory to google photos")
        // .args_from_usage(
        //     "-c, --config=[FILE] 'Sets a custom config file'
        //                       <INPUT>              'Sets the input file to use'
        //                       -v...                'Sets the level of verbosity'",
        // )
        .subcommand(
            clap::SubCommand::with_name("test")
                .about("controls testing features")
                .version("1.3")
                .author("Someone E. <someone_else@other.com>")
                .arg_from_usage("-d, --debug 'Print debug information'"),
        )
        .get_matches();
    gphotos::main();
}
