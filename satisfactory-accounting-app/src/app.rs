// Copyright 2021, 2022 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, HashMap};
use std::mem;
use std::rc::Rc;

use gloo::storage::errors::StorageError;
use gloo::storage::{LocalStorage, Storage};
use log::warn;
use uuid::Uuid;
use yew::prelude::*;

use satisfactory_accounting::accounting::Node;
use satisfactory_accounting::database::{Database, DatabaseVersion};

use crate::appheader::AppHeader;
use crate::node_display::{NodeDisplay, NodeMeta, NodeMetadata};
use crate::user_settings::{UserSettingsManager, UserSettingsWindowManager};

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum OverlayWindow {
    #[default]
    None,
    WorldChooser,
    DatabaseChooser,
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
    /// Show or hide one of the overlay windows. If the current window is already this window, show
    /// None. Otherwise show this window.
    ToggleWindow(OverlayWindow),
}

/// Current state of the app.
pub struct App {
    /// Overlay window to show.
    overlay_window: OverlayWindow,
    /// World with a "confirm delete" window currently present.
    pending_delete: Option<WorldId>,
    /// Whether to show deprecated database versions in the list.
    show_deprecated_databases: bool,

    // Cached Callbacks.
    undo: Callback<()>,
    redo: Callback<()>,
    toggle_world_chooser: Callback<()>,
    toggle_db_chooser: Callback<()>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link();

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
                let id = WorldId::new();
                let world = World::try_load_v1();
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

        Self {
            overlay_window: OverlayWindow::None,
            pending_delete: None,
            show_deprecated_databases: false,
            worlds,
            world,
            database,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),

            undo: link.callback(|()| Msg::Undo),
            redo: link.callback(|()| Msg::Redo),
            toggle_world_chooser: link
                .callback(|()| Msg::ToggleWindow(OverlayWindow::WorldChooser)),
            toggle_db_chooser: link
                .callback(|()| Msg::ToggleWindow(OverlayWindow::DatabaseChooser)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
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
            Msg::ToggleWindow(overlay) => {
                if self.pending_delete.is_some() {
                    // If there is a pending delete, clear it and switch to the specified window.
                    // Requires redraw because pending_delete changed.
                    self.pending_delete = None;
                    self.overlay_window = overlay;
                    true
                } else if self.overlay_window == overlay {
                    // Otherwise, if we're already on the specified window, close it. If the window
                    // requested was None and we're already on None, do nothing.
                    let changed = self.overlay_window == OverlayWindow::None;
                    self.overlay_window = OverlayWindow::None;
                    changed
                } else {
                    // If we're not already on the specified window, switch to it and redraw.
                    self.overlay_window = overlay;
                    true
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
        let move_node =
            Callback::from(|_| warn!("Root node tried to ask parent to move one of its children"));
        html! {
            <UserSettingsManager>
            <ContextProvider<Rc<Database>> context={Rc::clone(&self.database)}>
            <ContextProvider<NodeMetadata> context={self.world.node_metadata.clone()}>
            <div class="App">
                <UserSettingsWindowManager>
                {self.app_header()}
                </UserSettingsWindowManager>
                // TODO: hide empty balances.
                <div class={classes!("appbody")}>
                    <NodeDisplay node={self.world.root.clone()}
                        path={Vec::new()}
                        {replace} {set_metadata} {batch_set_metadata}
                        {move_node} />
                </div>
                { self.world_chooser(ctx) }
                { self.database_chooser(ctx) }
                if let Some(pending) = self.pending_delete {
                    { self.confirm_delete(ctx, pending) }
                }
            </div>
            </ContextProvider<NodeMetadata>>
            </ContextProvider<Rc<Database>>>
            </UserSettingsManager>
        }
    }
}

impl App {
    fn app_header(&self) -> Html {
        let db_choice = &self.world.database;

        let on_choose_world = &self.toggle_world_chooser;
        let on_undo = (!self.undo_stack.is_empty()).then_some(self.undo.clone());
        let on_redo = (!self.redo_stack.is_empty()).then_some(self.redo.clone());
        let on_choose_db = &self.toggle_db_chooser;
        html! {
            <AppHeader {db_choice} {on_choose_world} {on_undo} {on_redo} {on_choose_db} />
        }
    }

    fn world_chooser(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let close = link.callback(|_| Msg::ToggleWindow(OverlayWindow::None));
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
        let close = link.callback(|_| Msg::ToggleWindow(OverlayWindow::None));
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
}
