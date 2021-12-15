use std::mem;
use std::rc::Rc;

use gloo::storage::{LocalStorage, Storage};
use log::warn;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

use satisfactory_accounting::accounting::{BuildNode, Group, Node};
use satisfactory_accounting::database::Database;

use crate::node_display::NodeDisplay;

/// Key that the app state is stored under.
const KEY: &str = "zstewart.satisfactorydb.state";

/// Stored state of the app.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

impl Default for AppState {
    fn default() -> Self {
        let database = Rc::new(Database::load_default());
        let root = Group::default().build_node(&database).unwrap();
        Self { database, root }
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
        let state = LocalStorage::get(KEY).unwrap_or_default();
        Self {
            state,
            ..Default::default()
        }
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
                self.save();
                true
            }
            Msg::Undo => match self.undo_stack.pop() {
                Some(previous) => {
                    let next = mem::replace(&mut self.state, previous);
                    self.redo_stack.push(next);
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
                    true
                }
                None => {
                    warn!("Nothing to redo");
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
        let undo = link.callback(|_| Msg::Undo);
        let redo = link.callback(|_| Msg::Redo);
        html! {
            <ContextProvider<Rc<Database>> context={Rc::clone(&self.state.database)}>
                <div class="App">
                    <div class="navbar">
                        <div class="appheader">{"SATISFACTORY ACCOUNTING"}</div>
                    </div>
                    <div class="menubar">
                        <button class="unredo" onclick={undo}
                            disabled={self.undo_stack.is_empty()}>
                            <span class="material-icons">{"undo"}</span>
                        </button>
                        <button class="unredo" onclick={redo}
                            disabled={self.redo_stack.is_empty()}>
                            <span class="material-icons">{"redo"}</span>
                        </button>
                    </div>
                    <div class="appbody">
                        <NodeDisplay node={self.state.root.clone()}
                            path={Vec::new()}
                            {replace} />
                    </div>
                </div>
            </ContextProvider<Rc<Database>>>
        }
    }
}

impl App {
    /// Save the current state to local storage.
    fn save(&self) {
        if let Err(e) = LocalStorage::set(KEY, &self.state) {
            warn!("Unable to save state: {}", e);
        }
    }
}
