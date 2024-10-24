use std::cell::{Ref, RefCell};
use std::collections::{HashMap, VecDeque};
use std::mem;
use std::ops::Deref;
use std::rc::Rc;

use gloo::storage::errors::StorageError;
use gloo::storage::{LocalStorage, Storage as _};
use log::{error, info, warn};
use satisfactory_accounting::accounting::Node;
use satisfactory_accounting::database::Database;
use thiserror::Error;
use uuid::Uuid;
use yew::html::Scope;
use yew::{
    hook, html, use_context, AttrValue, Callback, Component, Context, ContextHandle,
    ContextProvider, Html, Properties,
};

use crate::bugreport::file_a_bug;
use crate::modal::{ModalDispatcher, ModalOk};
use crate::refeqrc::RefEqRc;
use crate::user_settings::UserSettingsDispatcher;
use crate::world::list::WorldEntry;
use crate::world::{
    v1storage, DatabaseChoice, DatabaseVersionSelector, NodeMeta, NodeMetas, SaveFile, WorldId,
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
    /// Mark an error on the given world id.
    MarkError(WorldId),
}

/// Wrapper for reporting errors from the world manager.
#[derive(Debug, Clone)]
struct WorldManagerErrorReporter {
    /// Dispatcher used to send modal dialogs.
    ///
    /// This is stored in a refcell because it doesn't affect our rendering and is only used from
    /// create and update, so we don't need to use a message to replace it if it ever changes
    /// (though it shouldn't actually change).
    /// It has to be wrapped in Option because we need to construct the Rc before we can create the
    /// callback to pass to receive context updates, but we won't have a value to store until we get
    /// the context. It should never be None after create.
    modal_dispatcher: Rc<RefCell<Option<ModalDispatcher>>>,
}

impl WorldManagerErrorReporter {
    /// Report an error through a persisted modal dialog.
    fn report_error(&self, title: impl Into<AttrValue>, content: Html) {
        self.modal_dispatcher
            .borrow()
            .as_ref()
            .expect(
                "ModalDispatcher was not set. This should always be set since it is only Optional \
                for initialization.",
            )
            .builder()
            .class("WorldManagerError")
            .kind(ModalOk::close())
            .title(title)
            .content(content)
            .build()
            .persist()
    }
}

mod save_tracker {
    use std::ops::{Deref, DerefMut};

    use gloo::storage::{LocalStorage, Storage};
    use log::warn;
    use serde::Serialize;
    use yew::html;

    use crate::bugreport::file_a_bug;
    use crate::world::manager::{WorldManagerErrorReporter, WORLD_MAP_KEY};
    use crate::world::{World, WorldId, WorldList};

    /// Tracks whether the given value has been saved.
    pub(super) struct SaveTracker<T, K> {
        /// The value that needs to be stored.
        value: T,
        /// Local storage key used for this item.
        key: K,
        /// Error reporter used to report save errors.
        error_reporter: WorldManagerErrorReporter,
        /// A bool indicating whether the value has been saved yet or not.
        is_saved: bool,
    }

    pub type WorldListTracker = SaveTracker<WorldList, &'static str>;
    pub type WorldTracker = SaveTracker<World, String>;

    impl<T, K> SaveTracker<T, K>
    where
        T: Serialize,
        K: AsRef<str>,
    {
        /// Try to save, updating the is_saved state if successful.
        pub fn try_save_if_unsaved(&mut self) {
            if !self.is_saved {
                match LocalStorage::set(self.key.as_ref(), &self.value) {
                    Ok(()) => self.is_saved = true,
                    Err(e) => {
                        let typename = std::any::type_name::<T>();
                        let title = format!("Unable to save {typename}");
                        let error_message = html! {
                            <>
                            <p>{"We were unable to save the most recent change to your "}{typename}
                            {". You may be out of Browser Storage quota (there is a standard 10MiB \
                            limit per website which we have no control over) or this may be a bug. \
                            If it appears to be a bug, you can "}{file_a_bug()}{". If you file a \
                            bug, please include this message:"}</p>
                            <pre>
                                {"Unable to save "}{typename}{": "}{&e}
                            </pre>
                            </>
                        };
                        self.error_reporter.report_error(title, error_message);
                        warn!("Unable to save {typename}: {e}",);
                    }
                }
            }
        }
    }

    impl SaveTracker<WorldList, &'static str> {
        /// Create a SaveTracker for an already saved value.
        pub fn saved(value: WorldList, error_reporter: WorldManagerErrorReporter) -> Self {
            Self {
                value,
                key: WORLD_MAP_KEY,
                error_reporter,
                is_saved: true,
            }
        }

        /// Create a SaveTracker for an unsaved value.
        pub fn unsaved(value: WorldList, error_reporter: WorldManagerErrorReporter) -> Self {
            Self {
                value,
                key: WORLD_MAP_KEY,
                error_reporter,
                is_saved: false,
            }
        }
    }

    impl SaveTracker<World, String> {
        /// Create a SaveTracker for an already saved value.
        pub fn saved(value: World, id: WorldId, error_reporter: WorldManagerErrorReporter) -> Self {
            Self {
                value,
                key: id.to_string(),
                error_reporter,
                is_saved: true,
            }
        }

        /// Create a SaveTracker for an unsaved value.
        pub fn unsaved(
            value: World,
            id: WorldId,
            error_reporter: WorldManagerErrorReporter,
        ) -> Self {
            Self {
                value,
                key: id.to_string(),
                error_reporter,
                is_saved: false,
            }
        }
    }

    impl<T, K> SaveTracker<T, K> {
        /// Get a mutable reference to the value without marking it as in need of saving.
        pub fn mutate_without_marking_dirty(&mut self) -> &mut T {
            &mut self.value
        }

        /// Gets a mutable reference to the contained value and marks the value as needing to be
        /// saved.
        pub fn mutate_and_mark_dirty(&mut self) -> &mut T {
            self.is_saved = false;
            &mut self.value
        }

        /// Gets a handle to the value for if you aren't sure if you are going to mutate it.
        pub fn maybe_mutate(&mut self) -> MutateHandle<T> {
            MutateHandle {
                value: &mut self.value,
                is_saved: Some(&mut self.is_saved),
            }
        }
    }

    impl<T, K> Deref for SaveTracker<T, K> {
        type Target = T;

        #[inline]
        fn deref(&self) -> &Self::Target {
            &self.value
        }
    }

    /// DerefMut on a SaveTracker always marks the value as dirty.
    impl<T, K> DerefMut for SaveTracker<T, K> {
        #[inline]
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.mutate_and_mark_dirty()
        }
    }

    /// A mutation handle which marks the value as dirty when dropped, unless explicitly told there
    /// is no change.
    pub struct MutateHandle<'a, T> {
        value: &'a mut T,
        is_saved: Option<&'a mut bool>,
    }

    impl<'a, T> MutateHandle<'a, T> {
        /// Consumes this MutateHandle and prevents it from marking the value as changed.
        pub fn no_change(mut self) {
            self.is_saved = None;
        }
    }

    impl<'a, T> Deref for MutateHandle<'a, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            self.value
        }
    }

    impl<'a, T> DerefMut for MutateHandle<'a, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.value
        }
    }

    impl<'a, T> Drop for MutateHandle<'a, T> {
        fn drop(&mut self) {
            if let Some(is_saved) = self.is_saved.as_deref_mut() {
                *is_saved = false;
            }
        }
    }
}

use save_tracker::{WorldListTracker, WorldTracker};

/// World manager manages the lifecycle of a single world.
pub struct WorldManager {
    /// List of available worlds.
    worlds: WorldListTracker,

    /// Current state of the world.
    world: WorldTracker,
    /// Currently selected database.
    database: Database,
    /// Stack of previous states for undo.
    undo_stack: VecDeque<UnReDoState>,
    /// Stack of future states for redo.
    redo_stack: VecDeque<UnReDoState>,

    /// Cached rc-wrapped link back to this component, used for the context managers it provides.
    link: Link,
    /// World reader which tracks the current world.
    world_reader: WorldReader,

    /// Utility used to send modal dialogs on errors.
    error_reporter: WorldManagerErrorReporter,
    /// Handle which ensure we receive updates to the modal dispatcher used in the error_reporter if
    /// it changes.
    _modal_dispatcher_handle: ContextHandle<ModalDispatcher>,
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

    /// Update the metadata for the currently selected world. Always saves the world list if it is
    /// in the unsaved state, even if the current world's metadata is unchanged.
    fn update_world_metadata(&mut self) {
        let world_meta = self.world.metadata();
        {
            let mut handle = self.worlds.maybe_mutate();
            match handle.selected_entry() {
                // If the world is already present with the correct metadata, do nothing.
                WorldEntry::Present(entry) if *entry.meta() == world_meta => handle.no_change(),
                entry => {
                    if !entry.exists() {
                        warn!("World {} was not in the worlds map", entry.id());
                    }
                    entry.insert_or_update_and_select(self.world.metadata());
                }
            }
        }
        self.worlds.try_save_if_unsaved();
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
        self.world.try_save_if_unsaved();
        self.update_world_metadata();
        true
    }

    /// Message handler for SetNodeMeta. Returns true if redraw is needed.
    fn update_node_meta(&mut self, id: Uuid, meta: NodeMeta) -> bool {
        self.world.node_metadata.set_meta(id, meta);
        self.world.try_save_if_unsaved();
        self.worlds.try_save_if_unsaved();
        true
    }

    /// Message handler for BatchUpdateNodeMeta. Returns true if redarw is needed.
    fn batch_update_node_meta(&mut self, updates: HashMap<Uuid, NodeMeta>) -> bool {
        self.world.node_metadata.batch_update(updates);
        self.world.try_save_if_unsaved();
        self.worlds.try_save_if_unsaved();
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
                self.world.try_save_if_unsaved();
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
                self.world.try_save_if_unsaved();
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
        self.world.try_save_if_unsaved();
        self.update_world_metadata();
        true
    }

    /// Shared helper to set the current world + database + clear the undo/redo stacks. Does not do
    /// any loading or saving.
    fn set_world_inner(&mut self, mut new_world: WorldTracker) {
        // Neither the root rebuild nor metadata pruning should trigger marking the world as dirty,
        // as both of those things can be re-done on future loads without affecting anything else.
        self.database = new_world.mutate_without_marking_dirty().post_load();
        self.world = new_world;
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Message handler for SetWorld. Switches to the specified world. Returns true if redraw is
    /// needed.
    fn set_world(&mut self, world_id: WorldId) -> bool {
        let mut handle = self.worlds.maybe_mutate();
        match handle.entry(world_id) {
            WorldEntry::Absent(_) => {
                warn!("Unknown world {world_id}");
                handle.no_change();
                false
            }
            WorldEntry::Present(entry) if entry.is_selected() => {
                handle.no_change();
                false
            }
            WorldEntry::Present(mut entry) => match load_world(world_id) {
                Ok(world) => {
                    entry.select();
                    // Release the handle and mark as changed.
                    drop(handle);
                    // Save the existing world before switching, in case it wasn't already saved.
                    self.world.try_save_if_unsaved();
                    // Set the world, marking it as already saved, since w just loaded it.
                    self.set_world_inner(WorldTracker::saved(
                        world,
                        world_id,
                        self.error_reporter.clone(),
                    ));
                    // This will always save the world list if it is unsaved, so it will persist the
                    // change to which entry is selected.
                    self.update_world_metadata();
                    true
                }
                Err(e) => {
                    warn!("Unable to load world {world_id}: {e}");
                    match e {
                        StorageError::KeyNotFound(_) => {
                            let title = "World Data Missing";
                            let content = html! {
                                <p>{"The world you selected appears to be missing from your
                                browser's storage, so we were unable to load it. Sorry about that."}
                                </p>
                            };
                            self.error_reporter.report_error(title, content);
                        }
                        e => {
                            let title = "Error Loading World";
                            let content = html! {
                                <>
                                <p>{"We were unable to load the world you selected. This may be a \
                                bug. Your world data seems to still be present, so this may be \
                                recoverable. For help you can "}{file_a_bug()}{". If you file a \
                                bug, please include this error message:"}</p>
                                <pre>
                                    {"Unable to load world "}{world_id}{": "}{e}
                                </pre>
                                </>
                            };
                            self.error_reporter.report_error(title, content);
                        }
                    }
                    // load_error isn't persisted, so don't bother saving the world state here.
                    entry.meta_mut().load_error = true;
                    // Updating load_error is not a saveable change, so don't mark as needing save.
                    handle.no_change();
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
            let new_choice = self
                .worlds
                // Don't worry about avoiding makring as mutated for the iterator, because both
                // paths result in mutation eventually.
                .iter_mut()
                .find_map(|mut world_meta| {
                    // Skip the world we are deleting.
                    if world_meta.id() == world_id {
                        return None;
                    }
                    match load_world(world_meta.id()) {
                        Ok(world) => {
                            world_meta.select();
                            // Just loaded this world, so it is already saved.
                            Some(WorldTracker::saved(
                                world,
                                world_meta.id(),
                                self.error_reporter.clone(),
                            ))
                        }
                        Err(e) => {
                            warn!("Unable to load world {}: {e}", world_meta.id());
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
                    let id = entry.id();
                    entry.insert_and_select(world.metadata());
                    // This is a new world, so it's not saved.
                    WorldTracker::unsaved(world, id, self.error_reporter.clone())
                });
            // The current world is being deleted, so don't bother proactively saving it before the
            // set_world call.
            self.set_world_inner(new_choice);
        } else {
            changed_world = false;
        }
        // Whether we actually removed the current world.
        let removed_world: bool;
        {
            let mut handle = self.worlds.maybe_mutate();
            match handle.remove(world_id) {
                Ok(_) => {
                    removed_world = true;
                    // Delete from local storage before persisting the world list.
                    LocalStorage::delete(world_id.to_string());
                }
                Err(e) => {
                    removed_world = false;
                    // Don't mark the worlds list as changed if remove failed.
                    handle.no_change();
                    warn!("Unable to remove world {world_id}: {e}");
                }
            }
        }

        self.world.try_save_if_unsaved();
        self.worlds.try_save_if_unsaved();

        // Only redraw if something actually changed.
        changed_world || removed_world
    }

    /// Message handler for CreateWorld. Creates a new world and switches to it.
    fn create_world(&mut self) -> bool {
        // If the current world has unsaved state, save it before creating a new world.
        self.world.try_save_if_unsaved();

        let entry = self.worlds.allocate_new_id();
        let world = World::new();
        let id = entry.id();
        entry.insert_and_select(world.metadata());
        self.set_world_inner(WorldTracker::unsaved(
            world,
            id,
            self.error_reporter.clone(),
        ));
        self.world.try_save_if_unsaved();
        self.worlds.try_save_if_unsaved();
        true
    }

    /// Message handler for MarkError. Adds an error marker to the given world.
    fn mark_error(&mut self, id: WorldId) -> bool {
        // The error indicator is not saved, so we don't need to mark dirty on this change.
        match self.worlds.mutate_without_marking_dirty().get_mut(id) {
            Some(mut world_meta) if !world_meta.load_error => {
                world_meta.load_error = true;
                true
            }
            // World not found or already marked with an error, so no change to redraw.
            _ => false,
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
        let modal_dispatcher = Rc::new(RefCell::new(None));
        let (inner_dispatcher, modal_dispatcher_handle) = ctx
            .link()
            .context::<ModalDispatcher>({
                let cell = modal_dispatcher.clone();
                Callback::from(move |new_dispatcher| *cell.borrow_mut() = Some(new_dispatcher))
            })
            .expect("WorldManager must be nesed in the ModalManager");
        *modal_dispatcher.borrow_mut() = Some(inner_dispatcher);
        let error_reporter = WorldManagerErrorReporter { modal_dispatcher };

        let (worlds, mut world) = match load_worlds_list() {
            Ok(worlds) => {
                // World list is currently saved.
                let mut worlds = WorldListTracker::saved(worlds, error_reporter.clone());
                let mut world = match load_world(worlds.selected_id()) {
                    Ok(world) => {
                        // Propagate the global metadata empty balances state.
                        // This will trigger saving user settings, which is fine because we loaded
                        // this state from local storage, meaning we're already using local storage.
                        #[allow(deprecated)]
                        user_settings_dispatcher
                            .maybe_init_from_world(world.global_metadata.hide_empty_balances);
                        let world_meta = world.metadata();
                        let id = worlds.selected_id();
                        // Update the world's metadata on loading, if it is different.
                        let mut handle = worlds.maybe_mutate();
                        match handle.selected_entry() {
                            WorldEntry::Present(entry) if *entry.meta() == world_meta => {
                                handle.no_change();
                            }
                            absent_or_different => {
                                absent_or_different.insert_or_update_and_select(world_meta);
                            }
                        }
                        // This world is currently already saved since we just loaded it.
                        WorldTracker::saved(world, id, error_reporter.clone())
                    }
                    Err(e) => {
                        // The error indicator isn't persisted, so this step doesn't need to mark
                        // the world as dirty.
                        let selected = match worlds.mutate_without_marking_dirty().selected_entry()
                        {
                            WorldEntry::Present(mut entry) => {
                                entry.meta_mut().load_error = true;
                                entry.id()
                            }
                            WorldEntry::Absent(entry) => entry.id(),
                        };
                        warn!("Failed to load selected world {selected}: {e}");
                        match e {
                            StorageError::KeyNotFound(_) => {
                                let title = "World Data Missing";
                                let content = html! {
                                    <>
                                    <p>{"The world you had selected when you last opened the app \
                                    appears to be missing from your browser's storage, so we were \
                                    unable to load it. Sorry about that."}</p>
                                    <p>{"You have been automatically placed in a new world."}</p>
                                    </>
                                };
                                error_reporter.report_error(title, content);
                            }
                            e => {
                                let title = "Error Loading World";
                                let content = html! {
                                    <>
                                    <p>{"We were unable to load the world that you had selected \
                                    when you last used the app. This may be a bug. Your world data \
                                    seems to still be present, so this may be recoverable."}</p>
                                    <p>{"For now, you have been automatically placed in a new, \
                                    empty world. You can continue with this new world or you can "}
                                    {file_a_bug()}{". If you file a bug, please include this error \
                                    message:"}</p>
                                    <pre>
                                        {"Failed to load selected world "}{selected}{": "}{e}
                                    </pre>
                                    </>
                                };
                                error_reporter.report_error(title, content);
                            }
                        }
                        let entry = worlds.allocate_new_id();
                        let world = World::new();
                        let id = entry.id();
                        entry.insert_and_select(world.metadata());
                        // The newly created world isn't saved yet.
                        WorldTracker::unsaved(world, id, error_reporter.clone())
                    }
                };
                // The worlds list already existed on the system, so we can save if there are any
                // changes or a new world.
                world.try_save_if_unsaved();
                worlds.try_save_if_unsaved();
                (worlds, world)
            }
            Err(StorageError::KeyNotFound(_)) => {
                // If the world manager was not found, try to load from a v1 world or just start
                // from scratch.
                let id = WorldId::new();
                match v1storage::try_load_v1() {
                    Some(v1world) => {
                        // If there is a v1 world, we need to persist it under the v1.2.x world
                        // storage keys.
                        // In case we loaded a v1 world, try to init the empty balances state.
                        // This will persist user metadata, which is fine because we loaded data
                        // from storage already.
                        #[allow(deprecated)]
                        user_settings_dispatcher
                            .maybe_init_from_world(v1world.global_metadata.hide_empty_balances);
                        let mut world = WorldTracker::unsaved(v1world, id, error_reporter.clone());
                        let mut worlds = WorldListTracker::unsaved(
                            WorldList::new(id, world.metadata()),
                            error_reporter.clone(),
                        );
                        // Since there was already a v1 world in the browser, we can persist
                        // immediately.
                        world.try_save_if_unsaved();
                        worlds.try_save_if_unsaved();
                        (worlds, world)
                    }
                    None => {
                        let world = World::new();
                        let worlds = WorldList::new(id, world.metadata());
                        // If nothing was already in storage, avoid saving unless the user
                        // interacts with the app.
                        (
                            WorldListTracker::unsaved(worlds, error_reporter.clone()),
                            WorldTracker::unsaved(world, id, error_reporter.clone()),
                        )
                    }
                }
            }
            Err(e) => {
                let error_message = html! {
                    <>
                        <p>{"A world list was found in your browser's local storage, but it could \
                        not be loaded. Any worlds you had may still exist, and your world list may \
                        still be recoverable, however if you make any changes in the app, your \
                        previous world list will be overwritten. Any existing worlds will not be \
                        overwritten, though they cannot be retrieved from in the app."}</p>
                        <p>{"You can either continue using the app, which will just give you a \
                        new, empty world list and overwrite the existing broken one, or you can "}
                        {file_a_bug()}{". If you file a bug, please include this message:"}</p>
                        <pre>
                            {"Failed to load the world list: "}{&e}
                        </pre>
                    </>
                };
                error_reporter.report_error("Error loading world list", error_message);
                warn!("Failed to load the world list: {}", e);
                // If a world list exists and just failed to parse, don't try to load a v1
                // world, just fall back to an empty world and don't mark as needing a save.
                let id = WorldId::new();
                let world = World::new();
                let worlds = WorldList::new(id, world.metadata());
                // Don't save yet to avoid overwriting what was already in storage. These will both
                // be saved if either is modified.
                (
                    WorldListTracker::unsaved(worlds, error_reporter.clone()),
                    WorldTracker::unsaved(world, id, error_reporter.clone()),
                )
            }
        };
        // The post-load operations should not create a new save, since they can be repeated on
        // every load.
        let database = world.mutate_without_marking_dirty().post_load();
        let world_reader = WorldReader::new(worlds.selected_id(), world.clone());

        Self {
            worlds,
            world,
            database,
            undo_stack: VecDeque::with_capacity(MAX_UNDO),
            redo_stack: VecDeque::with_capacity(MAX_UNDO),
            link: Link::new(ctx.link().clone()),
            world_reader,
            error_reporter,
            _modal_dispatcher_handle: modal_dispatcher_handle,
        }
    }

    /// Update the WorldManager.
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        let redraw = match msg {
            Msg::SetRoot { root } => self.set_root(root),
            Msg::UpdateNodeMeta { id, meta } => self.update_node_meta(id, meta),
            Msg::BatchUpdateNodeMeta(updates) => self.batch_update_node_meta(updates),
            Msg::Undo => self.undo(),
            Msg::Redo => self.redo(),
            Msg::SetDb(selector) => self.set_db(selector),
            Msg::SetWorld(world_id) => self.set_world(world_id),
            Msg::DeleteWorld(world_id) => self.delete_world(world_id),
            Msg::CreateWorld => self.create_world(),
            Msg::MarkError(id) => self.mark_error(id),
        };
        // This should be relatively cheap because all the content of the world is Rc'd.
        // This being held here does prevent the Rcs from ever successfully doing a Rc::make_mut,
        // but Yew holds Rcs for all these things anyway, so those make_mut calls don't work
        // already.
        //
        // Replacing the old world does cause it to be dropped, but its contents would be dropped
        // elsewhere regardless if this replacement call causes them to be dropped.
        self.world_reader
            .set(self.worlds.selected_id(), self.world.clone());
        redraw
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <ContextProvider<WorldList> context={self.worlds.clone()}>
            <ContextProvider<Database> context={self.database.clone()}>
            <ContextProvider<WorldReader> context={self.world_reader.clone()}>
            <ContextProvider<WorldRoot> context={WorldRoot(self.world.root.clone())}>
            <ContextProvider<NodeMetas> context={self.world.node_metadata.clone()}>
            <ContextProvider<Link> context={self.link.clone()}>
            <ContextProvider<UndoController> context={self.undo_controller()}>
            <ContextProvider<DbController> context={self.db_controller()}>
                {ctx.props().children.clone()}
            </ContextProvider<DbController>>
            </ContextProvider<UndoController>>
            </ContextProvider<Link>>
            </ContextProvider<NodeMetas>>
            </ContextProvider<WorldRoot>>
            </ContextProvider<WorldReader>>
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

/// Wrapper for the link to the world manager which makes its context package private.
#[derive(Debug, PartialEq, Clone)]
#[repr(transparent)]
struct Link(RefEqRc<Scope<WorldManager>>);

impl Link {
    /// Creates a new link.
    #[inline]
    fn new(link: Scope<WorldManager>) -> Self {
        Self(RefEqRc::new(link))
    }
}

impl Deref for Link {
    type Target = Scope<WorldManager>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
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
    link: Link,
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
    let link = use_context::<Link>()
        .expect("use_world_list_dispatcher can only be used from within a child of WorldManager");
    WorldListDispatcher { link }
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
    link: Link,
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
    let link = use_context::<Link>()
        .expect("use_world_dispatcher can only be used from within a child of WorldManager");
    WorldDispatcher { link }
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
    link: Link,
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
    link: Link,
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
    link: Link,
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
    link: Link,
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

/// This context type contains the current World and WorldId, but does not trigger context updates
/// when the world changes. This is useful for things which need to read the world on-demand but
/// don't need to re-render every time the world changes.
#[derive(Debug, Clone, PartialEq)]
struct WorldReader {
    inner: RefEqRc<RefCell<WorldReaderInner>>,
}

impl WorldReader {
    /// Create a new world reader.
    fn new(id: WorldId, world: World) -> Self {
        Self {
            inner: RefEqRc::new(RefCell::new(WorldReaderInner { id, world })),
        }
    }

    /// Borrow the current world.
    fn borrow(&self) -> WorldRef {
        WorldRef {
            inner: self.inner.borrow(),
        }
    }

    /// Updates the world. This does not trigger context changes but does mean that future reads
    /// will get the new value.
    fn set(&self, id: WorldId, world: World) {
        let mut inner = self.inner.borrow_mut();
        inner.id = id;
        inner.world = world;
    }
}

#[derive(Debug)]
struct WorldReaderInner {
    /// Id of the current world.
    id: WorldId,
    world: World,
}

/// Borrow of the WorldFetch. Holding this prevents the world from being updated.
struct WorldRef<'a> {
    inner: Ref<'a, WorldReaderInner>,
}

impl<'a> WorldRef<'a> {
    /// Gets the ID of the current world.
    fn id(&self) -> WorldId {
        self.inner.id
    }

    /// Gets the current worl.
    fn world(&self) -> &World {
        &self.inner.world
    }
}

/// Errors which can occur when fetching
#[derive(Error, Debug)]
pub enum FetchSaveFileError {
    #[error("Error loading the specified world from storage: {0}")]
    StorageError(#[from] StorageError),
}

/// Utility for fetching save files for particular worlds.
#[derive(Debug, Clone, PartialEq)]
pub struct SaveFileFetcher {
    reader: WorldReader,
    link: Link,
}

impl SaveFileFetcher {
    /// Get the save file for the given world.
    pub fn get_save_file(&self, id: WorldId) -> Result<SaveFile, FetchSaveFileError> {
        {
            let current = self.reader.borrow();
            // Retrieving the current world this way ensures that the download button still works even
            // if the current world hasn't been saved yet. That can only happen if the user is new and
            // hasn't made any edits yet or if the world list was corrupted and we just dropped into a
            // new world, but we don't want the download buttons to break in those cases.
            if current.id() == id {
                return Ok(SaveFile::Version1Minor2(current.world().clone()));
            }
        }
        // Note: this currently does not optimize for the case where the requested world is
        // currently loaded, though we do have the option to do so in the future if desired, since
        // we force caller to get this type through a hook.
        let world = load_world(id).inspect_err(|_| self.link.send_message(Msg::MarkError(id)))?;
        Ok(SaveFile::Version1Minor2(world))
    }
}

/// Gets the SaveFileFetcher.
#[hook]
pub fn use_save_file_fetcher() -> SaveFileFetcher {
    let reader = use_context::<WorldReader>()
        .expect("use_save_file_fetcher can only be used from within a child of the WorldManager");
    let link = use_context::<Link>()
        .expect("use_save_file_fetcher can only be used from within a child of the WorldManager");
    SaveFileFetcher { reader, link }
}
