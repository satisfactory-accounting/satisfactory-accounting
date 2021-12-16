use log::warn;
use satisfactory_accounting::accounting::{Building, Group};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::node_display::NodeMsg;

use super::{NodeDisplay, DRAG_INSERT_POINT};

impl NodeDisplay {
    /// Build the display for a Group.
    pub(super) fn view_group(&self, ctx: &Context<Self>, group: &Group) -> Html {
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
