use std::collections::{HashMap, VecDeque};
use std::mem;

use gloo::storage::errors::StorageError;
use gloo::storage::{LocalStorage, Storage as _};
use log::{error, info, warn};
use satisfactory_accounting::accounting::Node;
use satisfactory_accounting::database::Database;
use uuid::Uuid;
use yew::html::Scope;
use yew::{
    hook, html, use_context, Callback, Component, Context, ContextProvider, Html, Properties,
};

use crate::refeqrc::RefEqRc;
use crate::user_settings::UserSettingsDispatcher;
use crate::world::list::WorldEntry;
use crate::world::{
    v1storage, DatabaseChoice, DatabaseVersionSelector, NodeMeta, NodeMetas, WorldId,
};
use crate::world::{World, WorldList};

#[derive(PartialEq, Properties)]
pub struct Props {
    /// Children, which will have access to the world and the world manager's various context
    /// handles.
    pub children: Html,
}

#[derive(Debug)]
pub enum Msg {
    /// Set a new Root node for the World.
    SetRoot { root: Node },
    /// Update the metadatum of a single node.
    UpdateNodeMeta {
        /// UUID of the node to update the metadata of.
        id: Uuid,
        /// Meta for the individual node being updated.
        meta: NodeMeta,
    },
    /// Update many node metas at once.
    BatchUpdateNodeMeta(HashMap<Uuid, NodeMeta>),
    /// Change the most recent undo state, pushing the current state to the redo stack.
    Undo,
    /// Change to the most recent redo state, pushing the current state to the undo stack.
    Redo,
    /// Switch to the specified DatabaseVersion.
    SetDb(DatabaseVersionSelector),

    /// Change to the specified World ID.
    SetWorld(WorldId),
    /// Permanently delete the world with the given ID.
    DeleteWorld(WorldId),
    /// Create a new world and switch to it.
    CreateWorld,
}

/// World manager manages the lifecycle of a single world.
pub struct WorldManager {
    /// True if the worlds list has been saved with the latest changes. Anything which mutates the
    /// worlds list needs to clear this flag.
    is_worlds_list_saved: bool,

    /// List of available worlds.
    worlds: WorldList,

    /// Current state of the world.
    world: World,
    /// Currently selected database.
    database: Database,
    /// Stack of previous states for undo.
    undo_stack: VecDeque<UnReDoState>,
    /// Stack of future states for redo.
    redo_stack: VecDeque<UnReDoState>,

    /// Cached rc-wrapped link back to this component, used for the context managers it provides.
    link: RefEqRc<Scope<Self>>,
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

    /// Saves the current state of the current world, logging errors.
    fn save_world(&self) {
        if let Err(e) = LocalStorage::set(self.worlds.selected_id().to_string(), &self.world) {
            warn!("Unable to save world: {}", e);
        }
    }

    /// If the world list never been saved, save it.
    fn save_worlds_list_if_unsaved(&mut self) {
        if !self.is_worlds_list_saved {
            match LocalStorage::set(WORLD_MAP_KEY, &self.worlds) {
                Ok(()) => self.is_worlds_list_saved = true,
                Err(e) => warn!("Unable to save WorldList: {}", e),
            }
        }
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

    /// Update the metadata for the currently selected world. Always saves the world list if it is
    /// in the unsaved state, even if the current world's metadata is unchanged.
    fn update_world_metadata(&mut self) {
        let world_meta = self.world.metadata();
        match self.worlds.selected_entry() {
            // If the world is already present with the correct metadata, do nothing.
            WorldEntry::Present(entry) if *entry.meta() == world_meta => {}
            entry => {
                if !entry.exists() {
                    warn!("World {} was not in the worlds map", entry.id());
                }
                entry.insert_or_update_and_select(self.world.metadata());
                self.is_worlds_list_saved = false;
            }
        }
        self.save_worlds_list_if_unsaved();
    }

    /// Message handler for SetRoot. Returns true if redraw is needed.
    fn set_root(&mut self, new_root: Node) -> bool {
        if new_root.group().is_none() {
            error!("new root {new_root:?} was not a group");
            return false;
        }
        // Update the world state, tracking the old and new name.
        let old_root = mem::replace(&mut self.world.root, new_root);
        let undo = UnReDoState {
            root: old_root,
            database: self.world.database.clone(),
        };
        self.add_undo_state(undo);

        // Save the world, and if necessary update the world's metadata as well.
        self.save_world();
        self.update_world_metadata();
        true
    }

    /// Message handler for SetNodeMeta. Returns true if redraw is needed.
    fn update_node_meta(&mut self, id: Uuid, meta: NodeMeta) -> bool {
        self.world.node_metadata.set_meta(id, meta);
        self.save_world();
        self.save_worlds_list_if_unsaved();
        true
    }

    /// Message handler for BatchUpdateNodeMeta. Returns true if redarw is needed.
    fn batch_update_node_meta(&mut self, updates: HashMap<Uuid, NodeMeta>) -> bool {
        self.world.node_metadata.batch_update(updates);
        self.save_world();
        self.save_worlds_list_if_unsaved();
        true
    }

    /// Message handler for Undo. Returns true if redraw is needed.
    fn undo(&mut self) -> bool {
        match self.undo_stack.pop_back() {
            Some(previous) => {
                let next = self.apply_undo_state(previous);
                // We rely on the limit on the size of the undo stack to limit the size of the redo
                // stack.
                self.redo_stack.push_back(next);
                self.save_world();
                self.update_world_metadata();
                true
            }
            None => {
                warn!("Nothing to undo");
                false
            }
        }
    }

    /// Message handler for Redo. Returns true if redraw is needed.
    fn redo(&mut self) -> bool {
        match self.redo_stack.pop_back() {
            Some(next) => {
                let previous = self.apply_undo_state(next);
                // Rely on the limit on number of undo states enforced earlier to enforce the size
                // limit now.
                // We can't use add_undo_state because that would clear the redo stack.
                self.undo_stack.push_back(previous);
                self.save_world();
                self.update_world_metadata();
                true
            }
            None => {
                warn!("Nothing to redo");
                false
            }
        }
    }

    /// Message hander for SetDb. Set the current database version.
    fn set_db(&mut self, selector: DatabaseVersionSelector) -> bool {
        self.database = selector.load_database();
        let previous = UnReDoState {
            database: mem::replace(&mut self.world.database, selector.into()),
            root: {
                let new_root = self.world.root.rebuild(&self.database);
                mem::replace(&mut self.world.root, new_root)
            },
        };
        self.add_undo_state(previous);
        self.save_world();
        self.update_world_metadata();
        true
    }

    /// Shared helper to set the current world + database + clear the undo/redo stacks. Does not do
    /// any loading or saving.
    fn set_world_inner(&mut self, mut world: World) {
        self.database = world.post_load();
        self.world = world;
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Message handler for SetWorld. Switches to the specified world. Returns true if redraw is
    /// needed.
    fn set_world(&mut self, world_id: WorldId) -> bool {
        match self.worlds.entry(world_id) {
            WorldEntry::Absent(_) => {
                warn!("Unknown world {world_id}");
                false
            }
            WorldEntry::Present(entry) if entry.is_selected() => false,
            WorldEntry::Present(mut entry) => match load_world(world_id) {
                Ok(world) => {
                    entry.select();
                    self.is_worlds_list_saved = false;
                    self.set_world_inner(world);
                    // This will always save the world list if it is unsaved, so since we cleared
                    // the is_world_list_saved flag above, this saves the world list even if the
                    // world's metadata hasn't changed.
                    self.update_world_metadata();
                    true
                }
                Err(e) => {
                    warn!("Unable to load world {world_id}: {e}");
                    // load_error isn't persisted, so don't bother saving the world state here.
                    entry.meta_mut().load_error = true;
                    true
                }
            },
        }
    }

    /// Message handler for DeleteWorld. Removes the specified world and switches to another one or
    /// creates a new empty one if the last world was deleted.
    fn delete_world(&mut self, world_id: WorldId) -> bool {
        // Whether we switched to a different world before removing.
        let changed_world: bool;
        if self.worlds.selected_id() == world_id {
            changed_world = true;
            let mut is_world_persisted = false;
            let new_choice = self
                .worlds
                .iter_mut()
                .find_map(|mut world_meta| {
                    // Skip the world we are deleting.
                    if world_meta.id() == world_id {
                        return None;
                    }
                    match load_world(world_meta.id()) {
                        Ok(world) => {
                            is_world_persisted = true;
                            world_meta.select();
                            self.is_worlds_list_saved = false;
                            Some(world)
                        }
                        Err(e) => {
                            warn!("Unable to load world {}: {e}", world_meta.id());
                            // No need to mark is_worlds_list_saved = false because load_error is
                            // not persisted.
                            world_meta.load_error = true;
                            None
                        }
                    }
                })
                .unwrap_or_else(|| {
                    // No existing world was found which successfully loads and isn't the one we're
                    // about to delete, so create a new one.
                    let entry = self.worlds.allocate_new_id();
                    let world = World::new();
                    entry.insert_and_select(world.metadata());
                    self.is_worlds_list_saved = false;
                    world
                });
            self.set_world_inner(new_choice);
            if !is_world_persisted {
                self.save_world();
            }
            self.save_worlds_list_if_unsaved();
        } else {
            changed_world = false;
        }
        // Whether we actually removed the current world.
        let removed_world: bool;
        match self.worlds.remove(world_id) {
            Ok(_) => {
                removed_world = true;
                self.is_worlds_list_saved = false;
                // Delete from local storage second, in case worlds.remove panics, so we don't lose
                // the world on a panic.
                LocalStorage::delete(world_id.to_string());
                // Wait to save the change to the world list in case local storage panics.
                self.save_worlds_list_if_unsaved();
            }
            Err(e) => {
                removed_world = false;
                warn!("Unable to remove world {world_id}: {e}");
            }
        }

        // Only redraw if something actually changed.
        changed_world || removed_world
    }

    /// Message handler for CreateWorld. Creates a new world and switches to it.
    fn create_world(&mut self) -> bool {
        let entry = self.worlds.allocate_new_id();
        let world = World::new();
        entry.insert_and_select(world.metadata());
        self.is_worlds_list_saved = false;
        self.set_world_inner(world);
        self.save_world();
        self.save_worlds_list_if_unsaved();
        true
    }

    /// Creates the [`WorldListDispatcher`] for this [`WorldManager`].
    fn world_list_dispatcher(&self) -> WorldListDispatcher {
        WorldListDispatcher {
            link: self.link.clone(),
        }
    }

    /// Creates the [`WorldDispatcher`] for this [`WorldManager`].
    fn world_dispatcher(&self) -> WorldDispatcher {
        WorldDispatcher {
            link: self.link.clone(),
        }
    }

    /// Creates the [`DbController`] for the current db.
    fn db_controller(&self) -> DbController {
        DbController {
            current: self.world.database.version_selector(),
            link: self.link.clone(),
        }
    }

    /// Creates the [`UndoController`] for the current undo state.
    fn undo_controller(&self) -> UndoController {
        UndoController {
            has_undo: !self.undo_stack.is_empty(),
            has_redo: !self.redo_stack.is_empty(),
            link: self.link.clone(),
        }
    }
}

impl Component for WorldManager {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        info!("Creating WorldManager");
        let (user_settings_dispatcher, _) = ctx
            .link()
            .context::<UserSettingsDispatcher>(Callback::noop())
            .expect("WorldManager must be nested in the UserSettingsManager");

        enum SaveStatus {
            /// User has not interacted with the app yet (because there is no persisted data), so we
            /// should not save yet.
            ShouldNotSave,
            /// There was persisted data already, so the user has been here before, so we are
            /// allowed to save state if the loading process causes new changes.
            /// If we see this in combination with is_worlds_list_saved = false, then we will save
            /// the worlds list, but not the world.
            MaySave,
            /// There was persisted data already, so the user has been here before. The world has
            /// changed as well, so we should definitely save it. If we encounter this, we will save
            /// the world and maybe save the world metadata, if it is unsaved.
            ShouldSave,
        }

        // Whether we should save during create. Only true if there is already saved state in local
        // storage.
        let mut save_status = SaveStatus::ShouldNotSave;
        let mut is_worlds_list_saved;
        let (worlds, mut world) = match load_worlds_list() {
            Ok(mut worlds) => {
                // If the world list was loaded successfully, then it's already persisted, and we
                // are allowed to save.
                save_status = SaveStatus::MaySave;
                // At this point we can mark the world list as laready saved, and will only unmark
                // it if we make further changes to it.
                is_worlds_list_saved = true;
                let world = match load_world(worlds.selected_id()) {
                    Ok(world) => {
                        // Propagate the global metadata empty balances state.
                        // This will trigger saving user settings, which is fine because we loaded
                        // this state from local storage, meaning we're already using local storage.
                        #[allow(deprecated)]
                        user_settings_dispatcher
                            .maybe_init_from_world(world.global_metadata.hide_empty_balances);
                        let world_meta = world.metadata();
                        // Update the world's metadata on loading, if it is different.
                        match worlds.selected_entry() {
                            WorldEntry::Present(entry) if *entry.meta() == world_meta => {}
                            absent_or_different => {
                                absent_or_different.insert_or_update_and_select(world_meta);
                                is_worlds_list_saved = false;
                            }
                        }
                        world
                    }
                    Err(e) => {
                        let selected = match worlds.selected_entry() {
                            WorldEntry::Present(mut entry) => {
                                // load_error isn't persisted, so don't mark the world list as
                                // not-saved yet here.
                                entry.meta_mut().load_error = true;
                                entry.id()
                            }
                            WorldEntry::Absent(entry) => entry.id(),
                        };
                        warn!("Failed to load selected world {selected}: {e}");
                        let entry = worlds.allocate_new_id();
                        let world = World::new();
                        save_status = SaveStatus::ShouldSave;
                        entry.insert_and_select(world.metadata());
                        is_worlds_list_saved = false;
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
                match v1storage::try_load_v1() {
                    Some(v1world) => {
                        // If there is a v1 world, we need to persist it under the v1.2.x world
                        // storage keys.
                        save_status = SaveStatus::ShouldSave;
                        // In case we loaded a v1 world, try to init the empty balances state.
                        // This will persist user metadata, which is fine because we loaded data
                        // from storage already.
                        #[allow(deprecated)]
                        user_settings_dispatcher
                            .maybe_init_from_world(v1world.global_metadata.hide_empty_balances);
                        let worlds = WorldList::new(id, v1world.metadata());
                        is_worlds_list_saved = false;
                        (worlds, v1world)
                    }
                    None => {
                        let world = World::new();
                        let worlds = WorldList::new(id, world.metadata());
                        is_worlds_list_saved = false;
                        // If nothing was already in storage, avoid saving unless the user interacts
                        // with the app.
                        (worlds, world)
                    }
                }
            }
        };
        let database = world.post_load();

        let mut manager = Self {
            is_worlds_list_saved,
            worlds,
            world,
            database,
            undo_stack: VecDeque::with_capacity(MAX_UNDO),
            redo_stack: VecDeque::with_capacity(MAX_UNDO),
            link: RefEqRc::new(ctx.link().clone()),
        };
        match save_status {
            SaveStatus::ShouldNotSave => {}
            SaveStatus::MaySave => manager.save_worlds_list_if_unsaved(),
            SaveStatus::ShouldSave => {
                manager.save_world();
                manager.save_worlds_list_if_unsaved();
            }
        }
        manager
    }

    /// Update the WorldManager.
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetRoot { root } => self.set_root(root),
            Msg::UpdateNodeMeta { id, meta } => self.update_node_meta(id, meta),
            Msg::BatchUpdateNodeMeta(updates) => self.batch_update_node_meta(updates),
            Msg::Undo => self.undo(),
            Msg::Redo => self.redo(),
            Msg::SetDb(selector) => self.set_db(selector),
            Msg::SetWorld(world_id) => self.set_world(world_id),
            Msg::DeleteWorld(world_id) => self.delete_world(world_id),
            Msg::CreateWorld => self.create_world(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <ContextProvider<WorldList> context={self.worlds.clone()}>
            <ContextProvider<Database> context={self.database.clone()}>
            <ContextProvider<WorldRoot> context={WorldRoot(self.world.root.clone())}>
            <ContextProvider<NodeMetas> context={self.world.node_metadata.clone()}>
            <ContextProvider<WorldListDispatcher> context={self.world_list_dispatcher()}>
            <ContextProvider<WorldDispatcher> context={self.world_dispatcher()}>
            <ContextProvider<UndoController> context={self.undo_controller()}>
            <ContextProvider<DbController> context={self.db_controller()}>
                {ctx.props().children.clone()}
            </ContextProvider<DbController>>
            </ContextProvider<UndoController>>
            </ContextProvider<WorldDispatcher>>
            </ContextProvider<WorldListDispatcher>>
            </ContextProvider<NodeMetas>>
            </ContextProvider<WorldRoot>>
            </ContextProvider<Database>>
            </ContextProvider<WorldList>>
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

/// Load the world with the specified id.
fn load_world(id: WorldId) -> Result<World, StorageError> {
    let mut world: World = LocalStorage::get(id.to_string())?;
    // Remove metadata from deleted groups that are definitely no longer in the
    // undo/redo history.
    world.node_metadata.prune(&world.root);
    Ok(world)
}

/// Gets the current world list.
#[hook]
pub fn use_world_list() -> WorldList {
    use_context::<WorldList>()
        .expect("use_world_list can only be used from within a child of WorldManager")
}

/// Dispatcher used to make changes to the world list.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldListDispatcher {
    /// Link used to send messages back to the WorldManager.
    link: RefEqRc<Scope<WorldManager>>,
}

impl WorldListDispatcher {
    /// Set the currently selected world.
    pub fn set_world(&self, world_id: WorldId) {
        self.link.send_message(Msg::SetWorld(world_id));
    }

    /// Permanently deletes this world. Does not trigger a confirmation.
    pub fn delete_world(&self, world_id: WorldId) {
        self.link.send_message(Msg::DeleteWorld(world_id));
    }

    /// Creates a new empty world and switches to it.
    pub fn create_world(&self) {
        self.link.send_message(Msg::CreateWorld);
    }
}

/// Gets the dispatcher used to manage the world list.
#[hook]
pub fn use_world_list_dispatcher() -> WorldListDispatcher {
    use_context::<WorldListDispatcher>()
        .expect("use_world_list_dispatcher can only be used from within a child of WorldManager")
}

/// Context wrapper for the root node of the current world.
#[derive(Debug, Clone, PartialEq)]
struct WorldRoot(Node);

/// Gets the root node of the world.
#[hook]
pub fn use_world_root() -> Node {
    use_context::<WorldRoot>()
        .expect("use_world_root can only be used from within a child of WorldManager")
        .0
        .clone()
}

/// Dispatcher used to make changes to the World.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldDispatcher {
    /// Link used to send messages back to the WorldManager.
    link: RefEqRc<Scope<WorldManager>>,
}

impl WorldDispatcher {
    /// Set the world root.
    pub fn set_root(&self, root: Node) {
        self.link.send_message(Msg::SetRoot { root });
    }

    /// Update a single node's metadata.
    pub fn update_node_meta(&self, id: Uuid, meta: NodeMeta) {
        self.link.send_message(Msg::UpdateNodeMeta { id, meta });
    }

    /// Update a many nodes' metadata.
    pub fn batch_update_node_meta(&self, updates: HashMap<Uuid, NodeMeta>) {
        self.link.send_message(Msg::BatchUpdateNodeMeta(updates));
    }
}

/// Gets the world dispatcher.
#[hook]
pub fn use_world_dispatcher() -> WorldDispatcher {
    use_context::<WorldDispatcher>()
        .expect("use_world_dispatcher can only be used from within a child of WorldManager")
}

/// Get the database from context.
#[hook]
pub fn use_db() -> Database {
    use_context::<Database>().expect("use_db can only be used from within a child of WorldManager")
}

/// Controller for the database selection.
#[derive(Debug, Clone, PartialEq)]
pub struct DbController {
    /// Current database, if the current database is not custom.
    current: Option<DatabaseVersionSelector>,
    /// Link used to send messages to the WorldManager.
    link: RefEqRc<Scope<WorldManager>>,
}

impl DbController {
    /// Gets the current database selector. If the current database is a custom database, returns
    /// None.
    pub fn current_selector(&self) -> Option<DatabaseVersionSelector> {
        self.current
    }

    /// Gets the current database dispatcher
    pub fn dispatcher(&self) -> DbDispatcher {
        DbDispatcher {
            link: self.link.clone(),
        }
    }
}

/// Dispatcher for updating the selected database.
#[derive(Debug, Clone, PartialEq)]
pub struct DbDispatcher {
    /// Link used to send messages to the WorldManager.
    link: RefEqRc<Scope<WorldManager>>,
}

impl DbDispatcher {
    /// Updates the current database version.
    pub fn set_database(&self, selector: DatabaseVersionSelector) {
        self.link.send_message(Msg::SetDb(selector));
    }
}

/// Gets the DbController from the context.
#[hook]
pub fn use_db_controller() -> DbController {
    use_context::<DbController>()
        .expect("use_db_controller can only be used from within a child of the WorldManager")
}

/// Controller for the undo state.
#[derive(Debug, Clone, PartialEq)]
pub struct UndoController {
    /// Whether there was any state available to undo.
    has_undo: bool,
    /// Whether there was any state available to redo.
    has_redo: bool,
    /// Link used to send messages to the WorldManager.
    link: RefEqRc<Scope<WorldManager>>,
}

impl UndoController {
    /// Returns true if there is undo state available.
    pub fn has_undo(&self) -> bool {
        self.has_undo
    }

    /// Returns true if there is redo state available.
    pub fn has_redo(&self) -> bool {
        self.has_redo
    }

    /// Gets a dispatcher to trigger undo/redo.
    pub fn dispatcher(&self) -> UndoDispatcher {
        UndoDispatcher {
            link: self.link.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UndoDispatcher {
    /// Link used to send messages to the WorldManager.
    link: RefEqRc<Scope<WorldManager>>,
}

impl UndoDispatcher {
    /// Triggers undo.
    pub fn undo(&self) {
        self.link.send_message(Msg::Undo);
    }

    /// Triggers redo.
    pub fn redo(&self) {
        self.link.send_message(Msg::Redo);
    }
}

/// Gets the UndoController from the context.
#[hook]
pub fn use_undo_controller() -> UndoController {
    use_context::<UndoController>()
        .expect("use_undo_controller can only be used from within a child of the WorldManager")
}
