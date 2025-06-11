use serde::Deserialize;
use std::fs::File;
use std::io::{LineWriter, Write};

#[derive(Deserialize, Debug, Clone, PartialEq)]
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
    let reader_result = csv::Reader::from_path(path);
    if reader_result.is_err() {
        eprintln!("Failed to open CSV file at path: {}", path);
        return translations;
    }
    let mut reader = reader_result.unwrap();

    for result in reader.deserialize() {
        let record: Record = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error deserializing record: {}", e);
                Record {
                    location: String::from("error"),
                    source: String::from("error"),
                    translation: String::from("error"),
                }
            }
        };

        if record.source == "error" && record.location == "error" {
            continue;
        }

        if !check_for_duplicates(&translations, &record) {
            translations.push(record);
        }
    }
    translations
}

fn write_file(language: &String, translations: &Vec<Record>) -> std::io::Result<()> {
    let file_path = format!("{}.dart", language);
    let file = File::create(&file_path)?;
    let mut file_writer = LineWriter::new(file);
    file_writer.write_all(b"// ignore_for_file: file_names\n")?;
    file_writer.write_all(b"\n")?;
    file_writer.write_all(format!("const Map<String, String> {0} = {{\n", language).as_bytes())?;
    for record in translations {
        let s = record.source.replace("\"", "\\\"");
        let t = record.translation.replace("\"", "\\\"");
        file_writer.write_all(format!("  \"{0}\": \"{1}\",\n", s, t).as_bytes())?;
    }
    file_writer.write_all(b"};\n")?;
    file_writer.flush()?;
    Ok(())
}

pub fn csv_to_dart(csv_input_path: &String, language: &String) -> std::io::Result<()> {
    let translations = read_file(csv_input_path);
    write_file(language, &translations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;
    use std::io::Write as IoWrite;
    use std::path::Path;
    use tempfile::NamedTempFile;

    fn cleanup_dart_file(language_name: &str) {
        let dart_file_path_str = format!("{}.dart", language_name);
        let dart_file_path = Path::new(&dart_file_path_str);
        if dart_file_path.exists() {
            if let Err(e) = fs::remove_file(dart_file_path) {
                eprintln!(
                    "Warning: Could not remove Dart file {:?}: {}",
                    dart_file_path, e
                );
            }
        }
    }

    #[test]
    fn test_check_for_duplicates_empty_vec() {
        let records: Vec<Record> = Vec::new();
        let new_record = Record {
            location: "loc1".to_string(),
            source: "src1".to_string(),
            translation: "trn1".to_string(),
        };
        assert!(!check_for_duplicates(&records, &new_record));
    }

    #[test]
    fn test_check_for_duplicates_no_duplicate() {
        let records = vec![Record {
            location: "loc1".to_string(),
            source: "src1".to_string(),
            translation: "trn1".to_string(),
        }];
        let new_record = Record {
            location: "loc2".to_string(),
            source: "src2".to_string(),
            translation: "trn2".to_string(),
        };
        assert!(!check_for_duplicates(&records, &new_record));
    }

    #[test]
    fn test_check_for_duplicates_with_duplicate() {
        let records = vec![Record {
            location: "loc1".to_string(),
            source: "src1".to_string(),
            translation: "trn1".to_string(),
        }];
        let new_record = Record {
            location: "loc2".to_string(),
            source: "src1".to_string(),
            translation: "trn2".to_string(),
        };
        assert!(check_for_duplicates(&records, &new_record));
    }

    #[test]
    fn test_read_file_empty_csv() {
        let mut temp_csv_file = NamedTempFile::new().expect("Failed to create temp CSV file");
        temp_csv_file
            .write_all(b"location,source,translation\n")
            .expect("Failed to write to temp CSV");
        let file_path_str = temp_csv_file.path().to_str().unwrap().to_string();

        let records = read_file(&file_path_str);
        assert!(records.is_empty());
    }

    #[test]
    fn test_read_file_valid_csv() {
        let content = "location,source,translation\n\
                       loc1,src1,trn1\n\
                       loc2,src2,trn2";
        let mut temp_csv_file = NamedTempFile::new().expect("Failed to create temp CSV file");
        temp_csv_file
            .write_all(content.as_bytes())
            .expect("Failed to write to temp CSV");
        let file_path_str = temp_csv_file.path().to_str().unwrap().to_string();

        let records = read_file(&file_path_str);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].source, "src1");
        assert_eq!(records[1].translation, "trn2");
    }

    #[test]
    fn test_read_file_with_duplicates_in_csv() {
        let content = "location,source,translation\n\
                       loc1,src1,trn1\n\
                       loc2,src2,trn2\n\
                       loc3,src1,trn1_dup";
        let mut temp_csv_file = NamedTempFile::new().expect("Failed to create temp CSV file");
        temp_csv_file
            .write_all(content.as_bytes())
            .expect("Failed to write to temp CSV");
        let file_path_str = temp_csv_file.path().to_str().unwrap().to_string();

        let records = read_file(&file_path_str);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].source, "src1");
        assert_eq!(records[0].translation, "trn1");
        assert_eq!(records[1].source, "src2");
    }

    #[test]
    fn test_read_file_malformed_line() {
        let content = "location,source,translation\n\
                       loc1,src1,trn1\n\
                       this,isnot,enough,fields\n\
                       loc3,src3,trn3";
        let mut temp_csv_file = NamedTempFile::new().expect("Failed to create temp CSV file");
        temp_csv_file
            .write_all(content.as_bytes())
            .expect("Failed to write to temp CSV");
        let file_path_str = temp_csv_file.path().to_str().unwrap().to_string();

        let records = read_file(&file_path_str);
        assert_eq!(records.len(), 2, "Should skip the malformed line.");
        assert_eq!(records[0].source, "src1");
        assert_eq!(records[1].source, "src3");
    }

    #[test]
    fn test_read_file_csv_with_empty_lines() {
        let content = "location,source,translation\n\
                       loc1,src1,trn1\n\
                       \n\
                       loc2,src2,trn2\n";
        let mut temp_csv_file = NamedTempFile::new().expect("Failed to create temp CSV file");
        temp_csv_file
            .write_all(content.as_bytes())
            .expect("Failed to write to temp CSV");
        let file_path_str = temp_csv_file.path().to_str().unwrap().to_string();

        let records = read_file(&file_path_str);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].source, "src1");
        assert_eq!(records[1].source, "src2");
    }

    #[test]
    fn test_write_file_empty_records() {
        let lang_name = "empty_lang_test_write";
        let records: Vec<Record> = Vec::new();

        let result = write_file(&lang_name.to_string(), &records);
        assert!(result.is_ok());

        let dart_file_path_str = format!("{}.dart", lang_name);
        let mut file_content = String::new();
        File::open(&dart_file_path_str)
            .expect("Failed to open Dart file for reading")
            .read_to_string(&mut file_content)
            .expect("Failed to read Dart file");

        let expected_content = format!(
            "// ignore_for_file: file_names\n\
             \n\
             const Map<String, String> {} = {{\n\
             }};\n",
            lang_name
        );
        assert_eq!(file_content, expected_content);
        cleanup_dart_file(lang_name);
    }

    #[test]
    fn test_write_file_with_records() {
        let lang_name = "test_lang_test_write";
        let records = vec![
            Record {
                location: "l1".to_string(),
                source: "hello".to_string(),
                translation: "world".to_string(),
            },
            Record {
                location: "l2".to_string(),
                source: "foo".to_string(),
                translation: "bar".to_string(),
            },
            Record {
                location: "l3".to_string(),
                source: "quote_test".to_string(),
                translation: "this is a \"test\"".to_string(),
            },
        ];
        let result = write_file(&lang_name.to_string(), &records);
        assert!(result.is_ok());

        let dart_file_path_str = format!("{}.dart", lang_name);
        let mut file_content = String::new();
        File::open(&dart_file_path_str)
            .expect("Failed to open Dart file for reading")
            .read_to_string(&mut file_content)
            .expect("Failed to read Dart file");

        let expected_content = format!(
            concat!(
                "// ignore_for_file: file_names\n",
                "\n",
                "const Map<String, String> {} = {{\n",
                "  \"hello\": \"world\",\n",
                "  \"foo\": \"bar\",\n",
                "  \"quote_test\": \"this is a \\\"test\\\"\",\n",
                "}};\n"
            ),
            lang_name
        );
        assert_eq!(file_content, expected_content);
        cleanup_dart_file(lang_name);
    }

    #[test]
    fn test_csv_to_dart_integration() {
        let csv_content = "location,source,translation\n\
                           app_title,AppTitle,My Application\n\
                           greeting,GreetingMessage,Hello User!\n\
                           another_loc,AppTitle,This Should Be Ignored Because First AppTitle Wins";

        let mut temp_csv_file = NamedTempFile::new().expect("Failed to create temp CSV file");
        temp_csv_file
            .write_all(csv_content.as_bytes())
            .expect("Failed to write to temp CSV");
        let csv_path_str = temp_csv_file.path().to_str().unwrap().to_string();

        let lang_name = "integration_lang_test";

        let result = csv_to_dart(&csv_path_str, &lang_name.to_string());
        assert!(result.is_ok());

        let dart_file_path_str = format!("{}.dart", lang_name);
        let mut file_content = String::new();
        File::open(&dart_file_path_str)
            .expect("Failed to open Dart file for reading")
            .read_to_string(&mut file_content)
            .expect("Failed to read Dart file");

        let expected_content = format!(
            concat!(
                "// ignore_for_file: file_names\n",
                "\n",
                "const Map<String, String> {} = {{\n",
                "  \"AppTitle\": \"My Application\",\n",
                "  \"GreetingMessage\": \"Hello User!\",\n",
                "}};\n"
            ),
            lang_name
        );
        assert_eq!(file_content, expected_content);
        cleanup_dart_file(lang_name);
    }

    #[test]
    fn test_read_file_non_existent_csv() {
        let file_path = "non_existent_test_file.csv".to_string();
        let records = read_file(&file_path);
        assert!(
            records.is_empty(),
            "Expected empty records for non-existent file."
        );
    }
}
