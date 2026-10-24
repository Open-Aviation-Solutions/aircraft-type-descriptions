// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Turns `aircraft-type-descriptions.toml` into the packaged table: Rust
//! statics in `$OUT_DIR/packaged.rs`, which `src/table.rs` includes. So the
//! file is parsed here, when the crate is built, and never when it runs.
//!
//! The fields' types are the schema's; serde refuses a field or value that is
//! not one of them. Here too is what the schema cannot say: that every source
//! a row names is listed, that `checked-against` and `source-notes` name only
//! the row's own sources, and that each section is in byte (ASCII) order of
//! its keys. A file that breaks any of it does not build.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::{env, fs, path::Path, process};

use serde::Deserialize;

const DATA_FILE: &str = "aircraft-type-descriptions.toml";
const SCHEMA_VERSION: u32 = 1;

/// The schema version alone, read before anything else so that a file for a
/// newer schema is refused as that, not for a field or value this crate does
/// not know.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Version {
    schema_version: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct File {
    /// Already read, and refused if it is not ours, by [`Version`].
    #[serde(rename = "schema-version")]
    _schema_version: u32,
    sources: BTreeMap<String, Source>,
    designators: BTreeMap<String, Row>,
    #[serde(default)]
    not_to_be_used: BTreeMap<String, NotToBeUsed>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct Source {
    title: String,
    kind: SourceKind,
    revision: Option<String>,
    url: Option<String>,
    notes: Option<String>,
}

#[derive(Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum SourceKind {
    TypeCertificateDataSheet,
    TypeCertificate,
    ManufacturerInformation,
    ReferenceArticle,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct Row {
    models: String,
    airframe: Airframe,
    engines: u8,
    engine_type: EngineType,
    wake_turbulence_category: Option<WakeTurbulenceCategory>,
    sources: Vec<String>,
    #[serde(default)]
    checked_against: Vec<String>,
    notes: Option<String>,
    #[serde(default)]
    source_notes: BTreeMap<String, String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
enum Airframe {
    Landplane,
    Seaplane,
    Amphibian,
    Helicopter,
    Gyrocopter,
    Tiltrotor,
    Glider,
    Airship,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
enum EngineType {
    Piston,
    Turboprop,
    Jet,
    None,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
enum WakeTurbulenceCategory {
    Light,
    Medium,
    Heavy,
    Super,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct NotToBeUsed {
    replaced_by: Vec<String>,
    notes: String,
}

fn main() {
    println!("cargo::rerun-if-changed={DATA_FILE}");
    println!("cargo::rerun-if-changed=build.rs");

    let text = fs::read_to_string(DATA_FILE).unwrap_or_else(|e| fail(&format!("{e}")));
    let version: Version = toml::from_str(&text).unwrap_or_else(|e| fail(&format!("{e}")));
    if version.schema_version != SCHEMA_VERSION {
        fail(&format!(
            "schema-version {} is not {SCHEMA_VERSION}, the one this version of the crate reads",
            version.schema_version
        ));
    }
    let file: File = toml::from_str(&text).unwrap_or_else(|e| fail(&format!("{e}")));
    let ordered: toml::Table = toml::from_str(&text).unwrap_or_else(|e| fail(&format!("{e}")));
    if let Some(error) = out_of_order(&ordered) {
        fail(&error);
    }

    let errors = check(&file);
    if !errors.is_empty() {
        fail(&errors.join("\n"));
    }

    let out = Path::new(&env::var("OUT_DIR").expect("cargo sets OUT_DIR")).join("packaged.rs");
    fs::write(out, generate(&file)).unwrap_or_else(|e| fail(&format!("{e}")));
}

fn fail(message: &str) -> ! {
    eprintln!("{DATA_FILE}: {message}");
    process::exit(1);
}

/// The first key out of byte order in a section, so that a new row has one
/// place to go. `ordered` keeps the file's order (toml's `preserve_order`),
/// which `File`'s maps do not.
fn out_of_order(ordered: &toml::Table) -> Option<String> {
    for section in ["sources", "designators", "not-to-be-used"] {
        let Some(toml::Value::Table(table)) = ordered.get(section) else {
            continue;
        };
        let keys: Vec<&String> = table.keys().collect();
        for pair in keys.windows(2) {
            if pair[1].as_bytes() < pair[0].as_bytes() {
                return Some(format!(
                    "[{section}.{}] is out of order: it comes after [{section}.{}], and \
                     each section is in byte (ASCII) order of its keys",
                    pair[1], pair[0]
                ));
            }
        }
    }
    None
}

/// What the schema cannot say, each broken rule as one line.
fn check(file: &File) -> Vec<String> {
    let mut errors = Vec::new();
    for (designator, row) in &file.designators {
        for id in &row.sources {
            if !file.sources.contains_key(id) {
                errors.push(format!(
                    "{designator}: source {id:?} is not under [sources]"
                ));
            }
        }
        for id in &row.checked_against {
            if !row.sources.contains(id) {
                errors.push(format!(
                    "{designator}: checked-against {id:?} is not one of the row's sources"
                ));
            }
        }
        for id in row.source_notes.keys() {
            if !row.sources.contains(id) {
                errors.push(format!(
                    "{designator}: source-notes {id:?} is not one of the row's sources"
                ));
            }
        }
    }
    for designator in file.not_to_be_used.keys() {
        if file.designators.contains_key(designator) {
            errors.push(format!(
                "{designator} is both a designator and not to be used"
            ));
        }
    }
    errors
}

fn generate(file: &File) -> String {
    let index: BTreeMap<&str, usize> = file
        .sources
        .keys()
        .enumerate()
        .map(|(i, id)| (id.as_str(), i))
        .collect();

    let mut out = String::new();
    writeln!(out, "// Generated by build.rs from {DATA_FILE}.").unwrap();

    writeln!(out, "static SOURCES: [Source; {}] = [", file.sources.len()).unwrap();
    for (id, source) in &file.sources {
        writeln!(
            out,
            "    Source {{ id: {id:?}, title: {:?}, kind: SourceKind::{}, revision: {}, url: {}, notes: {} }},",
            source.title,
            source_kind(source.kind),
            option(&source.revision),
            option(&source.url),
            option(&source.notes),
        )
        .unwrap();
    }
    writeln!(out, "];").unwrap();

    writeln!(
        out,
        "pub(super) static DESIGNATORS: [(&str, DesignatorFacts); {}] = [",
        file.designators.len()
    )
    .unwrap();
    for (designator, row) in &file.designators {
        let confirmed = row
            .checked_against
            .iter()
            .any(|id| file.sources[id].kind == SourceKind::TypeCertificateDataSheet);
        let cited: Vec<String> = row
            .sources
            .iter()
            .map(|id| {
                format!(
                    "CitedSource {{ source: &SOURCES[{}], checked: {}, note: {} }}",
                    index[id.as_str()],
                    row.checked_against.contains(id),
                    option(&row.source_notes.get(id).cloned()),
                )
            })
            .collect();
        let citation = row
            .sources
            .iter()
            .map(|id| {
                let source = &file.sources[id];
                let mut cited = source.title.clone();
                if let Some(revision) = &source.revision {
                    write!(cited, " {revision}").unwrap();
                }
                let notes: Vec<&str> = [source.notes.as_ref(), row.source_notes.get(id)]
                    .into_iter()
                    .flatten()
                    .map(|note| note.trim())
                    .collect();
                if let Some(url) = &source.url {
                    write!(cited, ", {url}").unwrap();
                }
                if !notes.is_empty() {
                    write!(cited, " — {}", notes.join(" ")).unwrap();
                }
                cited
            })
            .collect::<Vec<_>>()
            .join("; ");
        writeln!(
            out,
            "    ({designator:?}, DesignatorFacts {{ models: {:?}, description: AircraftDescription {{ \
             airframe: AirframeKind::{}, engine_count: {}, engine_type: EngineType::{}, \
             wake_turbulence_category: {} }}, confidence: Confidence::{}, sources: &[{}], \
             notes: {}, citation: {citation:?} }}),",
            row.models,
            airframe(&row.airframe),
            row.engines,
            engine_type(&row.engine_type),
            match &row.wake_turbulence_category {
                Some(category) => format!("Some(WakeTurbulenceCategory::{category:?})"),
                None => "None".to_string(),
            },
            if confirmed { "Confirmed" } else { "Provisional" },
            cited.join(", "),
            option(&row.notes),
        )
        .unwrap();
    }
    writeln!(out, "];").unwrap();

    writeln!(
        out,
        "pub(super) static NOT_TO_BE_USED: [(&str, NotToBeUsed); {}] = [",
        file.not_to_be_used.len()
    )
    .unwrap();
    for (designator, entry) in &file.not_to_be_used {
        writeln!(
            out,
            "    ({designator:?}, NotToBeUsed {{ replaced_by: &{:?}, notes: {:?} }}),",
            entry.replaced_by,
            entry.notes.trim(),
        )
        .unwrap();
    }
    writeln!(out, "];").unwrap();
    out
}

/// A note as the file writes it, without the line break a `"""` string ends
/// with.
fn option(text: &Option<String>) -> String {
    match text {
        Some(text) => format!("Some({:?})", text.trim()),
        None => "None".to_string(),
    }
}

fn source_kind(kind: SourceKind) -> &'static str {
    match kind {
        SourceKind::TypeCertificateDataSheet => "TypeCertificateDataSheet",
        SourceKind::TypeCertificate => "TypeCertificate",
        SourceKind::ManufacturerInformation => "ManufacturerInformation",
        SourceKind::ReferenceArticle => "ReferenceArticle",
    }
}

fn airframe(airframe: &Airframe) -> &'static str {
    match airframe {
        Airframe::Landplane => "LandPlane",
        Airframe::Seaplane => "SeaPlane",
        Airframe::Amphibian => "Amphibian",
        Airframe::Helicopter => "Helicopter",
        Airframe::Gyrocopter => "Gyrocopter",
        Airframe::Tiltrotor => "TiltRotor",
        Airframe::Glider => "Glider",
        Airframe::Airship => "Airship",
    }
}

fn engine_type(engine_type: &EngineType) -> &'static str {
    match engine_type {
        EngineType::Piston => "Piston",
        EngineType::Turboprop => "TurboProp",
        EngineType::Jet => "Jet",
        EngineType::None => "None",
    }
}
