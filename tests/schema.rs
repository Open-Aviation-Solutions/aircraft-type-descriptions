// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The data file fits its schema, and the schema refuses what it should.

use rstest::rstest;
use serde_json::Value;

const DATA: &str = include_str!("../aircraft-type-descriptions.toml");
const SCHEMA: &str = include_str!("../aircraft-type-descriptions.schema.json");

fn validator() -> jsonschema::Validator {
    let schema: Value = serde_json::from_str(SCHEMA).expect("the schema is JSON");
    jsonschema::draft202012::new(&schema).expect("the schema is a valid schema")
}

fn as_json(toml_text: &str) -> Value {
    let table: toml::Table = toml::from_str(toml_text).expect("the file is TOML");
    serde_json::to_value(table).expect("TOML converts to JSON")
}

#[test]
fn the_data_file_fits_the_schema() {
    let errors: Vec<String> = validator()
        .iter_errors(&as_json(DATA))
        .map(|e| format!("{} at {}", e, e.instance_path()))
        .collect();
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

/// Each replacement breaks the C172 row in one way.
#[rstest]
#[case::a_mistyped_value("airframe = \"landplane\"", "airframe = \"land-plane\"")]
#[case::an_unknown_field("models = ", "model = \"Cessna 172\"\nmodels = ")]
#[case::engines_with_no_engine(
    "engine-type = \"piston\"\nsources = [\"faa-3A12\"]",
    "engine-type = \"none\"\nsources = [\"faa-3A12\"]"
)]
fn the_schema_refuses(#[case] from: &str, #[case] to: &str) {
    let row = DATA
        .find("[designators.C172]")
        .expect("C172 is in the file");
    let at = row
        + DATA[row..]
            .find(from)
            .expect("the text to replace is in the row");
    let broken = format!("{}{}{}", &DATA[..at], to, &DATA[at + from.len()..]);
    assert!(!validator().is_valid(&as_json(&broken)));
}
