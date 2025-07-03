use glob::glob;
use regex::Regex;
use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::{BufRead, BufReader},
};

fn read_file(path: String) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    println!("Parsing file {path}");

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;

    // Join all lines into a single string to handle multi-line cases
    let content = lines.join("\n");

    // Regex pattern to match strings ending with .tr
    // This pattern matches:
    // - Single or double quoted strings
    // - Followed by .tr
    // - Handles potential whitespace and punctuation after .tr
    let re = Regex::new(r#"['"]([^'"]*?)['"][\s\n]*\.tr"#)?;

    let mut tr_strings = Vec::new();

    for captures in re.captures_iter(&content) {
        if let Some(matched) = captures.get(1) {
            tr_strings.push(matched.as_str().to_string());
        }
    }

    Ok(tr_strings)
}

fn write_file(
    csv_path: &String,
    sources_file: &String,
    translations: &Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(csv_path)
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
            read_file(path.display().to_string()).unwrap(),
        );
        if translation_in_file.1.is_empty() {
            continue;
        }
        all_translations.push(translation_in_file);
    }

    if all_translations.is_empty() {
        return;
    }

    let csv_output_path = format!("{path}/{language}.csv");
    let mut wtr = csv::Writer::from_path(&csv_output_path).expect("Failed to create CSV file");

    let _ = wtr.write_record(["location", "source", "translation"]);
    let _ = wtr.flush();

    for translation in all_translations {
        let _ = write_file(&csv_output_path, &translation.0, &translation.1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write as IoWrite;
    use tempfile::tempdir;

    #[test]
    fn test_dart_to_csv_creates_correct_csv() {
        let dir = tempdir().unwrap();
        let dart_file_path = dir.path().join("example.dart");

        // Simulate Dart file with translatable strings
        let dart_content = "
      final a = 'hello'.tr;
      final b = 'world'.tr;
      final c = 'Concatenated'.tr + string;
      final d = 'Multi line string'
                .tr,;
      final e = 'not a tr string';
      final f = 'another'.tr;
    ";

        let mut file = File::create(&dart_file_path).unwrap();
        writeln!(file, "{}", dart_content).unwrap();

        let dir_path = dir.path().to_str().unwrap().to_string();
        let language = "en".to_string();

        // Run the conversion
        dart_to_csv(&dir_path, &language);

        // Check the CSV output
        let csv_path = dir.path().join("en.csv");
        assert!(csv_path.exists(), "CSV file should be created");

        let content = fs::read_to_string(csv_path).unwrap();
        assert!(content.contains("example.dart"));
        assert!(content.contains("hello"));
        assert!(content.contains("world"));
        assert!(content.contains("Concatenated"));
        assert!(content.contains("Multi line string"));
        assert!(content.contains("another"));
    }
}
