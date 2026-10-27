// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The packaged table: a row's fields come through from the data file, and
//! what is not a row is not guessed at.

use aircraft_type_descriptions::{
    AirframeKind, Confidence, DesignatorTable, EngineType, SourceKind,
};
use rstest::rstest;

use Confidence::{Confirmed, Provisional};

#[test]
fn a_row_comes_through_from_the_data_file() {
    let c172 = DesignatorTable::packaged()
        .lookup("C172")
        .expect("in the table");
    assert_eq!(c172.models, "Cessna 172 Skyhawk");
    assert_eq!(c172.description.airframe, AirframeKind::LandPlane);
    assert_eq!(c172.description.engine_count, 1);
    assert_eq!(c172.description.engine_type, EngineType::Piston);
}

#[rstest]
#[case::too_long_to_be_a_designator("C1720")]
#[case::not_to_be_used("PA28")]
#[case::lower_case("c172")]
fn not_a_row(#[case] designator: &str) {
    assert_eq!(DesignatorTable::packaged().lookup(designator), None);
}

/// A type certificate data sheet that is named but not read leaves a row
/// provisional; read, it confirms it.
#[test]
fn only_a_data_sheet_read_for_the_row_confirms_it() {
    let table = DesignatorTable::packaged();
    let c152 = table.lookup("C152").unwrap();
    assert_eq!(
        c152.sources[0].source.kind,
        SourceKind::TypeCertificateDataSheet
    );
    assert!(!c152.sources[0].checked);
    assert_eq!(c152.confidence, Provisional);

    let c172 = table.lookup("C172").unwrap();
    assert!(c172.sources[0].checked);
    assert_eq!(c172.confidence, Confirmed);
}

#[test]
fn a_row_carries_its_sources_and_notes() {
    let as21 = DesignatorTable::packaged().lookup("AS21").unwrap();
    assert_eq!(as21.models, "Schleicher ASK 21 Mi");
    assert_eq!(as21.sources.len(), 1);
    assert_eq!(as21.sources[0].source.id, "easa-A221");
    assert!(as21.sources[0]
        .note
        .unwrap()
        .starts_with("Not yet confirmed"));
    assert!(as21
        .notes
        .unwrap()
        .starts_with("The self-launching variant"));
}

#[test]
fn the_citation_is_each_source_with_its_revision_and_notes() {
    let table = DesignatorTable::packaged();
    assert_eq!(
        table.lookup("C172").unwrap().citation,
        "FAA Type Certificate Data Sheet 3A12 rev 80 (26 May 2010)"
    );
    assert_eq!(
        table.lookup("C152").unwrap().citation,
        "FAA Type Certificate Data Sheet 3A19 (Cessna 150/152) — A candidate: the revision \
         is not yet checked."
    );
}

#[test]
fn a_designator_not_to_be_used_says_what_replaces_it() {
    let pa28 = DesignatorTable::packaged().not_to_be_used("PA28").unwrap();
    assert_eq!(pa28.replaced_by, ["P28A", "P28B", "P28R", "P28T", "P28U"]);
    assert!(DesignatorTable::packaged().not_to_be_used("P28A").is_none());
}

#[rstest]
#[case::piston(EngineType::Piston, false)]
#[case::turboprop(EngineType::TurboProp, true)]
#[case::jet(EngineType::Jet, true)]
#[case::none(EngineType::None, false)]
fn which_engine_types_are_gas_turbines(#[case] engine: EngineType, #[case] expected: bool) {
    assert_eq!(engine.is_gas_turbine(), expected);
}
