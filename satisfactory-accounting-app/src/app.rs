use std::rc::Rc;

use gloo::storage::{LocalStorage, Storage};
use log::info;
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

pub enum Msg {
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
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        // This gives us a component's "`Scope`" which allows us to send messages, etc to the component.
        html! {
            <ContextProvider<Rc<Database>> context={Rc::clone(&self.state.database)}>
                <div class="App">
                    <div class="navbar">
                        <div class="appheader">{"SATISFACTORY ACCOUNTING"}</div>
                    </div>
                    <div class="appbody">
                        <NodeDisplay node={self.state.root.clone()}
                            path={Vec::new()} />
                    </div>
                </div>
            </ContextProvider<Rc<Database>>>
        }
    }
}
