use std::{
    error::Error,
    fs::{File, OpenOptions},
    io::{self, BufRead},
    path::Path,
};

use argh::FromArgs;
use glob::glob;
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

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn parse_file(path: String) -> Vec<String> {
    println!("Parsing file {path}");

    let mut translations = Vec::new();

    if let Ok(lines) = read_lines(path) {
        let mut previous_line = String::new();

        for ip in lines.flatten() {
            let start_bytes = ip.find('\'').unwrap_or(0);
            let end_bytes = ip.find(".tr").unwrap_or(ip.len());
            let mut result = (ip[start_bytes..end_bytes]).to_string();

            if !ip.ends_with(".tr,") {
                previous_line = result;
                continue;
            }

            if !previous_line.is_empty() && previous_line.ends_with('\'') && ip.ends_with(".tr,") {
                result = previous_line.clone();
            }

            if result.trim().is_empty() {
                continue;
            }

            while result.starts_with('\'') {
                result.remove(0);
            }

            while result.ends_with('\'') {
                result.pop();
            }

            println!("1.Translation found: {result}");

            translations.push(result);
        }
    }

    translations
}

fn write_file(
    language: &String,
    sources_file: &String,
    translations: &Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(format!("{language}.csv"))
        .unwrap();
    let mut wtr = csv::Writer::from_writer(file);
    for tr in translations {
        println!("3.Translation found: {sources_file} {tr}");
        wtr.write_record([sources_file, tr, &String::from("")])?;
    }

    wtr.flush()?;
    Ok(())
}

fn main() {
    let args: AppArgs = argh::from_env();

    if args.csv.is_some() {
        process::exit(0);
    }

    let mut all_translations = Vec::new();

    let project_path = { args.path } + "/**/*.dart";
    for path in glob(&project_path)
        .expect("Failed to read glob pattern")
        .flatten()
    {
        let translation_in_file = (
            path.display().to_string(),
            parse_file(path.display().to_string()),
        );
        if translation_in_file.1.is_empty() {
            continue;
        }
        all_translations.push(translation_in_file);
    }

    if all_translations.is_empty() {
        return;
    }

    let mut wtr = csv::Writer::from_path(format!("{}.csv", args.language))
        .expect("Failed to create CSV file");
    let _ = wtr.write_record(["location", "source", "translation"]);
    let _ = wtr.flush();

    for translation in all_translations {
        let _ = write_file(&args.language, &translation.0, &translation.1);
    }
}
