// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The `designator -> description` table, with the sources of each row.
//!
//! The table is built from `aircraft-type-descriptions.toml` when the crate
//! is built (`build.rs`), so nothing parses the file when the crate runs, and
//! no TOML parser is in the crate. [`DesignatorTable::packaged`] is that
//! table; it is a value, not a global, so that a table from elsewhere can be
//! added beside it later without changing what a caller of
//! [`DesignatorTable::lookup`] sees.

use serde::{Deserialize, Serialize};

use crate::description::{AircraftDescription, AirframeKind, EngineType};

/// How much weight a row carries.
///
/// It belongs to the row, not to anything a consumer derives from it: once
/// the description is right, a derivation from it is deterministic, so one
/// flag covers it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Confidence {
    /// Checked against a type certificate data sheet: the row names one as
    /// read for it.
    Confirmed,
    /// Not yet checked against a type certificate data sheet.
    Provisional,
    /// Not from the table at all: supplied wholly by a consumer's override.
    /// The table never gives it; it is here for a consumer that classifies a
    /// type the table does not have.
    Overridden,
}

/// What sort of source a row's facts come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceKind {
    /// A type certificate data sheet. The only kind that confirms a row.
    TypeCertificateDataSheet,
    /// A type certificate, where its data sheet has not been found.
    TypeCertificate,
    /// The manufacturer's information about its models.
    ManufacturerInformation,
    /// A reference article.
    ReferenceArticle,
}

/// A source, written once and named by each row it gives facts for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Source {
    /// The id the data file names it by.
    pub id: &'static str,
    pub title: &'static str,
    pub kind: SourceKind,
    /// The revision or issue and its date. `None` when not yet checked.
    pub revision: Option<&'static str>,
    pub url: Option<&'static str>,
    /// What is true of the source for every row that names it.
    pub notes: Option<&'static str>,
}

/// A source as one row names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CitedSource {
    pub source: &'static Source,
    /// Whether someone has read the source for this row.
    pub checked: bool,
    /// A note about this source that holds for this row only.
    pub note: Option<&'static str>,
}

/// One row: the facts, where they came from, and how far to trust them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DesignatorFacts {
    /// The manufacturer and models the designator covers.
    pub models: &'static str,
    pub description: AircraftDescription,
    /// `Confirmed` when a type certificate data sheet among `sources` has
    /// been read for the row, otherwise `Provisional`.
    pub confidence: Confidence,
    pub sources: &'static [CitedSource],
    /// What the row covers and what is left to check.
    pub notes: Option<&'static str>,
    /// The row's sources as one line of text: for each source, its title and
    /// revision, then after ` — ` its notes and the row's note about it;
    /// sources are separated by `; `.
    pub citation: &'static str,
}

/// A string known not to be a current designator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NotToBeUsed {
    /// The designators to use instead. Not all of them need be in the table.
    pub replaced_by: &'static [&'static str],
    /// Why, and how to choose among `replaced_by`.
    pub notes: &'static str,
}

/// The table of designators, and of the strings known not to be designators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesignatorTable {
    /// Sorted by designator.
    designators: &'static [(&'static str, DesignatorFacts)],
    /// Sorted by designator.
    not_to_be_used: &'static [(&'static str, NotToBeUsed)],
}

mod packaged {
    use super::{
        AircraftDescription, AirframeKind, CitedSource, Confidence, DesignatorFacts, EngineType,
        NotToBeUsed, Source, SourceKind,
    };
    #[allow(unused_imports)]
    use crate::description::WakeTurbulenceCategory;

    include!(concat!(env!("OUT_DIR"), "/packaged.rs"));
}

impl DesignatorTable {
    /// The table packaged with this version of the crate.
    pub fn packaged() -> Self {
        Self {
            designators: &packaged::DESIGNATORS,
            not_to_be_used: &packaged::NOT_TO_BE_USED,
        }
    }

    /// The row for a type designator.
    ///
    /// Returns `None` for a designator the table does not have, rather than
    /// guessing; that includes one that is [not to be
    /// used](Self::not_to_be_used).
    pub fn lookup(&self, designator: &str) -> Option<DesignatorFacts> {
        find(self.designators, designator)
    }

    /// Whether a string is known not to be a current designator, and what to
    /// use instead.
    pub fn not_to_be_used(&self, designator: &str) -> Option<NotToBeUsed> {
        find(self.not_to_be_used, designator)
    }
}

fn find<T: Copy>(rows: &[(&str, T)], designator: &str) -> Option<T> {
    rows.binary_search_by(|(key, _)| (*key).cmp(designator))
        .ok()
        .map(|index| rows[index].1)
}
