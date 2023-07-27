[![Rust](https://github.com/amarula/ftu/actions/workflows/rust.yml/badge.svg)](https://github.com/amarula/ftu/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

# ftu - Flutter Translation Utility

ftu is used to generate CSV translation files from Flutter projects that use GetX for internationalization and translations.

## Usage

### CSV Generation

To generate the CSV just run the program choosing the language within the Flutter project directory:
```bash
ftu -l it
```

or define the project location via the parameter
```bash
ftu -l en -p /home/work/flutter-project
```

The result will be a CSV-generated file containing 3 columns: location, source, and translation.

### Dart translation generation (TODO)

To generate the Dart file containing the translation from the CSV, run the program choosing the language e specify the 
CSV file:
```bash
ftu -l de --csv de.csv
```

The result will be a `de.dart` file containing a `Map<String,String>` of translations.

## Authors
 * **Andrea Ricchi** - *Maintainer* - [AndreaRicchi](https://github.com/AndreaRicchi)