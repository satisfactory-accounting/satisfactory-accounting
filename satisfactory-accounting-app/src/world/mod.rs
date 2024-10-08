use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use log::warn;
use satisfactory_accounting::accounting::{Group, Node, NodeKind};
use satisfactory_accounting::database::DatabaseVersion;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use yew::AttrValue;

pub use self::dbchoice::DatabaseChoice;
pub use self::id::{ParseWorldIdError, WorldId};
pub use self::list::{WorldList, WorldMetadata};
pub use self::manager::{
    use_db, use_db_controller, use_undo_controller, DbController, UndoController, UndoDispatcher,
    WorldManager,
};

mod dbchoice;
mod id;
mod list;
mod manager;
mod v1storage;

/// A single world with a particular database and structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct World {
    /// Which database is used for this world.
    database: DatabaseChoice,
    /// Root node for this world.
    root: Node,
    /// Non-undo metadata about nodes.
    node_metadata: NodeMetadata,
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

    /// Get the selected database version or None if the database is custom.
    fn database_version(&self) -> Option<DatabaseVersion> {
        match self.database {
            DatabaseChoice::Standard(version) => Some(version),
            DatabaseChoice::Custom(_) => None,
        }
    }

    /// Gets WorldMetadata for this world.
    fn metadata(&self) -> WorldMetadata {
        WorldMetadata {
            name: self.name(),
            database: self.database_version(),
            // An existing World should never have a load_error.
            load_error: false,
        }
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

/// Mapping of node medatata by node id.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeMetadata(Rc<HashMap<Uuid, NodeMetadatum>>);

impl NodeMetadata {
    /// Get the metadata for a particular node by id.
    pub fn meta(&self, uuid: Uuid) -> NodeMetadatum {
        self.0.get(&uuid).cloned().unwrap_or_default()
    }

    /// Build a version of the metadata with the given value updated.
    pub fn set_meta(&mut self, uuid: Uuid, meta: NodeMetadatum) {
        Rc::make_mut(&mut self.0).insert(uuid, meta);
    }

    /// Build a version of the metadata with the given values updated.
    pub fn batch_update(&mut self, update: impl IntoIterator<Item = (Uuid, NodeMetadatum)>) {
        Rc::make_mut(&mut self.0).extend(update);
    }

    /// Prune metadata for anything that isn't referenced from the given node.
    pub fn prune(&mut self, root: &Node) {
        let used_uuids: HashSet<_> = root
            .iter()
            .filter_map(|node| match node.kind() {
                NodeKind::Group(g) => Some(g.id),
                NodeKind::Building(_) => None,
            })
            .collect();
        Rc::make_mut(&mut self.0).retain(|k, _| used_uuids.contains(k));
    }
}

/// Metadata about a node which isn't stored in the tree and isn't available for
/// undo/redo.
#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeMetadatum {
    /// Whether the node should be shown collapsed or expanded.
    collapsed: bool,
}
