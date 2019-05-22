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

extern crate hyper;
extern crate open;
mod gphotos;
mod iterdir;
mod process;

fn main() -> () {
    gphotos::main();
}
