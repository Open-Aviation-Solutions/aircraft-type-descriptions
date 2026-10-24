// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The country-agnostic description of aircraft types, by type designator:
//! the kind of airframe, the number and kind of engines, and the wake
//! turbulence category, each row citing the sources its facts come from.
//!
//! - [`description`]: the types of a description ([`AircraftDescription`]).
//! - [`table`]: the table of rows ([`DesignatorTable`]), built from
//!   `aircraft-type-descriptions.toml` when the crate is built.
//!
//! Reading a description under one country's rules (a licensing category or
//! class) is not done here; it belongs to a national package.
//!
//! ```
//! use aircraft_type_descriptions::{AirframeKind, Confidence, DesignatorTable};
//!
//! let table = DesignatorTable::packaged();
//! let c172 = table.lookup("C172").expect("in the table");
//! assert_eq!(c172.description.airframe, AirframeKind::LandPlane);
//! assert_eq!(c172.description.engine_count, 1);
//! assert_eq!(c172.confidence, Confidence::Confirmed);
//!
//! // Not a current designator: the table says what replaces it.
//! assert!(table.lookup("PA28").is_none());
//! assert!(table.not_to_be_used("PA28").unwrap().replaced_by.contains(&"P28A"));
//! ```

pub mod description;
pub mod table;

pub use description::{AircraftDescription, AirframeKind, EngineType, WakeTurbulenceCategory};
pub use table::{
    CitedSource, Confidence, DesignatorFacts, DesignatorTable, NotToBeUsed, Source, SourceKind,
};

// Compiles and runs the README's examples with the doc tests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
