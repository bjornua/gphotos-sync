/*
Goal:
    Empty pictures from sd-card and upload to google images without user interaction

Features:
    Detect sd-card is inserted
    Delete successfuly uploaded files
    Send notification (email, IM, whatever) when certain events happens
        * Transfer started
            * Timestamp
            * Label
            * Image count
            * Size
            * ETA
        * Transfer completed
            * Timestamp
            * Image count
            * Size
            * Elapsed time
            * Transfer speed
            * Thumbnails
            * Failed files with reason
        * SD-Card Inserted
        * Failure
        * Multiple destinations
        * Google
    Checksums?
    Duplicate detection (skip alredy uploaded files)
    Read configuration file from mounted file-system

*/

fn main() {
    println!("Hello, worlds!", { s });
}
