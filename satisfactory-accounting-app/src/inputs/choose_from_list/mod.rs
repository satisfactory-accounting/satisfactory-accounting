// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use std::marker::PhantomData;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use log::warn;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlInputElement};
use yew::prelude::*;

use crate::inputs::events::get_value_from_input_event;
use crate::inputs::whitespace::space_to_nbsp;

/// An option to choose from.
#[derive(PartialEq, Clone, Debug)]
pub struct Choice<Id> {
    /// ID of the choice.
    pub id: Id,
    /// Name of the choice.
    pub name: AttrValue,
    /// Name of the image to show. This should be the the slug for the icon.
    pub image: Html,
}

#[derive(Properties)]
pub struct Props<I: PartialEq> {
    /// Available choices for this chooser.
    pub choices: Vec<Choice<I>>,
    /// Title to apply to the root of the chooser.
    #[prop_or_default]
    pub title: Option<AttrValue>,
    /// Extra classes to apply.
    #[prop_or_default]
    pub class: Classes,

    /// Callback for when an item is chosen.
    pub selected: Callback<I>,
    /// Callback for when selection is cancelled.
    pub cancelled: Callback<()>,
}

impl<I: PartialEq> PartialEq for Props<I> {
    fn eq(&self, other: &Self) -> bool {
        self.choices == other.choices
            && self.title == other.title
            && self.class == other.class
            // Skip comparing selected and cancelled, as changes to them should not trigger a
            // re-draw, and will only affect the next call to update.
    }
}

/// Messages for [`ChooseFromList`].
pub enum Msg {
    /// Move up to the previous entry.
    Up,
    /// Move down to the next entry.
    Down,
    /// Mouse hover over a particular item in the list.
    Hover { filtered_idx: usize },
    /// Cancel input.
    Cancel,
    /// Update the entered text value.
    UpdateInput { input: AttrValue },
    /// Select the specified item from the filtered list, otherwise select the currently highlighted
    /// item.
    Select { filtered_idx: Option<usize> },
}

/// Component for choosing an item from
pub struct ChooseFromList<I> {
    /// Current text input.
    input: AttrValue,
    /// Choice which is currently highlighted.
    highlighted: usize,
    /// Filtered set of choices with their assigned scores.
    filtered: Vec<(i64, Choice<I>)>,
    matcher: SkimMatcherV2,
    /// Input element, for focusing.
    input_ref: NodeRef,
    _phantom: PhantomData<I>,

    // Cached properties.
    class: Classes,

    // Cached Callbacks
    onkeydown: Callback<KeyboardEvent>,
    onkeyup: Callback<KeyboardEvent>,
    onfocusout: Callback<FocusEvent>,
    oninput: Callback<InputEvent>,
    onsubmit: Callback<SubmitEvent>,
}

impl<I: PartialEq> ChooseFromList<I> {
    // Compute the class list for this item.
    fn compute_classes(props: &Props<I>) -> Classes {
        classes!("ChooseFromList", props.class.clone())
    }
}

impl<I: PartialEq + Copy + Clone + 'static> Component for ChooseFromList<I> {
    type Message = Msg;
    type Properties = Props<I>;

    fn create(ctx: &Context<Self>) -> Self {
        let mut filtered: Vec<_> = ctx
            .props()
            .choices
            .iter()
            .cloned()
            .map(|choice| (0, choice))
            .collect();
        filtered.sort_by(|(_, c1), (_, c2)| c1.name.cmp(&c2.name));

        let link = ctx.link();

        Self {
            input: "".into(),
            highlighted: 0,
            filtered,
            matcher: Default::default(),
            input_ref: Default::default(),
            _phantom: PhantomData,

            class: Self::compute_classes(ctx.props()),

            onkeydown: link.batch_callback(|e: KeyboardEvent| match &*e.key() {
                "Up" | "ArrowUp" => {
                    e.prevent_default();
                    Some(Msg::Up)
                }
                "Down" | "ArrowDown" => {
                    e.prevent_default();
                    Some(Msg::Down)
                }
                _ => None,
            }),
            onkeyup: link.batch_callback(|e: KeyboardEvent| match &*e.key() {
                "Esc" | "Escape" => Some(Msg::Cancel),
                _ => None,
            }),
            onfocusout: link.batch_callback(|e: FocusEvent| {
                if let Some(target) = e.related_target() {
                    if let Ok(element) = target.dyn_into::<HtmlElement>() {
                        if element.class_list().contains("available-item") {
                            return None;
                        }
                    }
                }
                Some(Msg::Cancel)
            }),
            oninput: link.callback(|input| Msg::UpdateInput {
                input: get_value_from_input_event(input),
            }),
            onsubmit: link.callback(|e: SubmitEvent| {
                e.prevent_default();
                Msg::Select { filtered_idx: None }
            }),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Up => {
                if self.highlighted > 0 {
                    self.highlighted -= 1;
                    true
                } else {
                    false
                }
            }
            Msg::Down => {
                if self.highlighted + 1 < self.filtered.len() {
                    self.highlighted += 1;
                    true
                } else {
                    false
                }
            }
            Msg::Hover { filtered_idx } => {
                if filtered_idx < self.filtered.len() {
                    self.highlighted = filtered_idx;
                    true
                } else {
                    warn!("Hover over out of bounds index.");
                    false
                }
            }
            Msg::Cancel => {
                ctx.props().cancelled.emit(());
                false
            }
            Msg::UpdateInput { input } => {
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
            Msg::Select { filtered_idx } => {
                let filtered_idx = filtered_idx.unwrap_or(self.highlighted);
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
        html! {
            <form class={self.class.clone()} onsubmit={&self.onsubmit}
                onfocusout={&self.onfocusout} title={&ctx.props().title}>
                <input type="text" value={&self.input} class="text-input"
                    onkeydown={&self.onkeydown} onkeyup={&self.onkeyup} oninput={&self.oninput}
                    ref={&self.input_ref} />
                <div class="input-size">
                    {space_to_nbsp(&self.input)}
                </div>
                <div class="available">
                    { for self.filtered.iter().enumerate().map(|(i, (_, item))| {
                        let selected = (i == self.highlighted).then(|| "selected");
                        let onclick = link.callback(move |_|
                            Msg::Select {
                            filtered_idx: Some(i),
                        });
                        let onmouseenter = link.callback(move |_| Msg::Hover {
                            filtered_idx: i,
                        });
                        html! {
                            <div tabindex="-1" class={classes!("available-item", selected)}
                                {onclick} {onmouseenter}>
                                {item.image.clone()}
                                <span>{&item.name}</span>
                            </div>
                        }
                    }) }
                </div>
            </form>
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let new_props = ctx.props();
        if new_props.class != old_props.class {
            self.class = Self::compute_classes(new_props);
        }
        // Caller has already checked new_props != old_props, so it's only worthwhile to do
        // additional checks if we can avoid additional comparisons.
        true
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
