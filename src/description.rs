// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The description of an aircraft type: what is true of it wherever it flies.
//!
//! Nothing here is narrowed to suit one country, or to what one consumer reads:
//!
//! - [`AircraftDescription`] carries the wake turbulence category
//!   ([`WakeTurbulenceCategory`]) even where no consumer reads it, since a
//!   shared table that leaves it out is quietly incomplete. It is optional: a row leaves it out where
//!   it has not been compiled yet, or where the type's variants fall in more
//!   than one category.
//! - [`AirframeKind`] carries `Glider` and `Airship` as kinds of airframe of
//!   their own, so that a glider and an airship can be told apart from other
//!   airframes.
//! - [`EngineType::is_gas_turbine`] says which kinds of engine are gas
//!   turbines.

use serde::{Deserialize, Serialize};

/// What sort of airframe a type is.
///
/// `Glider` and `Airship` are kinds of airframe beyond the landplane,
/// seaplane, amphibian, helicopter, gyrocopter and tiltrotor. They are there
/// so that a glider and an airship can be described at all, and told apart
/// from other airframes: a country's rules may treat either as a category of
/// its own, and a description that could not say which an aircraft is would
/// leave that to guesswork.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AirframeKind {
    /// A landplane.
    LandPlane,
    /// A seaplane.
    SeaPlane,
    /// An amphibian.
    Amphibian,
    /// A helicopter.
    Helicopter,
    /// A gyrocopter.
    Gyrocopter,
    /// A tiltrotor.
    TiltRotor,
    /// A glider: see the enum's own documentation.
    Glider,
    /// An airship: see the enum's own documentation.
    Airship,
}

/// The kind of engine a type has.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EngineType {
    /// Piston.
    Piston,
    /// Turboprop or turboshaft.
    TurboProp,
    /// Jet.
    Jet,
    /// No engine: a glider.
    None,
}

impl EngineType {
    /// Whether this kind of engine is a gas turbine.
    ///
    /// `TurboProp` (which covers a turboshaft) and `Jet` are; `Piston` and
    /// `None` are not. `Jet` means a turbojet or turbofan, both gas turbines.
    /// There is no value for a pulsejet, ramjet or rocket: if one is ever
    /// added as an engine type, revisit this method; the `match` is
    /// exhaustive so that the compiler asks for that.
    pub fn is_gas_turbine(self) -> bool {
        match self {
            Self::TurboProp | Self::Jet => true,
            Self::Piston | Self::None => false,
        }
    }
}

/// ICAO's wake turbulence category, by maximum certificated take-off mass
/// (SKYbrary, "ICAO Wake Turbulence Category").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WakeTurbulenceCategory {
    /// `L`: 7,000 kg or less.
    Light,
    /// `M`: less than 136,000 kg and more than 7,000 kg.
    Medium,
    /// `H`: 136,000 kg or more, other than a type in `Super`.
    Heavy,
    /// `J`: the types designated as such.
    Super,
}

/// The facts about an aircraft type that do not depend on any one country's
/// rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AircraftDescription {
    pub airframe: AirframeKind,
    pub engine_count: u8,
    pub engine_type: EngineType,
    /// `None` when not given: either it has not been compiled for the type,
    /// or the type's variants fall in more than one category (the row's
    /// notes say which). Left out of the JSON when `None`, so a description
    /// serialised before it was carried reads the same.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wake_turbulence_category: Option<WakeTurbulenceCategory>,
}

impl AircraftDescription {
    /// A description with no wake turbulence category.
    pub fn new(airframe: AirframeKind, engine_count: u8, engine_type: EngineType) -> Self {
        Self {
            airframe,
            engine_count,
            engine_type,
            wake_turbulence_category: None,
        }
    }
}
