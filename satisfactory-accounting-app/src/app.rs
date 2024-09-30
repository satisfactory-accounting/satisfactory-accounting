// Copyright 2021, 2022 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use std::borrow::Cow;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::str::FromStr;
use std::{fmt, mem};

use gloo::storage::errors::StorageError;
use gloo::storage::{LocalStorage, Storage};
use log::warn;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use yew::prelude::*;

use satisfactory_accounting::accounting::{Group, Node};
use satisfactory_accounting::database::{Database, DatabaseVersion};

use crate::node_display::{BalanceSortMode, NodeDisplay, NodeMeta, NodeMetadata};

/// Key that the app state is stored under.
const DB_KEY: &str = "zstewart.satisfactorydb.state.database";
const GRAPH_KEY: &str = "zstewart.satisfactorydb.state.graph";
const METADATA_KEY: &str = "zstewart.satisfactorydb.state.metadata";
const GLOBAL_METADATA_KEY: &str = "zstewart.satisfactorydb.state.globalmetadata";

const WORLD_MAP_KEY: &str = "zstewart.satisfactorydb.state.world";
const USER_SETTINGS_KEY: &str = "zstewart.satisfactorydb.usersettings";

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum OverlayWindow {
    #[default]
    None,
    WorldChooser,
    DatabaseChooser,
    UserSettings,
}

/// App-wide settings specific to the user rather than the world.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserSettings {
    /// Whether empty balance values should be hidden.
    pub hide_empty_balances: bool,
    pub balance_sort_mode: BalanceSortMode,
}

impl UserSettings {
    /// Load from LocalStorage if possible.
    fn load() -> Result<Self, StorageError> {
        LocalStorage::get(USER_SETTINGS_KEY)
    }

    /// Save the current user settings.
    fn save(&self) {
        if let Err(e) = LocalStorage::set(USER_SETTINGS_KEY, &self) {
            warn!("Unable to save world: {}", e);
        }
    }
}

/// Unique ID of a world.
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct WorldId(Uuid);

impl WorldId {
    fn new() -> Self {
        Self(Uuid::new_v4())
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

/// Info about a particular world. Used in the world map to avoid needing to load the
/// whole world to get info about it.
#[derive(Serialize, Deserialize)]
struct WorldMetadata {
    /// Name of the world.
    name: AttrValue,
    /// If we attempted to load this world this session but it failed, it is flagged here.
    #[serde(skip, default)]
    load_error: bool,
}

/// Mapping of different worlds.
#[derive(Serialize, Deserialize)]
struct Worlds {
    /// Mapping of worlds by ID.
    worlds: BTreeMap<WorldId, WorldMetadata>,
    /// ID of the currently selected world.
    selected: WorldId,
}

impl Worlds {
    /// Load the worlds mapping or return the default.
    fn load() -> Result<Self, StorageError> {
        LocalStorage::get(WORLD_MAP_KEY)
    }

    /// Save the world mapping.
    fn save(&self) {
        if let Err(e) = LocalStorage::set(WORLD_MAP_KEY, self) {
            warn!("Unable to save metadata: {}", e);
        }
    }
}

/// The choice of database for a particular world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseChoice {
    /// Use one of the standard databases.
    Standard(DatabaseVersion),
    /// This world uses a custom database.
    Custom(Rc<Database>),
}

impl DatabaseChoice {
    /// Get the database for this database choice.
    fn get(&self) -> Rc<Database> {
        match *self {
            DatabaseChoice::Standard(version) => Rc::new(version.load_database()),
            DatabaseChoice::Custom(ref db) => Rc::clone(db),
        }
    }

    /// Return true if this is a standard database with the specified version.
    fn is_standard_version(&self, version: DatabaseVersion) -> bool {
        match *self {
            DatabaseChoice::Standard(v) => v == version,
            _ => false,
        }
    }
}

impl Default for DatabaseChoice {
    fn default() -> Self {
        DatabaseChoice::Standard(DatabaseVersion::LATEST)
    }
}

/// A single world with a particular database and structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct World {
    /// Which database is used for this world.
    database: DatabaseChoice,
    /// Root node for this world.
    root: Node,
    /// Non-undo metadata about nodes.
    node_metadata: NodeMetadata,
    /// Non-undo metadata about this particular world.
    global_metadata: GlobalMetadata,
}

impl World {
    /// Updates the Root Node and returns an undo state that will go back to the previous
    /// root. If the new root has a different name from the current root, returns a string
    /// of the new name.
    fn update_root(&mut self, root: Node) -> (UnReDoState, Option<AttrValue>) {
        assert!(root.group().is_some(), "new root was not a group");
        let new_name = {
            let new_name = &root.group().unwrap().name;
            let old_name = &self.root.group().unwrap().name;
            if new_name != old_name {
                Some(new_name.clone())
            } else {
                None
            }
        };

        let old = mem::replace(&mut self.root, root);
        (
            UnReDoState {
                database: self.database.clone(),
                root: old,
            },
            new_name,
        )
    }

    /// Updates the database and root from the given undo state and returns an undo state
    /// that will go back to the state before applying the undo.
    fn apply_undo_state(&mut self, state: UnReDoState) -> UnReDoState {
        UnReDoState {
            root: mem::replace(&mut self.root, state.root),
            database: mem::replace(&mut self.database, state.database),
        }
    }

    /// Load from LocalStorage, if possible.
    fn load(id: WorldId) -> Result<Self, StorageError> {
        let mut world: Self = LocalStorage::get(id.to_string())?;
        // Remove metadata from deleted groups that are definitely no longer in the
        // undo/redo history.
        world.node_metadata.prune(&world.root);
        Ok(world)
    }

    /// Try to load a V1 world, replacing any missing components with defaults.
    fn try_load_v1() -> Self {
        let database = match LocalStorage::get::<Database>(DB_KEY) {
            Ok(mut database) => {
                // All databases in the DB_KEY should be pre-U6 which means they shouldn't
                // have an icon prefix, and we can set the icon prefix to u5, unless for
                // some reason it's already set.
                if database.icon_prefix.is_empty() {
                    database.icon_prefix = "u5/".to_string();
                }
                DatabaseVersion::ALL
                    .iter()
                    .find_map(|&version| match version.load_database() {
                        db if database.compare_ignore_prefix(&db) => {
                            Some(DatabaseChoice::Standard(version))
                        }
                        _ => None,
                    })
                    .unwrap_or_else(move || DatabaseChoice::Custom(Rc::new(database)))
            }
            Err(e) => {
                if !matches!(e, StorageError::KeyNotFound(_)) {
                    warn!("Failed to load database: {}", e);
                }
                DatabaseChoice::default()
            }
        };
        let root = LocalStorage::get(GRAPH_KEY).unwrap_or_else(|e| {
            if !matches!(e, StorageError::KeyNotFound(_)) {
                warn!("Failed to load graph: {}", e);
            }
            Group::empty_node()
        });
        let mut metadata: NodeMetadata = LocalStorage::get(METADATA_KEY).unwrap_or_else(|e| {
            if !matches!(e, StorageError::KeyNotFound(_)) {
                warn!("Failed to load metadata: {}", e);
            }
            Default::default()
        });
        metadata.prune(&root);
        let global_metadata: GlobalMetadata = LocalStorage::get(GLOBAL_METADATA_KEY)
            .unwrap_or_else(|e| {
                if !matches!(e, StorageError::KeyNotFound(_)) {
                    warn!("Failed to load global metadata: {}", e);
                }
                Default::default()
            });

        World {
            database,
            root,
            node_metadata: metadata,
            global_metadata,
        }
    }

    /// Create a new empty world with the default database version.
    fn new() -> Self {
        World {
            database: Default::default(),
            root: Group::empty_node(),
            node_metadata: Default::default(),
            global_metadata: Default::default(),
        }
    }

    /// Get the name of this world.
    fn name(&self) -> AttrValue {
        match self.root.group() {
            Some(root) => root.name.clone(),
            None => {
                warn!("Cannot get world name: root was not a group!");
                "".into()
            }
        }
    }

    /// Gets metadata for this world.
    fn storage_metadata(&self) -> WorldMetadata {
        WorldMetadata {
            name: self.name(),
            load_error: false,
        }
    }

    /// Save the state of the current world.
    fn save(&self, id: WorldId) {
        if let Err(e) = LocalStorage::set(id.to_string(), &self) {
            warn!("Unable to save world: {}", e);
        }
    }
}

/// Metadata about a particular world.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GlobalMetadata {
    /// Whether to hide or show empty balances in group balances.
    ///
    /// This field has been moved to [`UserSettings`]. The field here is only used for
    /// backwards compatibility, so when migrating to v1.2.0 or later from an earlier
    /// version we can pull the user's hide_empty_balances setting from the GlobalMetadata
    /// of the selected world.
    #[deprecated]
    hide_empty_balances: bool,
}

/// State tracked for undo/redo.
struct UnReDoState {
    /// Database at this undo/redo version.
    database: DatabaseChoice,
    /// Root node of the world at this version.
    root: Node,
}

/// Messages for communicating with App.
pub enum Msg {
    ReplaceRoot {
        replacement: Node,
    },
    UpdateMetadata {
        id: Uuid,
        meta: NodeMeta,
    },
    /// Apply multiple metadata updates in a single step without saving in between.
    BatchUpdateMetadata {
        updates: HashMap<Uuid, NodeMeta>,
    },
    ToggleEmptyBalances {
        hide_empty_balances: bool,
    },
    /// Change to the specified balance sort mode.
    SetBalanceSortMode {
        sort_mode: BalanceSortMode,
    },
    Undo,
    Redo,
    /// Set the database to the given database choice.
    SetDb(DatabaseChoice),
    /// Set the status of show_deprecated_databases.
    ShowDeprecated(bool),
    /// Select a particular world.
    SetWorld(WorldId),
    /// Create a new world.
    CreateWorld,
    /// Initiate deleting a world.
    InitiateDelete(WorldId),
    /// Cancel deleting a world.
    CancelDelete,
    /// Permanently delete the world with the given ID.
    DeleteForever(WorldId),
    /// Show or hide one of the overlay windows.
    SetWindow(OverlayWindow),
}

/// Current state of the app.
pub struct App {
    /// Current user's global settings.
    user_settings: Rc<UserSettings>,
    /// Overlay window to show.
    overlay_window: OverlayWindow,
    /// World with a "confirm delete" window currently present.
    pending_delete: Option<WorldId>,
    /// Whether to show deprecated database versions in the list.
    show_deprecated_databases: bool,
    /// Listing of available worlds.
    worlds: Worlds,
    /// State of the currently selected world.
    world: World,
    /// Selected database.
    database: Rc<Database>,
    /// Stack of previous states for undo.
    undo_stack: Vec<UnReDoState>,
    /// Stack of future states for redo.
    redo_stack: Vec<UnReDoState>,
}

impl App {
    /// Add a state to the Undo stack, clearing the redo stack and any history beyond 100
    /// items.
    fn add_undo_state(&mut self, previous_state: UnReDoState) {
        self.undo_stack.push(previous_state);
        if self.undo_stack.len() > 100 {
            let num_to_remove = self.undo_stack.len() - 100;
            self.undo_stack.drain(..num_to_remove);
        }
        self.redo_stack.clear();
    }

    /// Saves the world into the
    fn save_world(&self) {
        self.world.save(self.worlds.selected);
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let (worlds, world) = match Worlds::load() {
            Ok(mut worlds) => {
                let world = match World::load(worlds.selected) {
                    Ok(world) => world,
                    Err(e) => {
                        if let Some(world) = worlds.worlds.get_mut(&worlds.selected) {
                            world.load_error = true;
                        }
                        warn!("Failed to load selected world: {}", e);
                        worlds.selected = WorldId::new();
                        let world = World::new();
                        match worlds.worlds.entry(worlds.selected) {
                            Entry::Occupied(_) => {
                                panic!("Created a duplicate UUID {}", worlds.selected);
                            }
                            Entry::Vacant(entry) => {
                                entry.insert(world.storage_metadata());
                            }
                        }
                        worlds.save();
                        world.save(worlds.selected);
                        world
                    }
                };
                (worlds, world)
            }
            Err(e) => {
                if !matches!(e, StorageError::KeyNotFound(_)) {
                    warn!("Failed to load world list: {}", e);
                }
                let mut worlds = Worlds {
                    worlds: BTreeMap::new(),
                    selected: WorldId::new(),
                };
                let world = World::try_load_v1();
                worlds
                    .worlds
                    .insert(worlds.selected, world.storage_metadata());
                worlds.save();
                world.save(worlds.selected);
                (worlds, world)
            }
        };
        let database = world.database.get();

        let user_settings = Rc::new(match UserSettings::load() {
            Ok(settings) => settings,
            Err(e) => {
                if !matches!(e, StorageError::KeyNotFound(_)) {
                    warn!("Failed to load user settings: {}", e);
                }
                let settings = UserSettings {
                    // Intentionally using the deprecated field to pull the old value into
                    // the new, non-deprecated field.
                    #[allow(deprecated)]
                    hide_empty_balances: world.global_metadata.hide_empty_balances,
                    ..Default::default()
                };
                settings.save();
                settings
            }
        });

        Self {
            user_settings,
            overlay_window: OverlayWindow::None,
            pending_delete: None,
            show_deprecated_databases: false,
            worlds,
            world,
            database,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ReplaceRoot { replacement } => {
                let (previous, new_name) = self.world.update_root(replacement);
                self.add_undo_state(previous);
                if let Some(new_name) = new_name {
                    match self.worlds.worlds.entry(self.worlds.selected) {
                        Entry::Occupied(mut entry) => entry.get_mut().name = new_name,
                        Entry::Vacant(entry) => {
                            warn!("World {} was not in the worlds map", self.worlds.selected);
                            entry.insert(self.world.storage_metadata());
                        }
                    }
                    self.worlds.save();
                }
                self.save_world();
                true
            }
            Msg::UpdateMetadata { id, meta } => {
                self.world.node_metadata.set_meta(id, meta);
                self.save_world();
                true
            }
            Msg::BatchUpdateMetadata { updates } => {
                if updates.is_empty() {
                    false
                } else {
                    self.world.node_metadata.batch_update(updates.into_iter());
                    self.save_world();
                    true
                }
            }
            Msg::ToggleEmptyBalances {
                hide_empty_balances,
            } => {
                Rc::make_mut(&mut self.user_settings).hide_empty_balances = hide_empty_balances;
                self.user_settings.save();
                true
            }
            Msg::SetBalanceSortMode { sort_mode }
                if self.user_settings.balance_sort_mode != sort_mode =>
            {
                Rc::make_mut(&mut self.user_settings).balance_sort_mode = sort_mode;
                self.user_settings.save();
                true
            }
            Msg::SetBalanceSortMode { sort_mode: _ } => false,
            Msg::Undo => match self.undo_stack.pop() {
                Some(previous) => {
                    let next = self.world.apply_undo_state(previous);
                    self.redo_stack.push(next);
                    self.save_world();
                    true
                }
                None => {
                    warn!("Nothing to undo");
                    false
                }
            },
            Msg::Redo => match self.redo_stack.pop() {
                Some(next) => {
                    let previous = self.world.apply_undo_state(next);
                    self.undo_stack.push(previous);
                    self.save_world();
                    true
                }
                None => {
                    warn!("Nothing to redo");
                    false
                }
            },
            Msg::SetDb(database) => {
                self.database = database.get();
                let previous = UnReDoState {
                    database: mem::replace(&mut self.world.database, database),
                    root: {
                        let new_root = self.world.root.rebuild(&*self.database);
                        mem::replace(&mut self.world.root, new_root)
                    },
                };
                self.add_undo_state(previous);
                self.save_world();
                true
            }
            Msg::ShowDeprecated(show_deprecated) => {
                if self.show_deprecated_databases != show_deprecated {
                    self.show_deprecated_databases = show_deprecated;
                    self.overlay_window == OverlayWindow::DatabaseChooser
                } else {
                    false
                }
            }
            Msg::SetWorld(world_id) => {
                if !self.worlds.worlds.contains_key(&world_id) {
                    warn!("Unknown world {world_id}");
                    false
                } else if world_id == self.worlds.selected {
                    false
                } else {
                    match World::load(world_id) {
                        Ok(world) => {
                            self.worlds.selected = world_id;
                            self.world = world;
                            self.database = self.world.database.get();
                            self.undo_stack.clear();
                            self.redo_stack.clear();
                            self.worlds.save();
                            true
                        }
                        Err(e) => {
                            warn!("Unable to load world {world_id}: {e}");
                            if let Some(world) = self.worlds.worlds.get_mut(&world_id) {
                                world.load_error = true;
                                true
                            } else {
                                false
                            }
                        }
                    }
                }
            }
            Msg::CreateWorld => {
                let new_id = WorldId::new();
                let world = World::new();
                world.save(new_id);
                self.worlds.worlds.insert(new_id, world.storage_metadata());
                self.worlds.selected = new_id;
                self.world = world;
                self.database = self.world.database.get();
                self.undo_stack.clear();
                self.redo_stack.clear();
                self.worlds.save();
                true
            }
            Msg::InitiateDelete(id) => {
                if self.pending_delete != Some(id) {
                    self.pending_delete = Some(id);
                    true
                } else {
                    false
                }
            }
            Msg::CancelDelete => {
                if self.pending_delete.is_some() {
                    self.pending_delete = None;
                    true
                } else {
                    false
                }
            }
            Msg::DeleteForever(id) => {
                if Some(id) != self.pending_delete {
                    warn!("Requested delete did not match pending delete");
                    self.pending_delete = None;
                    return true;
                }
                self.pending_delete = None;
                self.worlds.worlds.remove(&id);
                LocalStorage::delete(id.to_string());
                if self.worlds.selected == id {
                    for &id in self.worlds.worlds.keys() {
                        match World::load(id) {
                            Ok(world) => {
                                self.worlds.selected = id;
                                self.world = world;
                                self.database = self.world.database.get();
                                self.undo_stack.clear();
                                self.redo_stack.clear();
                                self.worlds.save();
                                return true;
                            }
                            Err(e) => {
                                warn!("Unable to load world {id}: {e}");
                            }
                        }
                    }
                    // Either there are no existing worlds or all worlds failed to load.
                    let new_id = WorldId::new();
                    let world = World::new();
                    world.save(new_id);
                    self.worlds.worlds.insert(new_id, world.storage_metadata());
                    self.worlds.selected = new_id;
                    self.world = world;
                    self.database = self.world.database.get();
                    self.undo_stack.clear();
                    self.redo_stack.clear();
                    self.worlds.save();
                }
                true
            }
            Msg::SetWindow(overlay) => {
                if self.pending_delete.is_some() {
                    self.pending_delete = None;
                    self.overlay_window = overlay;
                    return true;
                }
                if self.overlay_window != overlay {
                    self.overlay_window = overlay;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let replace = link.callback(|(idx, replacement)| {
            assert!(idx == 0, "Attempting to replace index {} at the root", idx);
            Msg::ReplaceRoot { replacement }
        });
        let set_metadata = link.callback(|(id, meta)| Msg::UpdateMetadata { id, meta });
        let batch_set_metadata = link.callback(|updates| Msg::BatchUpdateMetadata { updates });
        let chooseworld = if self.overlay_window == OverlayWindow::WorldChooser {
            link.callback(|_| Msg::SetWindow(OverlayWindow::None))
        } else {
            link.callback(|_| Msg::SetWindow(OverlayWindow::WorldChooser))
        };
        let undo = link.callback(|_| Msg::Undo);
        let redo = link.callback(|_| Msg::Redo);
        let choosedb = if self.overlay_window == OverlayWindow::DatabaseChooser {
            link.callback(|_| Msg::SetWindow(OverlayWindow::None))
        } else {
            link.callback(|_| Msg::SetWindow(OverlayWindow::DatabaseChooser))
        };
        let move_node =
            Callback::from(|_| warn!("Root node tried to ask parent to move one of its children"));

        let settings = if self.overlay_window == OverlayWindow::UserSettings {
            link.callback(|_| Msg::SetWindow(OverlayWindow::None))
        } else {
            link.callback(|_| Msg::SetWindow(OverlayWindow::UserSettings))
        };

        let hide_empty_balances = self.user_settings.hide_empty_balances;
        let toggle_empty_balances = link.callback(move |_| Msg::ToggleEmptyBalances {
            hide_empty_balances: !hide_empty_balances,
        });
        let hidden_balances = hide_empty_balances.then(|| "hide-empty-balances");
        html! {
            <ContextProvider<Rc<Database>> context={Rc::clone(&self.database)}>
            <ContextProvider<Rc<UserSettings>> context={Rc::clone(&self.user_settings)}>
            <ContextProvider<NodeMetadata> context={self.world.node_metadata.clone()}>
            <div class="App">
                <div class="navbar">
                    <div class="appheader">{"SATISFACTORY ACCOUNTING"}</div>
                </div>
                <div class="menubar">
                    <span class="section">
                        <button class="open-world" title="Choose World" onclick={chooseworld}>
                            <span class="material-icons">{"folder_open"}</span>
                        </button>
                        <button class="unredo" title="Undo"
                            onclick={undo}
                            disabled={self.undo_stack.is_empty()}>
                            <span class="material-icons">{"undo"}</span>
                        </button>
                        <button class="unredo" title="Redo"
                            onclick={redo}
                            disabled={self.redo_stack.is_empty()}>
                            <span class="material-icons">{"redo"}</span>
                        </button>
                        <button class="choose-database" title="Choose Database" onclick={choosedb}>
                            <span class="material-icons">{"factory"}</span>
                            <span>{self.name_db()}</span>
                        </button>
                        <label class="empty-balance-toggle" title="Show/Hide Zero Balances">
                            <input type="checkbox" checked={hide_empty_balances}
                                onchange={toggle_empty_balances} />
                            <span class="material-icons">{"exposure_zero"}</span>
                            if hide_empty_balances {
                                <span class="material-icons">{"visibility_off"}</span>
                            } else {
                                <span class="material-icons">{"visibility"}</span>
                            }
                        </label>
                    </span>
                    <span class="section">
                        <button class="settings" title="Settings" onclick={settings}>
                            <span class="material-icons">{"settings"}</span>
                        </button>
                        <a class="bug-report" target="_blank"
                            href="https://github.com/satisfactory-accounting/satisfactory-accounting/issues">
                            <span class="material-icons">
                                {"bug_report"}
                            </span>
                        </a>
                    </span>
                </div>
                <div class={classes!("appbody", hidden_balances)}>
                    <NodeDisplay node={self.world.root.clone()}
                        path={Vec::new()}
                        {replace} {set_metadata} {batch_set_metadata}
                        {move_node} />
                </div>
                { self.world_chooser(ctx) }
                { self.database_chooser(ctx) }
                { self.user_settings_window(ctx) }
                if let Some(pending) = self.pending_delete {
                    { self.confirm_delete(ctx, pending) }
                }
            </div>
            </ContextProvider<NodeMetadata>>
            </ContextProvider<Rc<UserSettings>>>
            </ContextProvider<Rc<Database>>>
        }
    }
}

impl App {
    fn name_db(&self) -> Cow<'static, str> {
        match self.world.database {
            DatabaseChoice::Standard(version) => {
                if version.is_deprecated() {
                    Cow::Owned(format!("{version} \u{2013} Update Available!"))
                } else {
                    Cow::Borrowed(version.name())
                }
            }
            DatabaseChoice::Custom(_) => Cow::Borrowed("Custom"),
        }
    }

    fn world_chooser(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let close = link.callback(|_| Msg::SetWindow(OverlayWindow::None));
        let new = link.callback(|_| Msg::CreateWorld);

        let worlds = self.worlds.worlds.iter().map(|(&id, meta)| {
            let load_error = meta.load_error;
            let open = link.callback(move |_| Msg::SetWorld(id));
            let delete = link.callback(move |_| Msg::InitiateDelete(id));
            html! {
                <div class="world-list-row">
                    <span>{&meta.name}</span>
                    <span class="right-buttons">
                        <button class="delete-world" title="Delete World" onclick={delete}>
                            <span class="material-icons">{"delete"}</span>
                        </button>
                        if load_error {
                            <span class="BuildError material-icons error" title="Unable to load this world in this version of Satisfactory Accounting">
                                {"warning"}
                            </span>
                        } else {
                            <button class="new-world" title="Switch to this World" onclick={open}>
                                <span class="material-icons">{"open_in_browser"}</span>
                            </button>
                        }
                    </span>
                </div>
            }
        });
        let hidden = match self.overlay_window {
            OverlayWindow::WorldChooser => None,
            _ => Some("hide"),
        };
        html! {
            <div class={classes!("overlay-window", hidden)}>
                <div class="close-bar">
                    <h3>{"Choose World"}</h3>
                    <button class="close" title="Close" onclick={close}>
                        <span class="material-icons">{"close"}</span>
                    </button>
                </div>
                <div class="world-list">
                    <div class="world-list-row">
                        <span>{"Create New"}</span>
                        <button class="new-world" title="Create New World" onclick={new}>
                            <span class="material-icons">{"add"}</span>
                        </button>
                    </div>
                    { for worlds }
                </div>
            </div>
        }
    }

    fn database_chooser(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let close = link.callback(|_| Msg::SetWindow(OverlayWindow::None));
        let show_deprecated = self.show_deprecated_databases;
        let toggle_deprecated = link.callback(move |_| Msg::ShowDeprecated(!show_deprecated));

        let databases = DatabaseVersion::ALL
            .iter()
            .filter(|version| show_deprecated || !version.is_deprecated())
            .map(|version| {
                let deprecated = if version.is_deprecated() {
                    Some("deprecated")
                } else {
                    None
                };
                let choose_db = link.callback(|_| Msg::SetDb(DatabaseChoice::Standard(*version)));
                html! {
                    <div class={classes!("database-list-row", deprecated)}>
                        <div class="version-namedesc">
                            <span class="version-name">{version.name()}</span>
                            <span class="version-description">{version.description()}</span>
                        </div>
                        <button class="choose-db" title="Select this Version" onclick={choose_db}>
                            <span class="material-icons">{
                                if self.world.database.is_standard_version(*version) {
                                    "radio_button_checked"
                                } else {
                                    "radio_button_unchecked"
                                }
                            }</span>
                        </button>
                    </div>
                }
            });
        let hidden = match self.overlay_window {
            OverlayWindow::DatabaseChooser => None,
            _ => Some("hide"),
        };
        html! {
            <div class={classes!("overlay-window", hidden)}>
                <div class="close-bar">
                    <h3>{"Choose Database"}</h3>
                    <span class="right-buttons">
                        <button class="show-deprecated" title="Show Deprecated Versions" onclick={toggle_deprecated}>
                            <span>{"Deprecated Versions"}</span>
                            <span class="material-icons">{
                                if self.show_deprecated_databases {
                                    "visibility"
                                } else {
                                    "visibility_off"
                                }
                            }</span>
                        </button>
                        <button class="close" title="Close" onclick={close}>
                            <span class="material-icons">{"close"}</span>
                        </button>
                    </span>
                </div>
                <div class="database-list">
                    { for databases }
                </div>
            </div>
        }
    }

    fn confirm_delete(&self, ctx: &Context<Self>, id: WorldId) -> Html {
        let link = ctx.link();
        let cancel = link.callback(|_| Msg::CancelDelete);
        let delete = link.callback(move |_| Msg::DeleteForever(id));

        let name = match self.worlds.worlds.get(&id) {
            Some(meta) if !meta.name.is_empty() => &meta.name,
            Some(_) => "<Empty Name>",
            None => "<Not Found>",
        };
        html! {
            <div class="overlay-delete-window">
                <h2>{"Are you sure you want to delete World "}{name}</h2>
                <h3>{"This CANNOT be undone!"}</h3>
                <div class="button-row">
                    <button class="cancel" title="Cancel" onclick={cancel}>
                        <span>{"Cancel"}</span>
                        <span class="material-icons">{"arrow_back"}</span>
                    </button>
                    <button class="delete-forever" title="Delete Forever" onclick={delete}>
                        <span>{"Delete"}</span>
                        <span class="material-icons">{"delete_forever"}</span>
                    </button>
                </div>
            </div>
        }
    }

    /// Display the user settings window. This is always displayed and is hidden in CSS
    /// when not needed.
    fn user_settings_window(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let close = link.callback(|_| Msg::SetWindow(OverlayWindow::None));

        let hide_empty_balances = self.user_settings.hide_empty_balances;
        let toggle_empty_balances = link.callback(move |_| Msg::ToggleEmptyBalances {
            hide_empty_balances: !hide_empty_balances,
        });

        let sort_by_item = link.callback(move |_| Msg::SetBalanceSortMode {
            sort_mode: BalanceSortMode::Item,
        });

        let sort_by_ioitem = link.callback(move |_| Msg::SetBalanceSortMode {
            sort_mode: BalanceSortMode::IOItem,
        });

        let hidden = match self.overlay_window {
            OverlayWindow::UserSettings => None,
            _ => Some("hide"),
        };
        html! {
            <div class={classes!("overlay-window", "user-settings", hidden)}>
                <div class="close-bar">
                    <h3>{"Settings"}</h3>
                    <button class="close" title="Close" onclick={close}>
                        <span class="material-icons">{"close"}</span>
                    </button>
                </div>
                <div class="settings-list">
                    <span class="setting-row toggle" onclick={toggle_empty_balances}>
                        <span>{"Hide Empty Balances"}</span>
                        <span class="material-icons">{
                            if hide_empty_balances {
                                "check_box"
                            } else {
                                "check_box_outline_blank"
                            }
                        }</span>
                    </span>
                    <div class="setting-group">
                        <h4>{"Balance Sort Mode"}</h4>
                        <span class="setting-row toggle" onclick={sort_by_item}>
                            <span>{"Sort by item"}</span>
                            <span class="material-icons">{
                                if self.user_settings.balance_sort_mode == BalanceSortMode::Item {
                                    "radio_button_checked"
                                } else {
                                    "radio_button_unchecked"
                                }
                            }</span>
                        </span>
                        <span class="setting-row toggle" onclick={sort_by_ioitem}>
                            <span>{"Sort by inputs vs outputs, then by item"}</span>
                            <span class="material-icons">{
                                if self.user_settings.balance_sort_mode == BalanceSortMode::IOItem {
                                    "radio_button_checked"
                                } else {
                                    "radio_button_unchecked"
                                }
                            }</span>
                        </span>
                    </div>
                </div>
            </div>
        }
    }
}
