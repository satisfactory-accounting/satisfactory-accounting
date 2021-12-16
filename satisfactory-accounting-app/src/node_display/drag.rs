use log::warn;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use yew::prelude::*;

use super::{NodeDisplay, NodeMsg, DRAG_INSERT_POINT};

/// Key used to store data about node being transferred in drag events.
const TRANSFER_KEY: &str = "zstewart.satisfactory-accounting/drag-node.path";

impl NodeDisplay {
    /// Get the insert_pos_chooser for this node.
    fn insert_pos_chooser(&self, ctx: &Context<Self>) -> InsertPosChooser {
        let children = self.children.clone();
        let path = ctx.props().path.clone();
        InsertPosChooser { children, path }
    }

    /// Build an event handler for the ondragover event.
    pub fn drag_over_handler(&self, ctx: &Context<Self>) -> Callback<DragEvent> {
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
    pub fn drop_handler(&self, ctx: &Context<Self>) -> Callback<DragEvent> {
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

    /// Creates a drag-handle for this element.
    pub fn drag_handle(&self, ctx: &Context<Self>) -> Html {
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
