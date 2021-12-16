use std::rc::Rc;

use log::warn;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlInputElement};
use yew::prelude::*;

use satisfactory_accounting::accounting::{Building, Group, Node, NodeKind};
use satisfactory_accounting::database::Database;

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
                        if let Some(new_group) =
                            move_child(group, &src_path[prefix_len..], &dest_path[prefix_len..])
                        {
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
            NodeKind::Building(building) => {
                html! {}
            }
        }
    }
}

/// Key used to store data about node being transferred in drag events.
const TRANSFER_KEY: &str = "zstewart.satisfactory-accounting/drag-node.path";
const DRAG_INSERT_POINT: &str = "drag-insert-point";

impl NodeDisplay {
    /// Build the display for a Group.
    fn view_group(&self, ctx: &Context<Self>, group: &Group) -> Html {
        let link = ctx.link();
        let replace =
            link.callback(|(idx, replacement)| NodeMsg::ReplaceChild { idx, replacement });
        let delete = link.callback(|idx| NodeMsg::DeleteChild { idx });
        let move_node = link.callback(|(src_path, dest_path)| NodeMsg::MoveNode {
            src_path,
            dest_path,
        });
        let add_group = link.callback(|_| NodeMsg::AddChild {
            child: Group::empty_node(),
        });
        let add_building = link.callback(|_| NodeMsg::AddChild {
            child: Building::empty_node(),
        });
        let rename = link.callback(|name| NodeMsg::Rename { name });

        let ondragover = self.drag_over_handler(ctx);
        let ondragleave = link.callback(|_| NodeMsg::DragLeave);
        let ondrop = self.drop_handler(ctx);
        html! {
            <div class="NodeDisplay group">
                <div class="header">
                    {self.drag_handle(ctx)}
                    <GroupName name={group.name.clone()} {rename} />
                    {self.delete_button(ctx)}
                </div>
                <div class="body">
                    <div class="children-display"
                        {ondragover} {ondragleave} {ondrop}
                        ref={self.children.clone()}>
                        { for group.children.iter().cloned().enumerate().map(|(i, node)| {
                            let mut path = ctx.props().path.clone();
                            path.push(i);
                            html! {
                                <>
                                    if self.insert_pos == Some(i) {
                                        <div class={DRAG_INSERT_POINT} />
                                    }
                                    <NodeDisplay {node} {path}
                                        replace={replace.clone()}
                                        delete={delete.clone()}
                                        move_node={move_node.clone()} />
                                </>
                            }
                        }) }
                        if self.insert_pos == Some(group.children.len()) {
                            <div class={DRAG_INSERT_POINT} />
                        }
                    </div>
                    {self.view_balance(ctx)}
                </div>
                <div class="footer">
                    <button class="create create-group" onclick={add_group}>
                        <span class="material-icons">{"create_new_folder"}</span>
                    </button>
                    <button class="create create-building" onclick={add_building}>
                        <span class="material-icons">{"add"}</span>
                    </button>
                </div>
            </div>
        }
    }

    /// Get the insert_pos_chooser for this node.
    fn insert_pos_chooser(&self, ctx: &Context<Self>) -> InsertPosChooser {
        let children = self.children.clone();
        let path = ctx.props().path.clone();
        InsertPosChooser { children, path }
    }

    /// Build an event handler for the ondragover event.
    fn drag_over_handler(&self, ctx: &Context<Self>) -> Callback<DragEvent> {
        let chooser = self.insert_pos_chooser(ctx);
        ctx.link().batch_callback(move |e: DragEvent| {
            if let Some((insert_pos, would_stay_in_place, _)) = chooser.choose_insert_pos(&e) {
                // If this is a valid drop point, prevent default to indicate that.
                e.prevent_default();
                // Drop points are nested, so if we're dropping here, we need to stop
                // propagation so we don't get two insert points.
                e.stop_propagation();
                // But if the node would stay in place, hide the drop indicator.
                if would_stay_in_place {
                    // Drag leave event is only used to clear the drop point indicator.
                    Some(NodeMsg::DragLeave)
                } else {
                    Some(NodeMsg::DragOver { insert_pos })
                }
            } else {
                None
            }
        })
    }

    /// Build an event handler for the ondrop event.
    fn drop_handler(&self, ctx: &Context<Self>) -> Callback<DragEvent> {
        let chooser = self.insert_pos_chooser(ctx);
        ctx.link().callback(move |e: DragEvent| {
            if let Some((insert_pos, would_stay_in_place, src_path)) = chooser.choose_insert_pos(&e)
            {
                // If this is a valid drop point, prevent default to indicate that.
                e.prevent_default();
                // Drop points are nested, so if we're dropping here, we need to stop
                // propagation so we don't get two insert points.
                e.stop_propagation();
                if would_stay_in_place {
                    NodeMsg::DragLeave
                } else {
                    let mut dest_path = chooser.path.clone();
                    dest_path.push(insert_pos);
                    NodeMsg::MoveNode {
                        src_path,
                        dest_path,
                    }
                }
            } else {
                // Clear insert marker on an invalid drop.
                NodeMsg::DragLeave
            }
        })
    }

    /// Build the display for a node's balance.
    fn view_balance(&self, ctx: &Context<Self>) -> Html {
        let balance = ctx.props().node.balance();
        let (db, _) = ctx
            .link()
            .context::<Rc<Database>>(Callback::noop())
            .expect("context to be set");
        html! {
            <div class="balance">
                <div class="entry-row">
                    <img class="icon" alt="power" src={slug_to_icon("power-line")} />
                    <div class={classes!("balance-value", balance_style(balance.power))}>
                        {balance.power}
                    </div>
                    { for balance.balances.iter().map(|(&itemid, &rate)| {
                        let (icon, name) = match db.get(itemid) {
                            Some(item) => (slug_to_icon(&item.image), item.name.to_owned()),
                            None => (slug_to_icon("expanded-power-infrastructure"), "unknown".to_owned()),
                        };
                        html! {
                            <div class="entry-row">
                                <img class="icon" alt={name} src={icon} />
                                <div class={classes!("balance-value", balance_style(rate))}>
                                    {rate}
                                </div>
                            </div>
                        }
                    }) }
                </div>
            </div>
        }
    }

    /// Creates a drag-handle for this element.
    fn drag_handle(&self, ctx: &Context<Self>) -> Html {
        if ctx.props().path.is_empty() {
            html! {}
        } else {
            let dragdata = serde_json::to_string(&ctx.props().path).unwrap();
            let ondragstart = Callback::from(move |e: DragEvent| match e.data_transfer() {
                Some(transfer) => {
                    if let Err(e) = transfer.set_data(TRANSFER_KEY, &dragdata) {
                        warn!("Unable to set drag data: {:?}", e);
                    }
                }
                None => {
                    warn!("Unable to get transfer data to set for drag event");
                }
            });
            html! {
                <div class="drag-handle" draggable="true" {ondragstart}>
                    <span class="material-icons">{"drag_handle"}</span>
                </div>
            }
        }
    }

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
                    <button {onclick} class="delete">
                        <span class="material-icons">{"delete"}</span>
                    </button>
                }
            }
            None => html! {},
        }
    }
}

/// Helper to choose an insert position for a Node.
struct InsertPosChooser {
    /// Children ref of the node. Used to find child client rects.
    children: NodeRef,
    /// Path to this node. Used to determine if the given node is a parent of this one.
    path: Vec<usize>,
}

impl InsertPosChooser {
    /// Chose the insert position for the given drag event in the node this chooser is
    /// for.
    ///
    /// Also return a boolean indicating if the given position would leave the node
    /// in the same place. This is used to allow the node to be dropped in the same place
    /// but not show the insert point indicator in that case. Otherwise the insert point
    /// bubbles up to the parent.
    ///
    /// Return the src path to use when finding the element to move.
    fn choose_insert_pos(&self, event: &DragEvent) -> Option<(usize, bool, Vec<usize>)> {
        let src_path = get_transfer_data(event)?;
        // If the source path is longer than ours, the node may be a child or a peer's
        // child, but it cannot be a parent or ourself.
        if src_path.len() <= self.path.len() {
            if src_path == self.path[..src_path.len()] {
                // Source is equal or a prefix of our path, so it is us or our parent.
                return None;
            }
        }

        let children = self.children.cast::<HtmlElement>()?.children();
        let drop_y = event.client_y() as f64;
        let mut child_idx = 0;
        let mut insert_idx = 0;
        while child_idx < children.length() {
            let child = match children.item(child_idx) {
                Some(child) => match child.dyn_into::<HtmlElement>() {
                    Ok(child) => child,
                    Err(e) => {
                        warn!("Unable to cast element {:?} to HtmlElement", e);
                        return None;
                    }
                },
                None => {
                    warn!("Unable to retrieve child {} of node", child_idx);
                    return None;
                }
            };
            if child.class_list().contains(DRAG_INSERT_POINT) {
                // Child is the insertion point marker, not a real child.
                child_idx += 1;
                continue;
            }

            let bounds = child.get_bounding_client_rect();
            let midpoint = bounds.y() + bounds.height() / 2.0;
            if drop_y < midpoint {
                break;
            }
            child_idx += 1;
            insert_idx += 1;
        }
        // If no index was picked so far, insert point is at the end.

        // Figure out if insert point would result in the node staying in the same place.
        if src_path.len() == self.path.len() + 1 && src_path[..self.path.len()] == self.path {
            // node is a child of this node.
            let child_idx = src_path.last().copied().unwrap();
            // Insert places an item in the list position before the specified index.
            // So if a node is being placed before itself, it will stay in the same place.
            // And if it is being placed before the next node, it will also stay in the
            // same place.
            if (child_idx..=child_idx + 1).contains(&insert_idx) {
                return Some((insert_idx, true, src_path));
            }
        }

        Some((insert_idx, false, src_path))
    }
}

/// Retrieve the transfer data from a drag event, if present.
fn get_transfer_data(event: &DragEvent) -> Option<Vec<usize>> {
    match event.data_transfer() {
        Some(transfer) => match transfer.get_data(TRANSFER_KEY) {
            Ok(data) => match serde_json::from_str::<Vec<usize>>(&data) {
                Ok(data) => Some(data),
                Err(err) => {
                    warn!("Unable to parse transfer data: {}", err);
                    None
                }
            },
            Err(err) => {
                warn!("Unable to retrieve transfer data: {:?}", err);
                None
            }
        },
        None => {
            warn!("No transfer available");
            None
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

fn balance_style(balance: f32) -> &'static str {
    if balance < 0.0 {
        "negative"
    } else if balance > 0.0 {
        "positive"
    } else {
        "neutral"
    }
}

#[derive(PartialEq, Properties)]
struct GroupNameProps {
    /// Current name of the Node.
    name: String,
    /// Callback to rename the node.
    rename: Callback<String>,
}

/// Messages for the GroupName component.
enum GroupNameMsg {
    /// Start editing.
    StartEdit,
    /// Change the pending value to the given value.
    UpdatePending {
        /// New value of `pending`.
        pending: String,
    },
    /// Save the value by passing it to the parent.
    CommitEdit,
}

#[derive(Default)]
struct GroupName {
    /// If currently editing, the edit in progress, or `None` if not editing.
    pending: Option<String>,
    input: NodeRef,
    did_focus: bool,
}

impl Component for GroupName {
    type Message = GroupNameMsg;
    type Properties = GroupNameProps;

    fn create(_: &Context<Self>) -> Self {
        Default::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            GroupNameMsg::StartEdit => {
                self.pending = Some(ctx.props().name.to_owned());
                self.did_focus = false;
                true
            }
            GroupNameMsg::UpdatePending { pending } => {
                self.pending = Some(pending);
                true
            }
            GroupNameMsg::CommitEdit => {
                if let Some(pending) = self.pending.take() {
                    ctx.props().rename.emit(pending);
                    true
                } else {
                    warn!("CommitEdit while not editing.");
                    false
                }
            }
        }
    }

    fn changed(&mut self, _: &Context<Self>) -> bool {
        self.pending = None;
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match self.pending.clone() {
            None => self.view_not_editing(ctx),
            Some(pending) => self.view_editing(ctx, pending),
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if !self.did_focus {
            if let Some(input) = self.input.cast::<HtmlInputElement>() {
                if let Err(e) = input.focus() {
                    warn!("Failed to focus input: {:?}", e);
                }
                self.did_focus = true;
            }
        }
    }
}

impl GroupName {
    /// View of the GroupName when not editing.
    fn view_not_editing(&self, ctx: &Context<Self>) -> Html {
        let name = &ctx.props().name;
        let startedit = ctx.link().callback(|_| GroupNameMsg::StartEdit);
        html! {
            <div class="GroupName">
                if name.is_empty() {
                    <span class="name notset" onclick={startedit.clone()}>
                        {"unnamed"}
                    </span>
                } else {
                    <span class="name" onclick={startedit.clone()}>
                        {name}
                    </span>
                }
                <div class="space" />
                <button class="edit" onclick={startedit}>
                    <span class="material-icons">{"edit"}</span>
                </button>
            </div>
        }
    }

    fn view_editing(&self, ctx: &Context<Self>, pending: String) -> Html {
        let link = ctx.link();
        let oninput = link.callback(|input| GroupNameMsg::UpdatePending {
            pending: get_value_from_input_event(input),
        });
        let commitedit = link.callback(|e: FocusEvent| {
            e.prevent_default();
            GroupNameMsg::CommitEdit
        });
        html! {
            <form class="GroupName" onsubmit={commitedit}>
                <input class="name" type="text" value={pending} {oninput} ref={self.input.clone()}/>
                <div class="space" />
                <button class="edit" type="submit">
                    <span class="material-icons">{"save"}</span>
                </button>
            </form>
        }
    }
}

fn get_value_from_input_event(e: InputEvent) -> String {
    let event: Event = e.dyn_into().unwrap();
    let event_target = event.target().unwrap();
    let target: HtmlInputElement = event_target.dyn_into().unwrap();
    target.value()
}

/// Move a node from one position in a group to another. Both src and dest paths should be
/// rooted at this group. Assumes that this node is the lowest common ancestor of src and
/// dest, that is that src and dest have no parents in common below this node.
fn move_child(group: &Group, src: &[usize], dest: &[usize]) -> Option<Group> {
    let (_, src_prefix) = src.split_last().expect("source path was empty");
    let (_, dest_prefix) = dest.split_last().expect("source path was empty");
    assert!(
        src_prefix
            .iter()
            .zip(dest_prefix.iter())
            .take_while(|(s, d)| s == d)
            .count()
            == 0,
        "src and dest had overlapping prefixes"
    );
    let src_first = src.first().copied().unwrap();
    let mut dest_first = dest.first().copied().unwrap();
    if src_prefix.is_empty() && src_first < dest_first {
        // If removal of src will affect dest, change the first index of dest.
        dest_first -= 1;
    }

    if src_first >= group.children.len() {
        warn!("Attempting to move from an out of bounds index");
        return None;
    }

    let mut new_group = group.clone();
    let moved = if src_prefix.is_empty() {
        new_group.children.remove(src_first)
    } else {
        let (replacement, moved) = remove_child(&new_group.children[src_first], &src[1..])?;
        new_group.children[src_first] = replacement;
        moved
    };

    if dest_first > new_group.children.len() {
        warn!("Attempting to move to an out of boudns index");
        return None;
    }

    if dest_prefix.is_empty() {
        new_group.children.insert(dest_first, moved);
    } else {
        new_group.children[dest_first] =
            insert_child(&new_group.children[dest_first], &dest[1..], moved)?;
    }

    Some(new_group)
}

/// Recursively removes a child node. Returns the new group to replace the one modified
/// and the node that was removed. Returns none if not a group or out of bounds.
fn remove_child(node: &Node, child: &[usize]) -> Option<(Node, Node)> {
    let group = match node.kind() {
        NodeKind::Group(group) => group,
        _ => {
            warn!("Source for remove child did not point to a group");
            return None;
        }
    };

    let (&next_idx, rest) = child
        .split_first()
        .expect("Don't call remove_child with an empty path");

    if next_idx >= group.children.len() {
        warn!("Attempting to remove from an out of bounds index");
        return None;
    }
    let mut new_group = group.clone();
    if rest.is_empty() {
        let moved = new_group.children.remove(next_idx);
        Some((new_group.into(), moved))
    } else {
        let (replacement, moved) = remove_child(&new_group.children[next_idx], rest)?;
        new_group.children[next_idx] = replacement;
        Some((new_group.into(), moved))
    }
}

/// Recursively inserts a child node. Returns the new group to replace the one modified.
/// Returns none if not a group or out of bounds.
fn insert_child(node: &Node, child: &[usize], moved: Node) -> Option<Node> {
    let group = match node.kind() {
        NodeKind::Group(group) => group,
        _ => {
            warn!("Source for insert child did not point to a group");
            return None;
        }
    };

    let (&next_idx, rest) = child
        .split_first()
        .expect("Don't call insert_child with an empty path");

    if next_idx > group.children.len() {
        warn!("Attempting to insert to an out of bounds index");
        return None;
    }

    let mut new_group = group.clone();
    if rest.is_empty() {
        new_group.children.insert(next_idx, moved);
    } else {
        new_group.children[next_idx] = insert_child(&new_group.children[next_idx], rest, moved)?;
    }
    Some(new_group.into())
}
