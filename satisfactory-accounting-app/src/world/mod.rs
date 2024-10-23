use log::warn;
use satisfactory_accounting::accounting::{Group, Node};
use satisfactory_accounting::database::Database;
use serde::{Deserialize, Serialize};
use yew::AttrValue;

pub use self::dbchoice::{DatabaseChoice, DatabaseVersionSelector};
#[allow(unused_imports)]
pub use self::dbwindow::{
    use_db_chooser_window, DbChooserWindowDispatcher, DbChooserWindowManager,
};
#[allow(unused_imports)]
pub use self::id::{ParseWorldIdError, WorldId};
pub use self::list::{WorldList, WorldMetadata};
#[allow(unused_imports)]
pub use self::manager::{
    use_db, use_db_controller, use_save_file_fetcher, use_undo_controller, use_world_dispatcher,
    use_world_list, use_world_list_dispatcher, use_world_root, DbController, FetchSaveFileError,
    SaveFileFetcher, UndoController, UndoDispatcher, WorldDispatcher, WorldListDispatcher,
    WorldManager,
};
pub use self::meta::{NodeMeta, NodeMetas};
pub use self::savefile::SaveFile;
#[allow(unused_imports)]
pub use self::worldwindow::{
    use_world_chooser_window, WorldChooserWindow, WorldChooserWindowManager,
};

mod dbchoice;
mod dbwindow;
mod id;
pub mod list;
mod manager;
mod meta;
mod savefile;
mod v1storage;
mod worldwindow;

/// A single world with a particular database and structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    /// Which database is used for this world.
    database: DatabaseChoice,
    /// Root node for this world.
    root: Node,
    /// Non-undo metadata about nodes.
    node_metadata: NodeMetas,
    /// Non-undo metadata about this particular world.
    /// This has been superceded by the
    #[deprecated]
    global_metadata: GlobalMetadata,
}

impl World {
    /// Creates a new empty world.
    fn new() -> Self {
        // Need to set the deprecated field on initialization.
        #[allow(deprecated)]
        Self {
            database: Default::default(),
            root: Group::empty_node(),
            node_metadata: Default::default(),
            global_metadata: Default::default(),
        }
    }

    /// Gets the name of this world from the root group.
    fn name(&self) -> AttrValue {
        match self.root.group() {
            Some(root) => root.name.clone(),
            None => {
                warn!("Cannot get world name: root was not a group!");
                "<Error: Root is not a Group>".into()
            }
        }
    }

    /// Gets WorldMetadata for this world.
    fn metadata(&self) -> WorldMetadata {
        WorldMetadata {
            name: self.name(),
            database: self.database.version_selector(),
            // An existing World should never have a load_error.
            load_error: false,
        }
    }

    /// Performs the world post-load actions. This fetches the current database, then rebuilds the
    /// root node in place (without creating an undo state). It then returns the database.
    fn post_load(&mut self) -> Database {
        let db = self.database.get();
        self.root = self.root.rebuild(&db);
        db
    }
}

/// Metadata about a particular world.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct GlobalMetadata {
    /// Whether to hide or show empty balances in group balances.
    ///
    /// This field has been moved to [`UserSettings`]. The field here is only used for
    /// backwards compatibility, so when migrating to v1.2.0 or later from an earlier
    /// version we can pull the user's hide_empty_balances setting from the GlobalMetadata
    /// of the selected world.
    #[deprecated]
    hide_empty_balances: bool,
}
