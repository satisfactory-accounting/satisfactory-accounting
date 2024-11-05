use serde::{Deserialize, Serialize};

use crate::world::{World, WorldId};

/// Format used for downloadable world save files.
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveFile {
    /// ID of the world.
    #[serde(default)]
    world_id: Option<WorldId>,
    /// The world model with version tag.
    #[serde(flatten)]
    versioned_model: VersionedWorldModel,
}

impl SaveFile {
    /// Create a new save file from the given world, using the current world model version.
    pub fn new(id: WorldId, world: World) -> Self {
        Self {
            world_id: Some(id),
            versioned_model: VersionedWorldModel::Version1Minor2(world),
        }
    }

    /// Get the world ID, if one was set in the file.
    pub fn id(&self) -> Option<WorldId> {
        self.world_id
    }

    /// Extracts the versioned world model.
    pub fn into_versioned_model(self) -> VersionedWorldModel {
        self.versioned_model
    }
}

/// Identifies the different world model versions we support.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "model_version")]
pub enum VersionedWorldModel {
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
