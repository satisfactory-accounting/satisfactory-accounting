// Copyright 2021, 2022 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use std::collections::HashMap;
use std::mem;
use std::rc::Rc;

use gloo::storage::errors::StorageError;
use gloo::storage::{LocalStorage, Storage};
use log::warn;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use yew::prelude::*;

use satisfactory_accounting::accounting::{Group, Node};
use satisfactory_accounting::database::Database;

use crate::node_display::{NodeDisplay, NodeMeta, NodeMetadata};

/// Key that the app state is stored under.
const DB_KEY: &str = "zstewart.satisfactorydb.state.database";
const GRAPH_KEY: &str = "zstewart.satisfactorydb.state.graph";
const METADATA_KEY: &str = "zstewart.satisfactorydb.state.metadata";
const GLOBAL_METADATA_KEY: &str = "zstewart.satisfactorydb.state.globalmetadata";

/// Stored state of the app.
#[derive(Debug, Clone)]
struct AppState {
    /// Database used in the app previously.
    database: Rc<Database>,
    /// Root node of the accounting tree.
    root: Node,
    /// Cached value tracking whether the database is out of date, so we don't have to
    /// repeatedly compare the database.
    database_outdated: bool,
}

impl AppState {
    /// Updates AppState and returns the previous version.
    fn update_root(&mut self, root: Node) -> Self {
        let old_root = mem::replace(&mut self.root, root);
        Self {
            root: old_root,
            ..self.clone()
        }
    }

    /// Load AppState from LocalStorage, or create state if it can't be loaded.
    fn load_or_create() -> Self {
        let default = Database::load_default();
        let (database, database_outdated) = match LocalStorage::get(DB_KEY) {
            Ok(database) => {
                let database_outdated = database != default;
                (Rc::new(database), database_outdated)
            }
            Err(e) => {
                if !matches!(e, StorageError::KeyNotFound(_)) {
                    warn!("Failed to load database: {}", e);
                }
                (Rc::new(default), false)
            }
        };
        let root = LocalStorage::get(GRAPH_KEY).unwrap_or_else(|e| {
            if !matches!(e, StorageError::KeyNotFound(_)) {
                warn!("Failed to load graph: {}", e);
            }
            Group::empty().into()
        });
        Self {
            database,
            root,
            database_outdated,
        }
    }

    /// Save the current app state.
    fn save(&self) {
        if let Err(e) = LocalStorage::set(DB_KEY, &self.database) {
            warn!("Unable to save database: {}", e);
        }
        if let Err(e) = LocalStorage::set(GRAPH_KEY, &self.root) {
            warn!("Unable to save graph: {}", e);
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GlobalMetadata {
    /// Whether empty balance values should be hidden.
    hide_empty_balances: bool,
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
    Undo,
    Redo,
    UpdateDb,
}

pub struct App {
    state: AppState,
    /// Non-undo metadata about nodes.
    metadata: NodeMetadata,
    /// Non-undo metadata about the global app state.
    global_metadata: GlobalMetadata,
    undo_stack: Vec<AppState>,
    redo_stack: Vec<AppState>,
}

impl App {
    fn save(&self) {
        self.state.save();
        if let Err(e) = LocalStorage::set(METADATA_KEY, &self.metadata) {
            warn!("Unable to save metadata: {}", e);
        }
        if let Err(e) = LocalStorage::set(GLOBAL_METADATA_KEY, &self.global_metadata) {
            warn!("Unable to save global metadata: {}", e);
        }
    }

    /// Add a state to the Undo stack, clearing the redo stack and any history beyond 100
    /// items.
    fn add_undo_state(&mut self, previous_state: AppState) {
        self.undo_stack.push(previous_state);
        if self.undo_stack.len() > 100 {
            let num_to_remove = self.undo_stack.len() - 100;
            self.undo_stack.drain(..num_to_remove);
        }
        self.redo_stack.clear();
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let state = AppState::load_or_create();
        let mut metadata: NodeMetadata = LocalStorage::get(METADATA_KEY).unwrap_or_else(|e| {
            if !matches!(e, StorageError::KeyNotFound(_)) {
                warn!("Failed to load metadata: {}", e);
            }
            Default::default()
        });
        // Remove metadata from deleted groups that are definitely no longer in the
        // undo/redo history.
        metadata.prune(&state.root);
        let global_metadata: GlobalMetadata = LocalStorage::get(GLOBAL_METADATA_KEY)
            .unwrap_or_else(|e| {
                if !matches!(e, StorageError::KeyNotFound(_)) {
                    warn!("Failed to load global metadata: {}", e);
                }
                Default::default()
            });
        Self {
            state,
            metadata,
            global_metadata,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ReplaceRoot { replacement } => {
                let previous = self.state.update_root(replacement);
                self.add_undo_state(previous);
                self.save();
                true
            }
            Msg::UpdateMetadata { id, meta } => {
                self.metadata.set_meta(id, meta);
                self.save();
                true
            }
            Msg::BatchUpdateMetadata { updates } => {
                if updates.is_empty() {
                    false
                } else {
                    self.metadata.batch_update(updates.into_iter());
                    self.save();
                    true
                }
            }
            Msg::ToggleEmptyBalances {
                hide_empty_balances,
            } => {
                self.global_metadata.hide_empty_balances = hide_empty_balances;
                self.save();
                true
            }
            Msg::Undo => match self.undo_stack.pop() {
                Some(previous) => {
                    let next = mem::replace(&mut self.state, previous);
                    self.redo_stack.push(next);
                    self.save();
                    true
                }
                None => {
                    warn!("Nothing to undo");
                    false
                }
            },
            Msg::Redo => match self.redo_stack.pop() {
                Some(next) => {
                    let previous = mem::replace(&mut self.state, next);
                    self.undo_stack.push(previous);
                    self.save();
                    true
                }
                None => {
                    warn!("Nothing to redo");
                    false
                }
            },
            Msg::UpdateDb => {
                let mut new_state = self.state.clone();
                new_state.database = Rc::new(Database::load_default());
                new_state.database_outdated = false;
                new_state.root = self.state.root.rebuild(&*new_state.database);
                let previous = mem::replace(&mut self.state, new_state);
                self.add_undo_state(previous);
                self.save();
                true
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
        let undo = link.callback(|_| Msg::Undo);
        let redo = link.callback(|_| Msg::Redo);
        let update_db = link.callback(|_| Msg::UpdateDb);
        let move_node =
            Callback::from(|_| warn!("Root node tried to ask parent to move one of its children"));

        let hide_empty_balances = self.global_metadata.hide_empty_balances;
        let toggle_empty_balances = link.callback(move |_| Msg::ToggleEmptyBalances {
            hide_empty_balances: !hide_empty_balances,
        });
        let hidden_balances = hide_empty_balances.then(|| "hide-empty-balances");
        html! {
            <ContextProvider<Rc<Database>> context={Rc::clone(&self.state.database)}>
                <ContextProvider<NodeMetadata> context={self.metadata.clone()}>
                    <div class="App">
                        <div class="navbar">
                            <div class="appheader">{"SATISFACTORY ACCOUNTING"}</div>
                        </div>
                        <div class="menubar">
                            <span class="section">
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
                                if self.state.database_outdated {
                                    <button class="update-db" onclick={update_db}
                                        title="Update the database of structures and recipes. This could break existing buildings (but you *can* undo this).">
                                        <span class="material-icons">
                                            {"browser_updated"}
                                        </span>
                                    </button>
                                }
                            </span>
                            <a class="bug-report" target="_blank"
                                href="https://github.com/satisfactory-accounting/satisfactory-accounting/issues">
                                <span class="material-icons">
                                    {"bug_report"}
                                </span>
                            </a>
                        </div>
                        <div class={classes!("appbody", hidden_balances)}>
                            <NodeDisplay node={self.state.root.clone()}
                                path={Vec::new()}
                                {replace} {set_metadata} {batch_set_metadata}
                                {move_node} />
                        </div>
                    </div>
                </ContextProvider<NodeMetadata>>
            </ContextProvider<Rc<Database>>>
        }
    }
}
