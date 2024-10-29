use std::rc::Rc;

use crate::database::BuildingId;

mod clock;
mod multiplier;

/// Config settings for a building.
#[derive(Debug, Clone, PartialEq)]
pub struct BuildingSettings {
    inner: Rc<BuildingSettingsInner>,
}

/// Shareable inner building.
#[derive(Debug, Clone, PartialEq)]
struct BuildingSettingsInner {
    /// ID of the building from the database which this building is configuring.
    building_id: BuildingId,
}
