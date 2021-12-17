use std::marker::PhantomData;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use log::{info, warn};
use satisfactory_accounting::database::BuildingId;
use satisfactory_accounting::{accounting::Building, database::Id};
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlInputElement};
use yew::prelude::*;

use super::{get_value_from_input_event, icon_missing, slug_to_icon, NodeDisplay};
use crate::node_display::NodeMsg;
use crate::GetDb;

impl NodeDisplay {
    /// Build display for a building.
    pub(super) fn view_building(&self, ctx: &Context<Self>, building: &Building) -> Html {
        let link = ctx.link();
        let change_type = link.callback(|id| NodeMsg::ChangeType { id });
        html! {
            <div class="NodeDisplay building">
                <div class="section">
                    {self.drag_handle(ctx)}
                    <BuildingTypeDisplay id={building.building} {change_type} />
                </div>
                <div class="section">
                    {self.view_balance(ctx)}
                    {self.delete_button(ctx)}
                </div>
            </div>
        }
    }
}

#[derive(PartialEq, Properties)]
struct BuildingTypeDisplayProps {
    /// ID of the selected building, if any.
    id: Option<BuildingId>,
    /// Callback to change the type of this building.
    change_type: Callback<BuildingId>,
}

/// Messages for [`BuildingTypeDisplay`]
enum BuildingTypeMsg {
    /// Switches in or out of editing.
    ToggleEdit {
        /// The new editing state.
        editing: bool,
    },
    /// Select a new building ID.
    Select {
        /// The new ID.
        id: BuildingId,
    },
}

/// Displays and allows selection of the Building's Type (BuildingId).
#[derive(Default)]
struct BuildingTypeDisplay {
    /// Whether a building is currently being entered.
    editing: bool,
}

impl Component for BuildingTypeDisplay {
    type Message = BuildingTypeMsg;
    type Properties = BuildingTypeDisplayProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Default::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            BuildingTypeMsg::ToggleEdit { editing } => {
                self.editing = editing;
                true
            }
            BuildingTypeMsg::Select { id } => {
                ctx.props().change_type.emit(id);
                self.editing = false;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let db = ctx.db();
        let link = ctx.link();
        if self.editing {
            let choices: Vec<_> = db
                .buildings
                .values()
                .map(|building| Choice {
                    id: building.id,
                    name: building.name.clone(),
                    image: slug_to_icon(&building.image),
                })
                .collect();

            let selected = link.callback(|id| BuildingTypeMsg::Select { id });
            let cancelled = link.callback(|()| BuildingTypeMsg::ToggleEdit { editing: false });
            html! {
                <span class="name">
                    <ChooseFromList<BuildingId> {choices} {selected} {cancelled} />
                </span>
            }
        } else {
            let edit = link.callback(|_| BuildingTypeMsg::ToggleEdit { editing: true });
            match ctx.props().id {
                None => html! {
                    <span class="name" onclick={edit}>{"select building"}</span>
                },
                Some(id) => match db.get(id) {
                    None => html! {
                        <span class="name" onclick={edit}>
                            {icon_missing()}
                            <span>{"Unknown Building "}{id}</span>
                        </span>
                    },
                    Some(building) => html! {
                        <span class="name" onclick={edit}>
                            <img class="icon"
                                src={slug_to_icon(&building.image)}
                                alt={building.name.clone()} />
                            <span>{&building.name}</span>
                        </span>
                    },
                },
            }
        }
    }
}

/// An option to choose from.
#[derive(PartialEq, Clone, Debug)]
struct Choice<I> {
    /// ID of the choice.
    id: I,
    /// Name of the choice.
    name: String,
    /// Name of the image to show. This should be the actual image, not the slug.
    image: String,
}

#[derive(PartialEq, Properties)]
struct ChooseFromListProps<I: PartialEq> {
    /// Available choices for this chooser.
    choices: Vec<Choice<I>>,
    /// Callback for when an item is chosen.
    selected: Callback<I>,
    /// Callback for when selection is cancelled.
    cancelled: Callback<()>,
}

/// Messages for [`ChooseFromList`].
enum ChooseFromListMsg {
    /// Move up to the previous entry.
    Up,
    /// Move down to the next entry.
    Down,
    /// Mouse hover over a particular item in the list.
    Hover { filtered_idx: usize },
    /// Cancel input.
    Cancel,
    /// Update the entered text value.
    UpdateInput { input: String },
    /// Select the specified item from the filtered list.
    Select { filtered_idx: usize },
}

/// Component for choosing an item from
struct ChooseFromList<I> {
    /// Current text input.
    input: String,
    /// Choice which is currently highlighted.
    highlighted: usize,
    /// Filtered set of choices with their assigned scores.
    filtered: Vec<(i64, Choice<I>)>,
    matcher: SkimMatcherV2,
    /// Input element, for focusing.
    input_ref: NodeRef,
    _phantom: PhantomData<I>,
}

impl<I: Id + 'static> Component for ChooseFromList<I> {
    type Message = ChooseFromListMsg;
    type Properties = ChooseFromListProps<I>;

    fn create(ctx: &Context<Self>) -> Self {
        let mut filtered: Vec<_> = ctx
            .props()
            .choices
            .iter()
            .cloned()
            .map(|choice| (0, choice))
            .collect();
        filtered.sort_by(|(_, c1), (_, c2)| c1.name.cmp(&c2.name));
        Self {
            input: String::new(),
            highlighted: 0,
            filtered,
            matcher: Default::default(),
            input_ref: Default::default(),
            _phantom: PhantomData,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ChooseFromListMsg::Up => {
                if self.highlighted > 0 {
                    self.highlighted -= 1;
                    true
                } else {
                    false
                }
            }
            ChooseFromListMsg::Down => {
                if self.highlighted + 1 < self.filtered.len() {
                    self.highlighted += 1;
                    true
                } else {
                    false
                }
            }
            ChooseFromListMsg::Hover { filtered_idx } => {
                if filtered_idx < self.filtered.len() {
                    self.highlighted = filtered_idx;
                    true
                } else {
                    warn!("Hover over out of bounds index.");
                    false
                }
            }
            ChooseFromListMsg::Cancel => {
                ctx.props().cancelled.emit(());
                false
            }
            ChooseFromListMsg::UpdateInput { input } => {
                if input != self.input {
                    self.input = input;
                    self.filtered = ctx
                        .props()
                        .choices
                        .iter()
                        .filter_map(|choice| {
                            self.matcher
                                .fuzzy_match(&choice.name, &self.input)
                                .map(|score| (score, choice.clone()))
                        })
                        .collect();
                    self.filtered.sort_by(|(s1, c1), (s2, c2)| {
                        s1.cmp(s2).then_with(|| c1.name.cmp(&c2.name))
                    });
                    self.highlighted = 0;
                    true
                } else {
                    false
                }
            }
            ChooseFromListMsg::Select { filtered_idx } => {
                if filtered_idx < self.filtered.len() {
                    ctx.props().selected.emit(self.filtered[filtered_idx].1.id);
                } else {
                    warn!("Tried to select choice outside of filtered items");
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let highlighted = self.highlighted;
        let onkeydown = link.batch_callback(|e: KeyboardEvent| match &*e.key() {
            "Up" | "ArrowUp" => {
                e.prevent_default();
                Some(ChooseFromListMsg::Up)
            }
            "Down" | "ArrowDown" => {
                e.prevent_default();
                Some(ChooseFromListMsg::Down)
            }
            _ => None,
        });
        let onkeyup = link.batch_callback(|e: KeyboardEvent| match &*e.key() {
            "Esc" | "Escape" => Some(ChooseFromListMsg::Cancel),
            _ => None,
        });
        let onblur = link.batch_callback(|e: FocusEvent| {
            info!("blur");
            if let Some(target) = e.related_target() {
                if let Ok(element) = target.dyn_into::<HtmlElement>() {
                    if element.class_list().contains("available-item") {
                        info!("is item, no hide yet");
                        return None;
                    } else {
                        info!("No matching class");
                    }
                } else {
                    info!("Not HtmlElement");
                }
            } else {
                info!("No related target");
            }
            Some(ChooseFromListMsg::Cancel)
        });
        let oninput = link.callback(|input| ChooseFromListMsg::UpdateInput {
            input: get_value_from_input_event(input),
        });
        let onsubmit = link.callback(move |e: FocusEvent| {
            e.prevent_default();
            ChooseFromListMsg::Select {
                filtered_idx: highlighted,
            }
        });
        html! {
            <form class="ChooseFromList" {onsubmit} {onblur}>
                <input type="text" value={self.input.clone()}
                    {onkeydown} {onkeyup} {oninput}
                    ref={self.input_ref.clone()} />
                <div class="available">
                    { for self.filtered.iter().enumerate().map(|(i, (_, item))| {
                        let selected = (i == self.highlighted).then(|| "selected");
                        let onclick = link.callback(move |_| {
                            info!("clicked element");
                            ChooseFromListMsg::Select {
                            filtered_idx: i,
                        }});
                        let onmouseenter = link.callback(move |_| ChooseFromListMsg::Hover {
                            filtered_idx: i,
                        });
                        html! {
                            <div tabindex="-1" class={classes!("available-item", selected)}
                                {onclick} {onmouseenter}>
                                <img class="icon" src={item.image.clone()} />
                                <span>{&item.name}</span>
                            </div>
                        }
                    }) }
                </div>
            </form>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            if let Some(input) = self.input_ref.cast::<HtmlInputElement>() {
                if let Err(e) = input.focus() {
                    warn!("Failed to focus input: {:?}", e);
                }
            }
        }
    }
}
