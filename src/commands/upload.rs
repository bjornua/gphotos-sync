use clap::{App, Arg, ArgMatches, SubCommand};

pub fn get_subcommand() -> App<'static, 'static> {
    SubCommand::with_name("upload")
        .about("Upload photos to Google")
        .arg(
            Arg::with_name("DIRECTORY")
                .index(1)
                .required(true)
                .multiple(false),
        )
        .arg(
            Arg::with_name("EXTENSION")
                .index(2)
                .required(true)
                .multiple(true),
        )
}

pub fn main(matches: &ArgMatches) {
    let directory = matches.value_of_os("DIRECTORY").unwrap().to_os_string();
    let extensions = matches.values_of_os("EXTENSION").unwrap();

    let files = crate::iterdir::findfiles(directory, extensions)
        .filter_map(Result::ok)
        .map(|m| m.dir_entry.path());
    println!("{:#?}", files.collect::<Vec<_>>())
}
