use std::ffi::OsString;

pub fn process() {
    let args = match get_args() {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };
    let files = crate::iterdir::findfiles(args.directory, &args.extensions)
        .filter_map(Result::ok)
        .map(|m| m.dir_entry.path());
    println!("{:#?}", files.collect::<Vec<_>>())
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
