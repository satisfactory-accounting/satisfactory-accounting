// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use log::warn;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::node_display::get_value_from_input_event;

#[derive(PartialEq, Properties)]
pub struct Props {
    /// Current name of the Node.
    pub name: String,
    /// Callback to rename the node.
    pub rename: Callback<String>,
}

/// Messages for the GroupName component.
pub enum Msg {
    /// Start editing.
    StartEdit,
    /// Cancel editing.
    CancelEdit,
    /// Change the pending value to the given value.
    UpdatePending {
        /// New value of `pending`.
        pending: String,
    },
    /// Save the value by passing it to the parent.
    CommitEdit,
}

#[derive(Default)]
pub struct GroupName {
    /// If currently editing, the edit in progress, or `None` if not editing.
    pending: Option<String>,
    input: NodeRef,
    did_focus: bool,
}

impl Component for GroupName {
    type Message = Msg;
    type Properties = Props;

    fn create(_: &Context<Self>) -> Self {
        Default::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StartEdit => {
                self.pending = Some(ctx.props().name.to_owned());
                self.did_focus = false;
                true
            }
            Msg::CancelEdit => {
                self.pending = None;
                true
            }
            Msg::UpdatePending { pending } => {
                self.pending = Some(pending);
                true
            }
            Msg::CommitEdit => {
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
        let startedit = ctx.link().callback(|_| Msg::StartEdit);
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
                <button class="edit" title="Edit Group Name"
                    onclick={startedit}>
                    <span class="material-icons">{"edit"}</span>
                </button>
            </div>
        }
    }

    fn view_editing(&self, ctx: &Context<Self>, pending: String) -> Html {
        let link = ctx.link();
        let oninput = link.callback(|input| Msg::UpdatePending {
            pending: get_value_from_input_event(input),
        });
        let onkeydown = link.batch_callback(|e: KeyboardEvent| {
            if e.key() == "Escape" {
                e.prevent_default();
                Some(Msg::CancelEdit)
            } else {
                None
            }
        });
        let commitedit = link.callback(|e: FocusEvent| {
            e.prevent_default();
            Msg::CommitEdit
        });
        html! {
            <form class="GroupName" onsubmit={commitedit}>
                <input class="name" type="text" value={pending} {oninput} {onkeydown} ref={self.input.clone()}/>
                <button class="edit" type="submit">
                    <span class="material-icons">{"save"}</span>
                </button>
            </form>
        }
    }
}
