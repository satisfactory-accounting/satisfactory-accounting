use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Unique ID of a world.
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct WorldId(Uuid);

impl WorldId {
    /// Creates a new random world ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Error from parsing a [`WorldId`].
#[derive(Error, Debug)]
pub enum ParseWorldIdError {
    #[error("ID did not start with zstewart.satisfactorydb.state.world.")]
    IncorrectPrefix,
    #[error("Parsing suffix as uuid failed")]
    InvalidUuid(#[from] uuid::Error),
}

impl FromStr for WorldId {
    type Err = ParseWorldIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const PREFIX: &str = "zstewart.satisfactorydb.state.world.";
        if s.starts_with(PREFIX) {
            Ok(WorldId(s[PREFIX.len()..].parse()?))
        } else {
            Err(ParseWorldIdError::IncorrectPrefix)
        }
    }
}

impl fmt::Display for WorldId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "zstewart.satisfactorydb.state.world.{}",
            self.0.as_simple()
        )
    }
}

impl Serialize for WorldId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for WorldId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct WorldIdVisitor;
        impl<'de> serde::de::Visitor<'de> for WorldIdVisitor {
            type Value = WorldId;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a string of the format \"zstewart.satisfactorydb.state.world.{uuid}\"")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                WorldId::from_str(v)
                    .map_err(|_| E::invalid_value(serde::de::Unexpected::Str(v), &self))
            }
        }

        deserializer.deserialize_str(WorldIdVisitor)
    }
}
