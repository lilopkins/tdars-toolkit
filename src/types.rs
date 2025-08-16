use derive_more::Display;
use getset::{Getters, MutGetters, Setters, WithSetters};
use serde::{Deserialize, Serialize};

#[derive(
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    Display,
    Default,
    Getters,
    MutGetters,
    Setters,
    WithSetters,
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
