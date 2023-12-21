use log::warn;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::events::get_value_from_input_event;

#[derive(PartialEq, Properties)]
pub struct Props {
    /// Last committed value.
    pub value: AttrValue,
    /// Title for the node.
    pub title: AttrValue,
    /// Extra classes to apply to the container div.
    #[prop_or_default]
    pub class: Classes,
    /// Extra HTML element to show before the editable text.
    #[prop_or_default]
    pub prefix: Html,
    /// Extra HTML element to show after the editable text.
    #[prop_or_default]
    pub suffix: Html,
    /// Callback to invoke when the edit is committed.
    pub on_commit: Callback<AttrValue>,
}

pub enum Msg {
    /// Message during editing to update the edited text.
    UpdateInput { input: AttrValue },
    /// Message while not editing to start editing.
    StartEdit,
    /// Message to finish editing.
    FinishEdit,
    /// Cancel editing without changing the value.
    Cancel,
}

/// Helper to display some text with click-to-edit.
pub struct ClickEdit {
    /// Pending edit text if clock speed is being changed.
    edit_text: Option<AttrValue>,
    /// Whether we did focus since last committing an edit.
    did_focus: bool,
    /// Input to focus on editing.
    input: NodeRef,

    // Memoized callbacks:
    oninput: Callback<InputEvent>,
    onkeyup: Callback<KeyboardEvent>,
    onblur: Callback<FocusEvent>,
    onsubmit: Callback<SubmitEvent>,
    onclick: Callback<MouseEvent>,

    // Memoized classes.
    class: Classes,
}

impl Component for ClickEdit {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link();
        ClickEdit {
            edit_text: None,
            did_focus: true,
            input: NodeRef::default(),

            oninput: link.callback(|input| Msg::UpdateInput {
                input: get_value_from_input_event(input),
            }),
            onkeyup: link.batch_callback(|e: KeyboardEvent| match &*e.key() {
                "Esc" | "Escape" => Some(Msg::Cancel),
                _ => None,
            }),
            onblur: link.callback(|_| Msg::FinishEdit),
            onsubmit: link.callback(|e: SubmitEvent| {
                e.prevent_default();
                Msg::FinishEdit
            }),
            onclick: link.callback(|_| Msg::StartEdit),

            class: classes!("ClickEdit", ctx.props().class.clone()),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let new_props = ctx.props();
        if new_props.class != old_props.class {
            self.class = classes!("ClickEdit", new_props.class.clone());
            return true;
        }
        // We only need to rerender if any of the rendered values changed. on_commit is used from
        // within our update method, so we don't need to re-render to pick up changes to it.
        new_props.value != old_props.value
            || new_props.title != old_props.title
            || new_props.prefix != old_props.prefix
            || new_props.suffix != old_props.suffix
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateInput { input } => {
                if self.edit_text.is_none() {
                    warn!("UpdateInput while not editing");
                }
                self.edit_text = Some(input);
                true
            }
            Msg::StartEdit => {
                if self.edit_text.is_some() {
                    warn!("StartEdit while already editing");
                }
                self.edit_text = Some(ctx.props().value.clone());
                self.did_focus = false;
                true
            }
            Msg::FinishEdit => {
                if let Some(edit_text) = self.edit_text.take() {
                    ctx.props().on_commit.emit(edit_text);
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
        let Props { value, title, prefix, suffix, .. } = ctx.props();
        let class = self.class.clone();
        if let Some(value) = self.edit_text.clone() {
            let oninput = self.oninput.clone();
            let onkeyup = self.onkeyup.clone();
            let onblur = self.onblur.clone();
            let onsubmit = self.onsubmit.clone();
            html! {
                <form {class} {title} {onsubmit}>
                    { prefix.clone() }
                    <input class="value" type="text" {value}
                        {oninput} {onblur} {onkeyup} ref={self.input.clone()} />
                    { suffix.clone() }
                </form>
            }
        } else {
            let onclick = self.onclick.clone();
            html! {
                <div {class} {title} {onclick}>
                    { prefix.clone() }
                    <span class="value">{value.to_string()}</span>
                    { suffix.clone() }
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
            } else {
                warn!("Cannot focus the input, no HtmlInputElement");
            }
        }
    }
}
