// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use satisfactory_accounting::accounting::ResourcePurity;
use yew::prelude::*;

use crate::inputs::choose_from_list::{Choice, ChooseFromList};

#[derive(PartialEq, Properties)]
pub struct Props {
    /// Currently selected node purity.
    pub purity: ResourcePurity,
    /// Callback to update the purity.
    pub set_purity: Callback<ResourcePurity>,
}

#[function_component]
pub fn Purity(Props { purity, set_purity }: &Props) -> Html {
    let editing = use_state_eq(|| false);
    let setter = editing.setter();

    let selected = use_callback(
        (setter.clone(), set_purity.clone()),
        |id, (setter, set_purity)| {
            setter.set(false);
            set_purity.emit(id);
        },
    );
    let cancelled = use_callback(setter.clone(), |(), setter| setter.set(false));
    let edit = use_callback(setter, |_, setter| setter.set(true));

    let choices = create_purity_choices();

    if *editing {
        html! {
            <ChooseFromList<ResourcePurity> class="Purity" title="Resource Node Purity"
                {choices} {selected} {cancelled} />
        }
    } else {
        html! {
            <div class="Purity" onclick={edit} title="Resource Node Purity">
                {purity_icon(*purity)}
                <span>{purity.name()}</span>
            </div>
        }
    }
}

fn create_purity_choices() -> Vec<Choice<ResourcePurity>> {
    ResourcePurity::values()
        .map(|purity| Choice {
            id: purity,
            name: purity.name().into(),
            image: purity_icon(purity),
        })
        .collect()
}

pub fn purity_icon(purity: ResourcePurity) -> Html {
    match purity {
        ResourcePurity::Impure => html! {
            <span class="purity-icon material-icons impure-node">
                {"remove_circle"}
            </span>
        },
        ResourcePurity::Normal => html! {
            <span class="purity-icon material-icons normal-node">
                {"circle"}
            </span>
        },
        ResourcePurity::Pure => html! {
            <span class="purity-icon material-icons pure-node">
                {"add_circle"}
            </span>
        },
    }
}
