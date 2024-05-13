use std::{
    error::Error,
    fs::{File, OpenOptions},
    io::{self, BufRead},
    path::Path,
};

use glob::glob;

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn read_file(path: String) -> Vec<String> {
    println!("Parsing file {path}");

    let mut translations = Vec::new();

    if let Ok(lines) = read_lines(path) {
        let mut previous_line = String::new();

        for ip in lines.map_while(Result::ok) {
            let start_bytes = ip.find('\'').unwrap_or(0);
            let end_bytes = ip.find("'.tr").unwrap_or(ip.len());
            let mut result = (ip[start_bytes..end_bytes]).to_string();

            if !ip.contains(".tr,") && !ip.contains(".tr;") {
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

            if !translations.contains(&result) {
                println!("1.Translation found: {result}");
                translations.push(result);
            }
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

pub fn dart_to_csv(path: &String, language: &String) {
    if path.is_empty() || language.is_empty() {
        return;
    }

    let mut all_translations = Vec::new();

    let project_path = { path }.to_owned() + "/**/*.dart";
    for path in glob(&project_path)
        .expect("Failed to read glob pattern")
        .flatten()
    {
        let translation_in_file = (
            path.display().to_string(),
            read_file(path.display().to_string()),
        );
        if translation_in_file.1.is_empty() {
            continue;
        }
        all_translations.push(translation_in_file);
    }

    if all_translations.is_empty() {
        return;
    }

    let mut wtr =
        csv::Writer::from_path(format!("{}.csv", language)).expect("Failed to create CSV file");
    let _ = wtr.write_record(["location", "source", "translation"]);
    let _ = wtr.flush();

    for translation in all_translations {
        let _ = write_file(language, &translation.0, &translation.1);
    }
}
