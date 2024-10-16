use std::cell::Cell;
use std::collections::btree_map::{Entry, Iter, IterMut, OccupiedEntry, VacantEntry};
use std::collections::BTreeMap;
use std::iter::FusedIterator;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use log::warn;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use yew::AttrValue;

use crate::world::{DatabaseVersionSelector, WorldId};

/// Info about a particular world. Used in the world map to avoid needing to load the
/// whole world to get info about it.
#[derive(PartialEq, Clone, Serialize, Deserialize)]
pub struct WorldMetadata {
    /// Name of the world.
    pub name: AttrValue,
    /// Version of the database used by this world, if known.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub database: Option<DatabaseVersionSelector>,
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
    pub fn selected_id(&self) -> WorldId {
        self.inner.selected
    }

    /// Get the currently selected world.
    pub fn get_selected(&self) -> Option<WorldMetaRef> {
        self.get(self.selected_id())
    }

    /// Get the currently selected world.
    pub fn get_selected_mut(&mut self) -> Option<WorldMetaMut> {
        self.get_mut(self.selected_id())
    }

    /// Get an entry for the currently selected world.
    pub fn selected_entry(&mut self) -> WorldEntry {
        self.entry(self.selected_id())
    }

    /// Remove the world with the given ID and return it.
    pub fn remove(&mut self, id: WorldId) -> Result<WorldMetadata, RemoveWorldError> {
        if self.inner.selected == id {
            return Err(RemoveWorldError::CurrentlySelected);
        }
        let worlds = &mut Rc::make_mut(&mut self.inner).worlds;
        worlds.remove(&id).ok_or(RemoveWorldError::NotFound)
    }

    /// Get the metadata for a particular world, if it exists.
    pub fn get(&self, id: WorldId) -> Option<WorldMetaRef> {
        self.inner.worlds.get(&id).map(|meta| WorldMetaRef {
            selected: &self.inner.selected,
            id,
            meta,
        })
    }

    /// Get the metadata for a particular world, if it exists.
    pub fn get_mut(&mut self, id: WorldId) -> Option<WorldMetaMut> {
        let inner = Rc::make_mut(&mut self.inner);
        inner.worlds.get_mut(&id).map(|meta| WorldMetaMut {
            selected: Cell::from_mut(&mut inner.selected),
            id,
            meta,
        })
    }

    /// Choose a new world ID and get a VacantEntry pointing to it.
    pub fn allocate_new_id(&mut self) -> AbsentWorld {
        let inner = Rc::make_mut(&mut self.inner);
        const MAX_ATTEMPTS: usize = 10;
        let mut chosen_id = None;
        for attempt in 0..MAX_ATTEMPTS {
            let new_id = WorldId::new();
            if inner.worlds.contains_key(&new_id) {
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
            Some(chosen_id) => match inner.worlds.entry(chosen_id) {
                Entry::Occupied(_) => panic!(
                    "Entry for {chosen_id} was occupied even though we just checked that the \
                        map did not contain that key"
                ),
                Entry::Vacant(entry) => AbsentWorld {
                    selected: &mut inner.selected,
                    entry,
                },
            },
        }
    }

    /// Gets the entry for the world with the given id.
    pub fn entry(&mut self, id: WorldId) -> WorldEntry {
        let inner = Rc::make_mut(&mut self.inner);
        let selected = &mut inner.selected;
        match inner.worlds.entry(id) {
            Entry::Occupied(entry) => WorldEntry::Present(PresentWorld { selected, entry }),
            Entry::Vacant(entry) => WorldEntry::Absent(AbsentWorld { selected, entry }),
        }
    }

    /// Gets an iterator over the world list.
    pub fn iter(&self) -> WorldListIter {
        WorldListIter {
            selected: &self.inner.selected,
            inner: self.inner.worlds.iter(),
        }
    }

    /// Gets a mutable iterator over the world list.
    pub fn iter_mut(&mut self) -> WorldListIterMut {
        let inner = Rc::make_mut(&mut self.inner);
        WorldListIterMut {
            selected: Cell::from_mut(&mut inner.selected),
            inner: inner.worlds.iter_mut(),
        }
    }
}

impl<'a> IntoIterator for &'a WorldList {
    type IntoIter = WorldListIter<'a>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut WorldList {
    type IntoIter = WorldListIterMut<'a>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[derive(PartialEq, Clone, Serialize, Deserialize)]
struct WorldListInner {
    /// Mapping of worlds by ID.
    worlds: BTreeMap<WorldId, WorldMetadata>,
    /// ID of the currently selected world.
    selected: WorldId,
}

/// Error cases for removing a world from the world list.
#[derive(Error, Debug, Copy, Clone)]
pub enum RemoveWorldError {
    /// Did not remove the world because it was not found.
    #[error("World to remove not found in the world list")]
    NotFound,
    /// Could not remove the world because it was currently selected.
    #[error("World to remove was currently selected")]
    CurrentlySelected,
}

/// Iterator over the world list.
pub struct WorldListIter<'a> {
    /// Reference to the currently selected world.
    selected: &'a WorldId,
    /// Iterator from the inner BTreeMap.
    inner: Iter<'a, WorldId, WorldMetadata>,
}

impl<'a> WorldListIter<'a> {
    fn map_iter(&self, (&id, meta): (&'a WorldId, &'a WorldMetadata)) -> WorldMetaRef<'a> {
        WorldMetaRef {
            selected: self.selected,
            id,
            meta,
        }
    }
}

impl<'a> Iterator for WorldListIter<'a> {
    type Item = WorldMetaRef<'a>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|e| self.map_iter(e))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth(n).map(|e| self.map_iter(e))
    }

    fn last(mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|e| self.map_iter(e))
    }
}

impl<'a> DoubleEndedIterator for WorldListIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|e| self.map_iter(e))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth_back(n).map(|e| self.map_iter(e))
    }
}

impl<'a> ExactSizeIterator for WorldListIter<'a> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> FusedIterator for WorldListIter<'a> {}

/// Iterator over the world list.
pub struct WorldListIterMut<'a> {
    /// Reference to the currently selected world.
    selected: &'a Cell<WorldId>,
    /// Iterator from the inner BTreeMap.
    inner: IterMut<'a, WorldId, WorldMetadata>,
}

impl<'a> WorldListIterMut<'a> {
    fn map_iter(&self, (&id, meta): (&'a WorldId, &'a mut WorldMetadata)) -> WorldMetaMut<'a> {
        WorldMetaMut {
            selected: self.selected,
            id,
            meta,
        }
    }
}

impl<'a> Iterator for WorldListIterMut<'a> {
    type Item = WorldMetaMut<'a>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|e| self.map_iter(e))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth(n).map(|e| self.map_iter(e))
    }

    fn last(mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|e| self.map_iter(e))
    }
}

impl<'a> DoubleEndedIterator for WorldListIterMut<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|e| self.map_iter(e))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth_back(n).map(|e| self.map_iter(e))
    }
}

impl<'a> ExactSizeIterator for WorldListIterMut<'a> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> FusedIterator for WorldListIterMut<'a> {}

/// Reference to a world's metadata.
pub struct WorldMetaRef<'a> {
    /// Reference to the currently-selected world.
    selected: &'a WorldId,
    /// Id of this world.
    id: WorldId,
    /// Reference to the world's metadata.
    meta: &'a WorldMetadata,
}

impl<'a> WorldMetaRef<'a> {
    /// Get the ID of this world.
    #[inline]
    pub fn id(&self) -> WorldId {
        self.id
    }

    /// Returns true if this world is the selected world.
    pub fn is_selected(&self) -> bool {
        *self.selected == self.id()
    }

    /// Get a reference to the world's metadata.
    #[inline]
    pub fn meta(&self) -> &WorldMetadata {
        self.meta
    }
}

impl<'a> Deref for WorldMetaRef<'a> {
    type Target = WorldMetadata;

    fn deref(&self) -> &Self::Target {
        self.meta()
    }
}

/// Mutable reference to a world's metadata.
pub struct WorldMetaMut<'a> {
    /// Shared mutable reference to the currently-selected world.
    selected: &'a Cell<WorldId>,
    /// Id of this world.
    id: WorldId,
    /// Reference to the world's metadata.
    meta: &'a mut WorldMetadata,
}

impl<'a> WorldMetaMut<'a> {
    /// Get the ID of this world.
    #[inline]
    pub fn id(&self) -> WorldId {
        self.id
    }

    /// Returns true if this world is the selected world.
    pub fn is_selected(&self) -> bool {
        self.selected.get() == self.id()
    }

    /// Make this world the selected world.
    pub fn select(&mut self) {
        self.selected.set(self.id);
    }

    /// Get a reference to the world's metadata.
    #[inline]
    pub fn meta(&self) -> &WorldMetadata {
        self.meta
    }

    /// Get a mutable reference to the world's metadata.
    #[inline]
    pub fn meta_mut(&mut self) -> &mut WorldMetadata {
        self.meta
    }
}

impl<'a> Deref for WorldMetaMut<'a> {
    type Target = WorldMetadata;

    fn deref(&self) -> &Self::Target {
        self.meta()
    }
}

impl<'a> DerefMut for WorldMetaMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.meta_mut()
    }
}

/// Entry for a World.
pub enum WorldEntry<'a> {
    Present(PresentWorld<'a>),
    Absent(AbsentWorld<'a>),
}

impl<'a> WorldEntry<'a> {
    /// Returns true if this world has an entry in the world list, otherwise false if it is an empty
    /// entry.
    pub fn exists(&self) -> bool {
        match self {
            Self::Present(_) => true,
            Self::Absent(_) => false,
        }
    }

    /// Gets the id of this entry
    pub fn id(&self) -> WorldId {
        match self {
            Self::Present(entry) => entry.id(),
            Self::Absent(entry) => entry.id(),
        }
    }

    /// Inserts the value if missing or updates the value if it exists and makes this world the
    /// selected world.
    pub fn insert_or_update_and_select(self, meta: WorldMetadata) {
        match self {
            Self::Present(mut entry) => {
                *entry.meta_mut() = meta;
                entry.select();
            }
            Self::Absent(entry) => entry.insert_and_select(meta),
        }
    }
}

/// Entry for a world that is present.
pub struct PresentWorld<'a> {
    /// Backref to the world list's selected status.
    selected: &'a mut WorldId,
    /// Entry for the existing world.
    entry: OccupiedEntry<'a, WorldId, WorldMetadata>,
}

impl<'a> PresentWorld<'a> {
    /// Gets the ID assigned to the world.
    pub fn id(&self) -> WorldId {
        *self.entry.key()
    }

    /// Get an immutable reference to the metadata for the world.
    pub fn meta(&self) -> &WorldMetadata {
        self.entry.get()
    }

    /// Gets a mutable reference to the metadata for the world.
    pub fn meta_mut(&mut self) -> &mut WorldMetadata {
        self.entry.get_mut()
    }

    /// Return true if this world is the selected world.
    pub fn is_selected(&self) -> bool {
        *self.selected == self.id()
    }

    /// Makes this world the selected world. Also clears any load_error.
    pub fn select(&mut self) {
        *self.selected = self.id();
        self.meta_mut().load_error = false;
    }
}

/// Entry for a world that doesn't exist in the world list.
pub struct AbsentWorld<'a> {
    /// Backref to the world list's selected status.
    selected: &'a mut WorldId,
    /// Entry for the missing world.
    entry: VacantEntry<'a, WorldId, WorldMetadata>,
}

impl<'a> AbsentWorld<'a> {
    /// Gets the ID assigned to the world.
    pub fn id(&self) -> WorldId {
        *self.entry.key()
    }

    /// Insert the world with the given metadata and make it the selected world.
    pub fn insert_and_select(self, meta: WorldMetadata) {
        let id = self.id();
        self.entry.insert(meta);
        *self.selected = id;
    }
}
