use std::hash::Hasher;

use derive_more::Display;
use getset::{Getters, MutGetters, Setters, WithSetters};
use serde::{Deserialize, Serialize};

#[derive(
    Serialize, Deserialize, Clone, Display, Default, Getters, MutGetters, Setters, WithSetters,
)]
#[display("{callsign} {name}")]
#[getset(get = "pub", get_mut = "pub", set = "pub", set_with = "pub")]
pub struct Callsign {
    /// The individual callsign, or if they do not have one allocated, a
    /// callsign-like reference, for example their forename.
    callsign: String,
    /// The individual's name.
    name: String,
}

impl std::cmp::PartialEq for Callsign {
    fn eq(&self, other: &Self) -> bool {
        self.callsign == other.callsign
    }
}

impl std::cmp::Eq for Callsign {}

impl std::hash::Hash for Callsign {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.callsign.hash(state);
    }
}

#[cfg(feature = "escpos")]
#[derive(Copy, Clone, PartialEq)]
pub struct ESCPOSDevice(pub u16, pub u16);
