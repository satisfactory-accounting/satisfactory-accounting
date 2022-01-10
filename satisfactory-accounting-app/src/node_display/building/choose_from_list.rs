// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use std::marker::PhantomData;
use std::rc::Rc;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use log::warn;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, HtmlInputElement};
use yew::prelude::*;

use crate::node_display::get_value_from_input_event;

/// An option to choose from.
#[derive(PartialEq, Clone, Debug)]
pub struct Choice<Id> {
    /// ID of the choice.
    pub id: Id,
    /// Name of the choice.
    pub name: Rc<str>,
    /// Name of the image to show. This should be the the slug for the icon.
    pub image: Html,
}

#[derive(PartialEq, Properties)]
pub struct Props<I: PartialEq> {
    /// Available choices for this chooser.
    pub choices: Vec<Choice<I>>,
    /// Callback for when an item is chosen.
    pub selected: Callback<I>,
    /// Callback for when selection is cancelled.
    pub cancelled: Callback<()>,
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
    UpdateInput { input: String },
    /// Select the specified item from the filtered list.
    Select { filtered_idx: usize },
}

/// Component for choosing an item from
pub struct ChooseFromList<I> {
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
                Some(Msg::Up)
            }
            "Down" | "ArrowDown" => {
                e.prevent_default();
                Some(Msg::Down)
            }
            _ => None,
        });
        let onkeyup = link.batch_callback(|e: KeyboardEvent| match &*e.key() {
            "Esc" | "Escape" => Some(Msg::Cancel),
            _ => None,
        });
        let onblur = link.batch_callback(|e: FocusEvent| {
            if let Some(target) = e.related_target() {
                if let Ok(element) = target.dyn_into::<HtmlElement>() {
                    if element.class_list().contains("available-item") {
                        return None;
                    } else {
                    }
                } else {
                }
            } else {
            }
            Some(Msg::Cancel)
        });
        let oninput = link.callback(|input| Msg::UpdateInput {
            input: get_value_from_input_event(input),
        });
        let onsubmit = link.callback(move |e: FocusEvent| {
            e.prevent_default();
            Msg::Select {
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
                        let onclick = link.callback(move |_|
                            Msg::Select {
                            filtered_idx: i,
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
