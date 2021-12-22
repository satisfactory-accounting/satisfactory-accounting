use log::warn;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::node_display::get_value_from_input_event;

#[derive(Debug, PartialEq, Properties)]
pub struct Props {
    /// Last set value for the clock speed.
    pub consumption: f32,
    /// Callback to change the actual value.
    pub update_consumption: Callback<f32>,
}

pub enum Msg {
    /// Message during editing to update the edited text.
    UpdateInput { input: String },
    /// Message while not editing to start editing.
    StartEdit { input: f32 },
    /// Message to finish editing.
    FinishEdit,
    /// Cancel editing without changing the value.
    Cancel,
}

/// Display and editing for clock speed.
#[derive(Default)]
pub struct StationConsumption {
    /// Pending edit text if clock speed is being changed.
    edit_text: Option<String>,
    /// Whether we did focus since last committing an edit.
    did_focus: bool,
    /// Input to focus on editing.
    input: NodeRef,
}

impl Component for StationConsumption {
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
                        ctx.props().update_consumption.emit(value.max(0.0));
                    }
                    true
                } else {
                    warn!("FinishEdit while not editing");
                    false
                }
            }
            Msg::Cancel => {
                self.edit_text = None;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        if let Some(edit_text) = &self.edit_text {
            let oninput = link.callback(|input| Msg::UpdateInput {
                input: get_value_from_input_event(input),
            });
            let onkeyup = link.batch_callback(|e: KeyboardEvent| match &*e.key() {
                "Esc" | "Escape" => Some(Msg::Cancel),
                _ => None,
            });
            let onblur = link.callback(|_| Msg::FinishEdit);
            let onsubmit = link.callback(|e: FocusEvent| {
                e.prevent_default();
                Msg::FinishEdit
            });
            html! {
                <form class="StationConsumption" {onsubmit}
                    title="Fuel Consumption of Fueled Vehicles">
                    <span class="material-icons">{"trending_down"}</span>
                    <input class="current-consumption" type="text" value={edit_text.clone()}
                        {oninput} {onblur} {onkeyup} ref={self.input.clone()} />
                </form>
            }
        } else {
            let value = ctx.props().consumption;
            let onclick = link.callback(move |_| Msg::StartEdit { input: value });
            html! {
                <div class="StationConsumption" {onclick}
                    title="Fuel Consumption of Fueled Vehicles">
                    <span class="material-icons">{"trending_down"}</span>
                    <span class="current-consumption">{value.to_string()}</span>
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
