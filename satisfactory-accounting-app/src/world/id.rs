use std::fmt;
use std::str::FromStr;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::{DecodeSliceError, Engine};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::fmt::Simple;
use uuid::Uuid;

/// Formatter type which formats a WorldId without any prefix, just the raw UUID.
pub type Unprefixed = Simple;

/// Unique ID of a world.
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct WorldId(Uuid);

impl WorldId {
    /// Creates a new random world ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get a formatter that formats the underlying UUID without the path prefix.
    pub fn as_unprefixed(&self) -> &Unprefixed {
        self.0.as_simple()
    }

    /// Get a formatter that formats the world ID in the legacy
    /// zstewart.satisfactroydb.state.world.{uuids} format.
    pub fn as_legacy_dotted(&self) -> AsLegacyDotted {
        AsLegacyDotted { id: self }
    }

    /// Get a formatter that formats the world ID in the new worlds/{base64uuid} format.
    pub fn as_resource_id(&self) -> AsResourceId {
        AsResourceId { id: self }
    }
}

/// Error from parsing a [`WorldId`].
#[derive(Error, Debug)]
pub enum ParseWorldIdError {
    #[error("ID did not start with \"zstewart.satisfactorydb.state.world.\" or \"worlds/\"")]
    IncorrectPrefix,
    #[error(
        "ID was the wrong number of bytes. Expected 22 bytes of base64 data (for 16 bytes of \
        uuid), but got {0} bytes of base64."
    )]
    IncorrectDataLen(usize),
    #[error("Parsing suffix as uuid failed: {0}")]
    InvalidUuid(#[from] uuid::Error),
    #[error("Parsing base64 Uuid failed: {0}")]
    InvalidBase64(#[from] base64::DecodeError),
}

const LEGACY_DOTTED_PREFIX: &str = "zstewart.satisfactorydb.state.world.";
const RESOURCE_ID_PREFIX: &str = "worlds/";

impl FromStr for WorldId {
    type Err = ParseWorldIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with(LEGACY_DOTTED_PREFIX) {
            Ok(WorldId(s[LEGACY_DOTTED_PREFIX.len()..].parse()?))
        } else if s.starts_with(RESOURCE_ID_PREFIX) {
            let mut data = uuid::Bytes::default();
            match URL_SAFE_NO_PAD.decode_slice(&s[RESOURCE_ID_PREFIX.len()..], &mut data) {
                Ok(decoded) if decoded < data.len() => {
                    return Err(ParseWorldIdError::IncorrectDataLen(
                        s.len() - RESOURCE_ID_PREFIX.len(),
                    ));
                }
                Err(DecodeSliceError::OutputSliceTooSmall) => {
                    return Err(ParseWorldIdError::IncorrectDataLen(
                        s.len() - RESOURCE_ID_PREFIX.len(),
                    ));
                }
                Err(DecodeSliceError::DecodeError(e)) => {
                    return Err(ParseWorldIdError::InvalidBase64(e));
                }
                Ok(_) => {}
            }
            Ok(WorldId(Uuid::from_bytes(data)))
        } else {
            Err(ParseWorldIdError::IncorrectPrefix)
        }
    }
}

impl Serialize for WorldId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.as_resource_id())
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
                f.write_str(
                    "a string of the format \"zstewart.satisfactorydb.state.world.{uuid}\" or \
                    \"worlds/{base64uuid}\"",
                )
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

impl fmt::Debug for WorldId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.as_resource_id(), self.as_unprefixed())
    }
}

/// Formats the WorldId as a legacy dotted id.
#[repr(transparent)]
pub struct AsLegacyDotted<'a> {
    id: &'a WorldId,
}

impl<'a> fmt::Display for AsLegacyDotted<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{LEGACY_DOTTED_PREFIX}{}", self.id.as_unprefixed())
    }
}

/// Formats the WorldId as a base64 resource ID.
#[repr(transparent)]
pub struct AsResourceId<'a> {
    id: &'a WorldId,
}

impl<'a> fmt::Display for AsResourceId<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use base64::display::Base64Display;
        let id = Base64Display::new(self.id.0.as_bytes(), &URL_SAFE_NO_PAD);
        write!(f, "{RESOURCE_ID_PREFIX}{id}")
    }
}
