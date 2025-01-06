use log::warn;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::inputs::events::get_value_from_input_event;
use crate::inputs::whitespace::space_to_nbsp;

/// Direction of the adjustment to apply.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AdjustDir {
    Up,
    Down,
}

/// The scale of a value adjustment.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AdjustScale {
    /// A fine adjustment (arrow keys)
    Fine,
    /// A coarse adjustment (pg up/down keys).
    Coarse,
}

/// A modifier to add to the adjustment scale.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AdjustModifier {
    /// No modifier on the scale.
    None,
    /// Adjust by a smaller amount than the scale.
    Smaller,
}

impl AdjustModifier {
    /// Interpret keys pressed on the given keybaord event as a modifier.
    fn interpret(e: &KeyboardEvent) -> Self {
        if e.shift_key() {
            Self::Smaller
        } else {
            Self::None
        }
    }
}

/// An adjustment to apply to the value, rather than by typing in full numbers.
#[derive(Debug, Copy, Clone)]
pub struct ValueAdjustment {
    /// Whether to adjust up or down.
    pub dir: AdjustDir,
    /// What scale of adjustment to make.
    pub scale: AdjustScale,
    /// Modifier to apply to the scale.
    pub modifier: AdjustModifier,
}

#[derive(Debug, Properties, PartialEq)]
pub struct Props {
    /// Last committed value.
    pub value: AttrValue,
    /// Rounded value. Displayed when *not* editing.
    #[prop_or_default]
    pub rounded_value: Option<AttrValue>,
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
    /// Callback to allow small adjustments of the value by itting keys like pg up, pg down, or
    /// up/down. This callback takes the adjustment info and the current value and emits an updated
    /// editable value.
    #[prop_or_default]
    pub adjust: Option<fn(ValueAdjustment, AttrValue) -> AttrValue>,
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
    /// Adjust the value by a given amount.
    Adjust { adjustment: ValueAdjustment },
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
                "Up" | "ArrowUp" => Some(Msg::Adjust {
                    adjustment: ValueAdjustment {
                        dir: AdjustDir::Up,
                        scale: AdjustScale::Fine,
                        modifier: AdjustModifier::interpret(&e),
                    },
                }),
                "Down" | "ArrowDown" => Some(Msg::Adjust {
                    adjustment: ValueAdjustment {
                        dir: AdjustDir::Down,
                        scale: AdjustScale::Fine,
                        modifier: AdjustModifier::interpret(&e),
                    },
                }),
                "PageUp" => Some(Msg::Adjust {
                    adjustment: ValueAdjustment {
                        dir: AdjustDir::Up,
                        scale: AdjustScale::Coarse,
                        modifier: AdjustModifier::interpret(&e),
                    },
                }),
                "PageDown" => Some(Msg::Adjust {
                    adjustment: ValueAdjustment {
                        dir: AdjustDir::Down,
                        scale: AdjustScale::Coarse,
                        modifier: AdjustModifier::interpret(&e),
                    },
                }),
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
            return true;
        }
        // Skip re-rendering if only the callback has changed.
        new_props.value != old_props.value
            || new_props.rounded_value != old_props.rounded_value
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
            Msg::Adjust { adjustment } => {
                match (ctx.props().adjust.as_ref(), self.edit_text.take()) {
                    (Some(adjuster), Some(value)) => {
                        self.edit_text = Some(adjuster(adjustment, value));
                        true
                    }
                    (None, Some(text)) => {
                        self.edit_text = Some(text);
                        false
                    }
                    (_, None) => {
                        warn!("Adjust while not editing");
                        false
                    }
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let Props {
            value,
            rounded_value,
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
                        <div class="value-display">
                            {space_to_nbsp(&value)}
                            if value.is_empty() {
                                {"\u{00a0}"}
                            }
                        </div>
                    </div>
                    { suffix.clone() }
                </form>
            }
        } else {
            let onclick = self.onclick.clone();
            let value = rounded_value.as_ref().unwrap_or(value);
            html! {
                <div {class} {title} {onclick}>
                    { prefix.clone() }
                    <div class="value">
                        <div class="value-display">
                            {value}
                            if value.is_empty() {
                                {"\u{00a0}"}
                            }
                        </div>
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
