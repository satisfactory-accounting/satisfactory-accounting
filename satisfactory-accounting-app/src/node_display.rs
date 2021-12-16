use log::warn;
use yew::prelude::*;

use satisfactory_accounting::accounting::{Node, NodeKind};

mod balance;
mod building;
mod drag;
mod graph_manipulation;
mod group;

#[derive(Debug, PartialEq, Properties)]
pub struct NodeDisplayProperties {
    /// The node to display.
    pub node: Node,
    /// Path to this node in the tree.
    pub path: Vec<usize>,
    /// Callback to tell the parent to delete this node.
    #[prop_or_default]
    pub delete: Option<Callback<usize>>,
    /// Callback to tell the parent to replace this node.
    pub replace: Callback<(usize, Node)>,
    /// Callback to tell the parent to move a node.
    pub move_node: Callback<(Vec<usize>, Vec<usize>)>,
}

/// Messages which can be sent to a Node.
pub enum NodeMsg {
    // Messages for groups:
    /// Replace the child at the given index with the specified node.
    ReplaceChild { idx: usize, replacement: Node },
    /// Delete the child at the specified index.
    DeleteChild { idx: usize },
    /// Add the given node as a child at the end of the list.
    AddChild { child: Node },
    /// Rename this node.
    Rename { name: String },
    /// When another node is dragged over this one.
    DragOver { insert_pos: usize },
    /// When another dragging node leaves this one.
    DragLeave,
    MoveNode {
        src_path: Vec<usize>,
        dest_path: Vec<usize>,
    },
}

/// Display for a single AccountingGraph node.
#[derive(Default)]
pub struct NodeDisplay {
    /// Element where children are attached.
    children: NodeRef,
    /// When a drag is in progress and over our children area, this is the proposed insert
    /// position.
    insert_pos: Option<usize>,
}

impl Component for NodeDisplay {
    type Message = NodeMsg;
    type Properties = NodeDisplayProperties;

    fn create(_: &Context<Self>) -> Self {
        Default::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let our_idx = ctx.props().path.last().copied().unwrap_or_default();
        match msg {
            NodeMsg::ReplaceChild { idx, replacement } => {
                if let NodeKind::Group(group) = ctx.props().node.kind() {
                    if idx < group.children.len() {
                        let mut new_group = group.clone();
                        new_group.children[idx] = replacement;
                        ctx.props().replace.emit((our_idx, new_group.into()));
                    } else {
                        warn!(
                            "Cannot replace child index {}; out of range for this group",
                            idx
                        );
                    }
                } else {
                    warn!("Cannot replace child of a non-group");
                }
                false
            }
            NodeMsg::DeleteChild { idx } => {
                if let NodeKind::Group(group) = ctx.props().node.kind() {
                    if idx < group.children.len() {
                        let mut new_group = group.clone();
                        new_group.children.remove(idx);
                        ctx.props().replace.emit((our_idx, new_group.into()));
                    } else {
                        warn!(
                            "Cannot delete child index {}; out of range for this group",
                            idx
                        );
                    }
                } else {
                    warn!("Cannot delete child of a non-group");
                }
                false
            }
            NodeMsg::AddChild { child } => {
                if let NodeKind::Group(group) = ctx.props().node.kind() {
                    let mut new_group = group.clone();
                    new_group.children.push(child);
                    ctx.props().replace.emit((our_idx, new_group.into()));
                } else {
                    warn!("Cannot add child to a non-group");
                }
                false
            }
            NodeMsg::Rename { name } => {
                if let NodeKind::Group(group) = ctx.props().node.kind() {
                    let name = name.trim().to_owned();
                    if name != group.name {
                        let mut new_group = group.clone();
                        new_group.name = name;
                        ctx.props().replace.emit((our_idx, new_group.into()));
                    }
                } else {
                    warn!("Cannot rename a non-group");
                }
                false
            }
            NodeMsg::DragOver { insert_pos } => {
                self.insert_pos = Some(insert_pos);
                true
            }
            NodeMsg::DragLeave => {
                self.insert_pos = None;
                true
            }
            NodeMsg::MoveNode {
                src_path,
                dest_path,
            } => {
                let path = &ctx.props().path[..];
                let prefix_len = path.len();
                debug_assert!(
                    prefix_len < dest_path.len(),
                    "Got asked to move a node for a parent."
                );
                if prefix_len < src_path.len()
                    && path == &src_path[..prefix_len]
                    && path == &dest_path[..prefix_len]
                {
                    // This node is the common ancestor of the source and destination
                    // paths.
                    if let NodeKind::Group(group) = ctx.props().node.kind() {
                        if let Some(new_group) = graph_manipulation::move_child(
                            group,
                            &src_path[prefix_len..],
                            &dest_path[prefix_len..],
                        ) {
                            ctx.props().replace.emit((our_idx, new_group.into()));
                        }
                    } else {
                        warn!("Attempting to move nodes in a non-group.");
                    }
                } else {
                    // No common ancestor yet, ask parent to do the move.
                    ctx.props().move_node.emit((src_path, dest_path));
                }
                if self.insert_pos.is_some() {
                    self.insert_pos = None;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match ctx.props().node.kind() {
            NodeKind::Group(group) => self.view_group(ctx, group),
            NodeKind::Building(building) => self.view_building(ctx, building),
        }
    }
}

/// CSS class that identifies children which identifies the `div` which marks where an
/// element will be dropped. Used to avoid having the insert point count towards the
/// index being chosen for insertion when searching children to figure out what index the
/// drop is at. Also used to style the insert point.
const DRAG_INSERT_POINT: &str = "drag-insert-point";

impl NodeDisplay {
    /// Creates the delete button, if the parent allows this node to be deleted.
    fn delete_button(&self, ctx: &Context<Self>) -> Html {
        match ctx.props().delete.clone() {
            Some(delete_from_parent) => {
                let idx = ctx
                    .props()
                    .path
                    .last()
                    .copied()
                    .expect("Parent provided a delete callback, but this is the root node.");
                let onclick = Callback::from(move |_| delete_from_parent.emit(idx));
                html! {
                    <button {onclick} class="delete" title="Delete">
                        <span class="material-icons">{"delete"}</span>
                    </button>
                }
            }
            None => html! {},
        }
    }
}

/// Get the icon path for a given slug name.
fn slug_to_icon(slug: impl AsRef<str>) -> String {
    let mut icon = slug.as_ref().to_owned();
    icon.insert_str(0, "/images/items/");
    icon.push_str("_64.png");
    icon
}

/// Get a span to use when an icon is unknown.
fn icon_missing() -> Html {
    html! {
        <span class="material-icons error">{"error"}</span>
    }
}
