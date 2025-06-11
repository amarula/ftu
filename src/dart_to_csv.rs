use std::{
    error::Error,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader},
};

use glob::glob;

fn read_file(path: String) -> Vec<String> {
    println!("Parsing file {path}");

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let reader = BufReader::new(file);
    let mut translations = Vec::new();

    let mut previous_line = String::new();
    let mut parsing_multi_line = false;
    let mut multi_line = String::new();

    for current_line in reader.lines().map_while(Result::ok) {
        let start_bytes = current_line.find('\'').unwrap_or(0);
        let end_bytes = current_line.find("'.tr").unwrap_or(current_line.len());
        let mut result = (current_line[start_bytes..end_bytes]).to_string();

        if !parsing_multi_line && current_line.ends_with("'''") {
            parsing_multi_line = true;
            continue;
        }

        if parsing_multi_line {
            if current_line.ends_with(".tr,") && previous_line.ends_with("'''") {
                parsing_multi_line = false;
                println!("1.Translation found: {multi_line}");
                translations.push(multi_line.clone());
                continue;
            } else {
                let mut current_line_clone = current_line.clone();

                if current_line_clone.ends_with("'''") {
                    previous_line = current_line.clone();
                    current_line_clone.truncate(current_line_clone.len() - 3)
                }

                multi_line += current_line_clone.as_str();
                continue;
            }
        }

        if !current_line.contains(".tr,") && !current_line.contains(".tr;") && !parsing_multi_line {
            previous_line = result;
            continue;
        }

        if !previous_line.is_empty()
            && previous_line.ends_with('\'')
            && current_line.ends_with(".tr,")
        {
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
