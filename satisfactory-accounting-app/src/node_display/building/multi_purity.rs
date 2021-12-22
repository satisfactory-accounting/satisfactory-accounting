use log::warn;
use satisfactory_accounting::accounting::ResourcePurity;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::node_display::{building::purity::purity_icon, get_value_from_input_event};

#[derive(Debug, PartialEq, Properties)]
pub struct Props {
    /// Resource purity being set.
    pub purity: ResourcePurity,
    /// Last set value for the number of pads of this type.
    pub num_pads: u32,
    /// Callback to change the actual value.
    pub update_pads: Callback<(ResourcePurity, u32)>,
}

pub enum Msg {
    /// Message during editing to update the edited text.
    UpdateInput { input: String },
    /// Message while not editing to start editing.
    StartEdit { input: u32 },
    /// Message to finish editing.
    FinishEdit,
}

/// Display and editing for one purity type on a node that supports multiple.
#[derive(Default)]
pub struct MultiPurity {
    /// Pending edit text if clock speed is being changed.
    edit_text: Option<String>,
    /// Whether we did focus since last committing an edit.
    did_focus: bool,
    /// Input to focus on editing.
    input: NodeRef,
}

impl Component for MultiPurity {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Default::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateInput { input } => {
                self.edit_text = Some(input);
                true
            }
            Msg::StartEdit { input } => {
                self.edit_text = Some(input.to_string());
                self.did_focus = false;
                true
            }
            Msg::FinishEdit => {
                if let Some(edit_text) = self.edit_text.take() {
                    if let Ok(value) = edit_text.parse::<u32>() {
                        let purity = ctx.props().purity;
                        ctx.props().update_pads.emit((purity, value));
                    }
                    true
                } else {
                    warn!("FinishEdit while not editing");
                    false
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let purity = ctx.props().purity;
        if let Some(edit_text) = &self.edit_text {
            let oninput = link.callback(|input| Msg::UpdateInput {
                input: get_value_from_input_event(input),
            });
            let onblur = link.callback(|_| Msg::FinishEdit);
            let onsubmit = link.callback(|e: FocusEvent| {
                e.prevent_default();
                Msg::FinishEdit
            });
            html! {
                <form class="MultiPurity" {onsubmit}>
                    {purity_icon(purity)}
                    <input class="current-num-pads" type="text" value={edit_text.clone()}
                        {oninput} {onblur} ref={self.input.clone()} />
                </form>
            }
        } else {
            let value = ctx.props().num_pads;
            let onclick = link.callback(move |_| Msg::StartEdit { input: value });
            html! {
                <div class="MultiPurity" {onclick}
                    title={format!("Number of {} Nodes", purity.name())}>
                    {purity_icon(purity)}
                    <span class="current-num-pads">{value.to_string()}</span>
                </div>
            }
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if !self.did_focus {
            if let Some(input) = self.input.cast::<HtmlInputElement>() {
                if let Err(e) = input.focus() {
                    warn!("Failed to focus input: {:?}", e);
                }
                input.select();
                self.did_focus = true;
            }
        }
    }
}
