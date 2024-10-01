use log::warn;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::inputs::events::get_value_from_input_event;
use crate::inputs::whitespace::space_to_nbsp;

#[derive(Debug, Properties)]
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

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
            && self.title == other.title
            && self.class == other.class
            && self.prefix == other.prefix
            && self.suffix == other.suffix
            // Skip comparing on_commit, as change to on_commit should not trigger a re-draw, and
            // will only affect the next call to update.
    }
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

    // Memoized classes.
    class: Classes,

    // Memoized callbacks:
    oninput: Callback<InputEvent>,
    onkeyup: Callback<KeyboardEvent>,
    onblur: Callback<FocusEvent>,
    onsubmit: Callback<SubmitEvent>,
    onclick: Callback<MouseEvent>,
}

impl ClickEdit {
    /// Recompute the cached classes list.
    fn compute_classes(props: &Props) -> Classes {
        classes!("ClickEdit", props.class.clone())
    }
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

            class: Self::compute_classes(ctx.props()),

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
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let new_props = ctx.props();
        if new_props.class != old_props.class {
            self.class = classes!("ClickEdit", new_props.class.clone());
        }
        // Caller has already checked new_props != old_props, so it's only worthwhile to do
        // additional checks if we can avoid additional comparisons.
        true
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
        let Props {
            value,
            title,
            prefix,
            suffix,
            ..
        } = ctx.props();
        let class = self.class.clone();
        if let Some(value) = self.edit_text.clone() {
            let oninput = self.oninput.clone();
            let onkeyup = self.onkeyup.clone();
            let onblur = self.onblur.clone();
            let onsubmit = self.onsubmit.clone();
            html! {
                <form {class} {title} {onsubmit}>
                    { prefix.clone() }
                    <div class="value">
                        <input class="value-input" type="text" value={&value}
                            {oninput} {onblur} {onkeyup} ref={&self.input} />
                        <div class="value-display">{space_to_nbsp(&value)}</div>
                    </div>
                    { suffix.clone() }
                </form>
            }
        } else {
            let onclick = self.onclick.clone();
            html! {
                <div {class} {title} {onclick}>
                    { prefix.clone() }
                    <div class="value">
                        <div class="value-display">{value}</div>
                    </div>
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
