use std::mem;
use std::rc::Rc;

use gloo::storage::errors::StorageError;
use gloo::storage::{LocalStorage, Storage};
use log::warn;
use yew::prelude::*;

use satisfactory_accounting::accounting::{Group, Node};
use satisfactory_accounting::database::Database;

use crate::node_display::NodeDisplay;

/// Key that the app state is stored under.
const DB_KEY: &str = "zstewart.satisfactorydb.state.database";
const GRAPH_KEY: &str = "zstewart.satisfactorydb.state.graph";

/// Stored state of the app.
#[derive(Debug, Clone)]
struct AppState {
    /// Database used in the app previously.
    database: Rc<Database>,
    /// Root node of the accounting tree.
    root: Node,
}

impl AppState {
    /// Updates AppState and returns the previous version.
    fn update_root(&mut self, root: Node) -> Self {
        let old_root = mem::replace(&mut self.root, root);
        Self {
            database: self.database.clone(),
            root: old_root,
        }
    }

    /// Load AppState from LocalStorage, or create state if it can't be loaded.
    fn load_or_create() -> Self {
        let database = Rc::new(LocalStorage::get(DB_KEY).unwrap_or_else(|e| {
            if !matches!(e, StorageError::KeyNotFound(_)) {
                warn!("Failed to load database: {}", e);
            }
            Database::load_default()
        }));
        let root = LocalStorage::get(GRAPH_KEY).unwrap_or_else(|e| {
            if !matches!(e, StorageError::KeyNotFound(_)) {
                warn!("Failed to load graph: {}", e);
            }
            Group::default().into()
        });
        Self { database, root }
    }

    /// Save the current app state.
    fn save(&self) {
        if let Err(e) = LocalStorage::set(DB_KEY, &self.database) {
            warn!("Unable to save database: {}", e);
        }
        if let Err(e) = LocalStorage::set(GRAPH_KEY, &self.root) {
            warn!("Unable to save database: {}", e);
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::load_or_create()
    }
}

/// Messages for communicating with App.
pub enum Msg {
    ReplaceRoot { replacement: Node },
    Undo,
    Redo,
}

#[derive(Default)]
pub struct App {
    state: AppState,
    undo_stack: Vec<AppState>,
    redo_stack: Vec<AppState>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Default::default()
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ReplaceRoot { replacement } => {
                self.undo_stack.push(self.state.update_root(replacement));
                if self.undo_stack.len() > 100 {
                    let num_to_remove = self.undo_stack.len() - 100;
                    self.undo_stack.drain(..num_to_remove);
                }
                self.redo_stack.clear();
                self.state.save();
                true
            }
            Msg::Undo => match self.undo_stack.pop() {
                Some(previous) => {
                    let next = mem::replace(&mut self.state, previous);
                    self.redo_stack.push(next);
                    self.state.save();
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
                    self.state.save();
                    true
                }
                None => {
                    warn!("Nothing to redo");
                    false
                }
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let replace = link.callback(|(idx, replacement)| {
            assert!(idx == 0, "Attempting to replace index {} at the root", idx);
            Msg::ReplaceRoot { replacement }
        });
        let undo = link.callback(|_| Msg::Undo);
        let redo = link.callback(|_| Msg::Redo);
        let move_node =
            Callback::from(|_| warn!("Root node tried to ask parent to move one of its children"));
        html! {
            <ContextProvider<Rc<Database>> context={Rc::clone(&self.state.database)}>
                <div class="App">
                    <div class="navbar">
                        <div class="appheader">{"SATISFACTORY ACCOUNTING"}</div>
                    </div>
                    <div class="menubar">
                        <button class="unredo" title="undo"
                            onclick={undo}
                            disabled={self.undo_stack.is_empty()}>
                            <span class="material-icons">{"undo"}</span>
                        </button>
                        <button class="unredo" title="redo"
                            onclick={redo}
                            disabled={self.redo_stack.is_empty()}>
                            <span class="material-icons">{"redo"}</span>
                        </button>
                    </div>
                    <div class="appbody">
                        <NodeDisplay node={self.state.root.clone()}
                            path={Vec::new()}
                            {replace}
                            {move_node} />
                    </div>
                </div>
            </ContextProvider<Rc<Database>>>
        }
    }
}
