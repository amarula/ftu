use glob::glob;
use regex::Regex;
use std::{
    error::Error,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader},
};

fn read_file(path: String) -> Vec<String> {
    println!("Parsing file {path}");

    let file = match File::open(&path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let reader = BufReader::new(file);
    let mut translations = Vec::new();

    let single_line_re = Regex::new(r"'([^']+?)'\s*\.tr").unwrap();
    let triple_quote_start_re = Regex::new(r"'''").unwrap();
    let mut parsing_multi_line = false;
    let mut multi_line = String::new();

    for line in reader.lines().map_while(Result::ok) {
        if parsing_multi_line {
            if triple_quote_start_re.is_match(&line) {
                parsing_multi_line = false;
                if !multi_line.is_empty() && !translations.contains(&multi_line) {
                    println!("1.Translation found (multi): {multi_line}");
                    translations.push(multi_line.clone());
                }
                multi_line.clear();
            } else {
                multi_line.push_str(&line);
                multi_line.push('\n');
            }
            continue;
        }

        if triple_quote_start_re.is_match(&line) && line.contains(".tr") {
            parsing_multi_line = true;
            continue;
        }

        for cap in single_line_re.captures_iter(&line) {
            let tr_string = cap[1].trim().to_string();
            if !translations.contains(&tr_string) {
                println!("1.Translation found: {tr_string}");
                translations.push(tr_string);
            }
        }
    }

    translations
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

    let csv_output_path = format!("{}/{}.csv", path, language);
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
    use std::fs;
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
        assert!(content.contains("hello"));
        assert!(content.contains("world"));
        assert!(content.contains("example.dart"));
    }
}
