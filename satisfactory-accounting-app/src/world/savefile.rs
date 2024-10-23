use serde::{Deserialize, Serialize};

use crate::world::World;

/// Format used for downloadable world save files.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "model_version")]
pub enum SaveFile {
    /// World model used in the 1.2.x series of releases.
    #[allow(non_camel_case_types)]
    #[serde(rename = "v1.2.*")]
    Version1Minor2(World),
    /// Variant that gets deserialized if the model version isn't recognized.
    ///
    /// This variant is for deserialization error handling and generally should not be intentionally
    /// serialized.
    #[serde(untagged)]
    Unknown {
        /// The model version of the file that was deserialzied, if any was provided.
        #[serde(default)]
        model_version: Option<String>,
    },
}
