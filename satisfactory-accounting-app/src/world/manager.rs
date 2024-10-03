use std::collections::btree_map::Entry;
use std::collections::VecDeque;
use std::mem;
use std::rc::Rc;

use gloo::storage::errors::StorageError;
use gloo::storage::{LocalStorage, Storage as _};
use log::warn;
use satisfactory_accounting::accounting::Node;
use satisfactory_accounting::database::{Database, DatabaseVersion};
use uuid::Uuid;
use yew::{html, Callback, Component, Context, Html, Properties};

use crate::user_settings::UserSettingsDispatcher;
use crate::world::{v1storage, DatabaseChoice, NodeMetadatum, WorldId};
use crate::world::{World, WorldList};

#[derive(PartialEq, Properties)]
pub struct Props {
    /// Children, which will have access to the world and the world manager's various context
    /// handles.
    pub children: Html,
}

pub enum Msg {
    /// Set a new Root node for the World.
    SetRoot { root: Node },
    /// Update the metadatum of a single node.
    UpdateNodeMeta {
        /// UUID of the node to update the metadata of.
        id: Uuid,
        /// Meta for the individual node being updated.
        meta: NodeMetadatum,
    },
    /// Change the most recent undo state, pushing the current state to the redo stack.
    Undo,
    /// Change to the most recent redo state, pushing the current state to the undo stack.
    Redo,
    /// Switch to the specified DatabaseVersion.
    SetDb(DatabaseVersion),

    /// Change to the specified World ID.
    SetWorld(WorldId),
    /// Permanently delete the world with the given ID.
    DeleteWorld(WorldId),
}

/// World manager manages the lifecycle of a single world.
pub struct WorldManager {
    /// List of available worlds.
    worlds: WorldList,

    /// Current state of the world.
    world: World,
    /// Currently selected database.
    database: Rc<Database>,
    /// Stack of previous states for undo.
    undo_stack: VecDeque<UnReDoState>,
    /// Stack of future states for redo.
    redo_stack: VecDeque<UnReDoState>,
}

impl WorldManager {
    /// Applies an undo state or a redo state to the world and returns an [`UnReDoState`] which will
    /// return to the previous state.
    fn apply_undo_state(&mut self, state: UnReDoState) -> UnReDoState {
        let prior_state = UnReDoState {
            root: mem::replace(&mut self.world.root, state.root),
            database: mem::replace(&mut self.world.database, state.database),
        };
        if self.world.database != prior_state.database {
            self.database = self.world.database.get();
        }
        prior_state
    }

    /// Add an undo state, clearing the redo states.
    fn add_undo_state(&mut self, state: UnReDoState) {
        self.redo_stack.clear();
        if self.undo_stack.len() >= MAX_UNDO {
            // Remove all items beyond MAX_UNDO as well as 1 additional item to make room to push
            // without going over.
            let to_remove = self.undo_stack.len() - MAX_UNDO + 1;
            if to_remove > 1 {
                warn!(
                    "Undo stack grew larger than {MAX_UNDO} items: {}",
                    self.undo_stack.len()
                );
            }
            self.undo_stack.drain(..to_remove);
        }
        self.undo_stack.push_back(state);
    }

    /// Message handler for SetRoot. Returns true if redraw is needed.
    fn set_root(&mut self, new_root: Node) -> bool {
        assert!(
            new_root.group().is_some(),
            "new root {new_root:?} was not a group"
        );
        // Update the world state, tracking the old and new name.
        let old_name = self.world.name();
        let old_root = mem::replace(&mut self.world.root, new_root);
        let new_name = self.world.name();
        let undo = UnReDoState {
            root: old_root,
            database: self.world.database.clone(),
        };
        self.add_undo_state(undo);

        // Save the world, and if necessary update the world's metadata as well.
        let selected = self.worlds.selected();
        save_world(selected, &self.world);
        if old_name != new_name {
            match self.worlds.entry(selected) {
                Entry::Occupied(mut entry) => entry.get_mut().name = new_name,
                Entry::Vacant(entry) => {
                    warn!("World {selected} was not in the worlds map");
                    entry.insert(self.world.metadata());
                }
            }
            save_worlds_list(&self.worlds);
        }

        true
    }
}

impl Component for WorldManager {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (user_settings_dispatcher, _) = ctx
            .link()
            .context::<UserSettingsDispatcher>(Callback::noop())
            .expect("WorldManager must be nested in the UserSettingsManager");

        let (worlds, world) = match load_worlds_list() {
            Ok(mut worlds) => {
                let world = match load_world(worlds.selected()) {
                    Ok(world) => {
                        // Propagate the global metadat empty balances state.
                        #[allow(deprecated)]
                        user_settings_dispatcher
                            .maybe_init_from_world(world.global_metadata.hide_empty_balances);
                        world
                    }
                    Err(e) => {
                        let selected = worlds.selected();
                        warn!("Failed to load selected world {selected}: {}", e);
                        if let Some(world) = worlds.get_mut(selected) {
                            world.load_error = true;
                        }
                        let entry = worlds.allocate_new_id();
                        let world = World::new();
                        save_world(*entry.key(), &world);
                        entry.insert(world.metadata());
                        save_worlds_list(&worlds);
                        world
                    }
                };
                (worlds, world)
            }
            Err(e) => {
                if !matches!(e, StorageError::KeyNotFound(_)) {
                    warn!("Failed to load the world list: {}", e);
                }
                let id = WorldId::new();
                let world = v1storage::try_load_v1();
                // In case we loaded a v1 world, try to init the empty balances state.
                #[allow(deprecated)]
                user_settings_dispatcher
                    .maybe_init_from_world(world.global_metadata.hide_empty_balances);
                let worlds = WorldList::new(id, world.metadata());
                save_world(id, &world);
                save_worlds_list(&worlds);
                (worlds, world)
            }
        };
        let database = world.database.get();

        Self {
            worlds,
            world,
            database,
            undo_stack: VecDeque::with_capacity(MAX_UNDO),
            redo_stack: VecDeque::with_capacity(MAX_UNDO),
        }
    }

    /// Update the WorldManager.
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetRoot { root } => self.set_root(root),
            Msg::UpdateNodeMeta { id, meta } => todo!(),
            Msg::Undo => todo!(),
            Msg::Redo => todo!(),
            Msg::SetDb(database_version) => todo!(),
            Msg::SetWorld(world_id) => todo!(),
            Msg::DeleteWorld(world_id) => todo!(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            {ctx.props().children.clone()}
        }
    }
}

/// Maximum amount of undo history to keep.
const MAX_UNDO: usize = 100;

/// State tracked for undo/redo.
struct UnReDoState {
    /// Database at this undo/redo version.
    database: DatabaseChoice,
    /// Root node of the world at this version.
    root: Node,
}

/// Local storage key where the world list map should be stored/loaded.
const WORLD_MAP_KEY: &str = "zstewart.satisfactorydb.state.world";

/// Load the world list.
fn load_worlds_list() -> Result<WorldList, StorageError> {
    LocalStorage::get(WORLD_MAP_KEY)
}

/// Try to save the world list, logging errors.
fn save_worlds_list(list: &WorldList) {
    if let Err(e) = LocalStorage::set(WORLD_MAP_KEY, list) {
        warn!("Unable to save metadata: {}", e);
    }
}

/// Load the world with the specified id.
fn load_world(id: WorldId) -> Result<World, StorageError> {
    let mut world: World = LocalStorage::get(id.to_string())?;
    // Remove metadata from deleted groups that are definitely no longer in the
    // undo/redo history.
    world.node_metadata.prune(&world.root);
    Ok(world)
}

/// Try to save the world with the given key, ignoring errors.
fn save_world(id: WorldId, world: &World) {
    if let Err(e) = LocalStorage::set(id.to_string(), world) {
        warn!("Unable to save world: {}", e);
    }
}
