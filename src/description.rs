// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The description of an aircraft type: what is true of it wherever it flies.
//!
//! These types moved here from au-casa, where Part 61 had narrowed them in
//! three places. On the move, the one a consumer could not see is undone, and
//! the other two stay, documented:
//!
//! - [`AircraftDescription`] carries the wake turbulence category again
//!   ([`WakeTurbulenceCategory`]). au-casa had left it out because no Part 61
//!   rule uses it, which would make a shared table quietly incomplete. It is
//!   optional, since no row has had it compiled yet.
//! - [`AirframeKind`] carries `Glider` and `Airship`, which are not among the
//!   letters of an aircraft description, so that a glider and an airship can
//!   be told apart from other airframes.
//! - [`EngineType::is_gas_turbine`] says which kinds of engine are gas
//!   turbines; it is there because Part 61's design features (reg 61.755) ask.

use serde::{Deserialize, Serialize};

/// What sort of airframe a type is: the first letter of an aircraft
/// description.
///
/// `Glider` and `Airship` are not among those letters. They are carried so
/// that a glider and an airship can be described at all: CASA's Part 61, for
/// one, needs to reach the airship category (reg 61.015(e)) and registered
/// sailplanes (reg 61.007(2)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AirframeKind {
    /// `L`: landplane.
    LandPlane,
    /// `S`: seaplane.
    SeaPlane,
    /// `A`: amphibian.
    Amphibian,
    /// `H`: helicopter.
    Helicopter,
    /// `G`: gyrocopter.
    Gyrocopter,
    /// `T`: tiltrotor.
    TiltRotor,
    /// Not one of the letters: see the enum's own documentation.
    Glider,
    /// Not one of the letters: see the enum's own documentation.
    Airship,
}

/// The kind of engine a type has: the third element of an aircraft
/// description.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EngineType {
    /// `P`: piston.
    Piston,
    /// `T`: turboprop or turboshaft.
    TurboProp,
    /// `J`: jet.
    Jet,
    /// No engine: a glider.
    None,
}

impl EngineType {
    /// Whether this kind of engine is a gas turbine, as CASA's reg 61.755
    /// `gas turbine engine` design feature asks.
    pub fn is_gas_turbine(self) -> bool {
        matches!(self, Self::TurboProp | Self::Jet)
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
    /// `None` until it has been compiled for the type. Left out of the JSON
    /// when `None`, so a description serialised before it was carried reads
    /// the same.
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
