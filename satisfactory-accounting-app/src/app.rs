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
}

pub struct App {
    state: AppState,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let state = LocalStorage::get(KEY).unwrap_or_default();
        Self { state }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ReplaceRoot { replacement } => {
                self.state.root = replacement;
                self.save();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let replace = ctx.link().callback(|(idx, replacement)| {
            assert!(idx == 0, "Attempting to replace index {} at the root", idx);
            Msg::ReplaceRoot { replacement }
        });
        html! {
            <ContextProvider<Rc<Database>> context={Rc::clone(&self.state.database)}>
                <div class="App">
                    <div class="navbar">
                        <div class="appheader">{"SATISFACTORY ACCOUNTING"}</div>
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
