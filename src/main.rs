pub mod dart_to_csv;

use argh::FromArgs;
use std::process;

#[derive(FromArgs)]
/// Flutter Translation Service. Used to generate translations file for flutter applications.
struct AppArgs {
    /// language selected for current run
    #[argh(option, short = 'l')]
    language: String,

    /// CSV file, if set, dart file will be generated
    #[argh(option, short = 'c')]
    csv: Option<String>,

    /// flutter project path
    #[argh(option, short = 'p', default = "String::from(\".\")")]
    path: String,
}

pub fn main() {
    let args: AppArgs = argh::from_env();

    if args.csv.is_some() {
        process::exit(0);
    } else {
        dart_to_csv::dart_to_csv(&args.path, &args.language);
    }
}
