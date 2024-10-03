use std::collections::btree_map::{Entry, VacantEntry};
use std::collections::BTreeMap;
use std::rc::Rc;

use log::warn;
use satisfactory_accounting::database::DatabaseVersion;
use serde::{Deserialize, Serialize};
use yew::AttrValue;

use crate::world::WorldId;

/// Info about a particular world. Used in the world map to avoid needing to load the
/// whole world to get info about it.
#[derive(PartialEq, Clone, Serialize, Deserialize)]
pub struct WorldMetadata {
    /// Name of the world.
    pub name: AttrValue,
    /// Version of the database used by this world, if known.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub database: Option<DatabaseVersion>,
    /// If we attempted to load this world this session but it failed, it is flagged here.
    /// This is not serialized in order to allow it to be retried next time the app is opened.
    #[serde(skip, default)]
    pub load_error: bool,
}

/// Mapping of different worlds.
#[derive(PartialEq, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorldList {
    /// Shared inner world list.
    inner: Rc<WorldListInner>,
}

impl WorldList {
    /// Creates a new world list with the given world/metadata as the selected value.
    pub(super) fn new(selected: WorldId, meta: WorldMetadata) -> Self {
        let mut worlds = BTreeMap::new();
        worlds.insert(selected, meta);
        Self {
            inner: Rc::new(WorldListInner { worlds, selected }),
        }
    }

    /// Get the ID of the currently selected world.
    pub fn selected(&self) -> WorldId {
        self.inner.selected
    }

    /// Get the metadata for a particular world, if it exists.
    pub fn get_mut(&mut self, id: WorldId) -> Option<&mut WorldMetadata> {
        Rc::make_mut(&mut self.inner).worlds.get_mut(&id)
    }

    /// Choose a new world ID and get a VacantEntry pointing to it.
    pub fn allocate_new_id(&mut self) -> VacantEntry<WorldId, WorldMetadata> {
        let worlds = &mut Rc::make_mut(&mut self.inner).worlds;
        const MAX_ATTEMPTS: usize = 10;
        let mut chosen_id = None;
        for attempt in 0..MAX_ATTEMPTS {
            let new_id = WorldId::new();
            if worlds.contains_key(&new_id) {
                warn!(
                    "Tried to allocate id {new_id} which is already occupied. (Attempt {attempt})"
                );
            } else {
                chosen_id = Some(new_id);
                break;
            }
        }
        match chosen_id {
            None => panic!("Failed to allocate a new WorldId {MAX_ATTEMPTS} times in a row!"),
            Some(chosen_id) => match worlds.entry(chosen_id) {
                Entry::Occupied(_) => panic!(
                    "Entry for {chosen_id} was occupied even though we just checked that the \
                        map did not contain that key"
                ),
                Entry::Vacant(entry) => entry,
            },
        }
    }

    /// Gets the entry for the world with the given id.
    pub fn entry(&mut self, id: WorldId) -> Entry<WorldId, WorldMetadata> {
        Rc::make_mut(&mut self.inner).worlds.entry(id)
    }
}

#[derive(PartialEq, Clone, Serialize, Deserialize)]
struct WorldListInner {
    /// Mapping of worlds by ID.
    worlds: BTreeMap<WorldId, WorldMetadata>,
    /// ID of the currently selected world.
    selected: WorldId,
}
