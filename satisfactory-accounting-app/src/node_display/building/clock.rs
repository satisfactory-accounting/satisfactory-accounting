use log::warn;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::node_display::get_value_from_input_event;

#[derive(Debug, PartialEq, Properties)]
pub struct Props {
    /// Last set value for the clock speed.
    pub clock_speed: f32,
    /// Callback to change the actual value.
    pub update_speed: Callback<f32>,
}

pub enum Msg {
    /// Message during editing to update the edited text.
    UpdateInput { input: String },
    /// Message while not editing to start editing.
    StartEdit { input: f32 },
    /// Message to finish editing.
    FinishEdit,
}

/// Display and editing for clock speed.
#[derive(Default)]
pub struct ClockSpeed {
    /// Pending edit text if clock speed is being changed.
    edit_text: Option<String>,
    /// Whether we did focus since last committing an edit.
    did_focus: bool,
    /// Input to focus on editing.
    input: NodeRef,
}

impl Component for ClockSpeed {
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
                    if let Ok(value) = edit_text.parse::<f32>() {
                        ctx.props().update_speed.emit(value.clamp(0.01, 2.5));
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
                <form class="ClockSpeed" {onsubmit}>
                    <span class="material-icons-outlined" title="Clock Speed">
                        {"timer"}
                    </span>
                    <input class="current-speed" type="text" value={edit_text.clone()}
                        {oninput} {onblur} ref={self.input.clone()} />
                </form>
            }
        } else {
            let value = ctx.props().clock_speed;
            let onclick = link.callback(move |_| Msg::StartEdit { input: value });
            html! {
                <div class="ClockSpeed" title="Clock Speed">
                    <span class="material-icons-outlined">{"timer"}</span>
                    <span class="current-speed" {onclick}>{value.to_string()}</span>
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
