use serde::Deserialize;
use std::fs::File;
use std::io::{LineWriter, Write};

#[derive(Deserialize)]
pub struct Record {
    location: String,
    source: String,
    translation: String,
}

fn check_for_duplicates(vec: &Vec<Record>, new_record: &Record) -> bool {
    for record in vec {
        if record.source == new_record.source {
            return true;
        }
    }

    false
}

fn read_file(path: &String) -> Vec<Record> {
    let mut translations = Vec::new();

    let reader = csv::Reader::from_path(path);
    for record in reader.unwrap().deserialize() {
        let record: Record = record.unwrap_or(Record {
            location: String::new(),
            source: String::new(),
            translation: String::new(),
        });
        println!(
            "In '{}', source '{}' - translation '{}'.",
            record.location, record.source, record.translation
        );
        if !check_for_duplicates(&translations, &record) {
            translations.push(record);
        }
    }

    translations
}

fn write_file(language: &String, translations: &Vec<Record>) {
    let file = File::create(format!("{}.dart", language)).expect("Failed to create DART file");
    let mut file = LineWriter::new(file);
    let _ = file.write_all(b"// ignore_for_file: file_names\n");
    let _ = file.write_all(b"\n");
    let _ = file.write_all(format!("const Map<String, String> {0} = {{\n", language).as_bytes());
    for record in translations {
        let _ = file.write_all(
            format!("\"{0}\": \"{1}\",\n", record.source, record.translation).as_bytes(),
        );
    }
    let _ = file.write_all(b"};\n");
    file.flush().expect("Failed to write DART file");
}

pub fn csv_to_dart(path: &String, language: &String) {
    let translations = read_file(path);
    write_file(language, &translations);
}
