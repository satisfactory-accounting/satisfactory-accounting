use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use satisfactory_accounting::accounting::{Node, NodeKind};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Mapping of node medatata by node id.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeMetas(Rc<HashMap<Uuid, NodeMeta>>);

impl NodeMetas {
    /// Get the metadata for a particular node by id.
    pub fn meta(&self, uuid: Uuid) -> NodeMeta {
        self.0.get(&uuid).cloned().unwrap_or_default()
    }

    /// Build a version of the metadata with the given value updated. If the metada is shared, this
    /// creates a new copy to make it mutable.
    pub(super) fn set_meta(&mut self, uuid: Uuid, meta: NodeMeta) {
        Rc::make_mut(&mut self.0).insert(uuid, meta);
    }

    /// Build a version of the metadata with the given values updated. If the metada is shared, this
    /// creates a new copy to make it mutable.
    pub(super) fn batch_update(&mut self, update: impl IntoIterator<Item = (Uuid, NodeMeta)>) {
        Rc::make_mut(&mut self.0).extend(update);
    }

    /// Prune metadata for anything that isn't referenced from the given node.
    pub(super) fn prune(&mut self, root: &Node) {
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
pub struct NodeMeta {
    /// Whether the node should be shown collapsed or expanded.
    pub collapsed: bool,
}
