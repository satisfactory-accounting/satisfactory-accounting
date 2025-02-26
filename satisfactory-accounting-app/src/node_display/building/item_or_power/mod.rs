// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use satisfactory_accounting::database::{Database, ItemIdOrPower};
use yew::prelude::*;

use crate::inputs::choose_from_list::{Choice, ChooseFromList};
use crate::node_display::icon::Icon;
use crate::world::use_db;

#[derive(PartialEq, Properties)]
pub struct Props {
    /// ID of the selected building, if any.
    pub item_id: Option<ItemIdOrPower>,
    /// Callback to change the type of this building.
    pub on_change_item: Callback<ItemIdOrPower>,
}

/// Displays and allows selection of the Building's item (fuel or resource).
#[function_component]
pub fn ItemOrPowerDisplay(
    &Props {
        item_id,
        ref on_change_item,
    }: &Props,
) -> Html {
    let db = use_db();
    let editing = use_state_eq(|| false);
    let setter = editing.setter();

    let on_selected = use_callback(
        (setter.clone(), on_change_item.clone()),
        |id, (setter, on_change_item)| {
            setter.set(false);
            on_change_item.emit(id);
        },
    );
    let on_cancelled = use_callback(setter.clone(), |(), setter| setter.set(false));
    let edit = use_callback(setter, |_, setter| setter.set(true));
    let title = "Item balance to adjust";

    if *editing {
        let choices = create_item_choices(&db);
        html! {
            <ChooseFromList<ItemIdOrPower> class="ItemOrPowerDisplay" {title} {choices} {on_selected} {on_cancelled} />
        }
    } else {
        match item_id {
            None => html! {
                <div class="ItemOrPowerDisplay" {title} onclick={edit}>
                    {"select item"}
                </div>
            },
            Some(ItemIdOrPower::Power) => html! {
                <div class="ItemOrPowerDisplay" {title} onclick={edit}>
                    <Icon icon={"power-line"} />
                    <span>{"Power"}</span>
                </div>
            },
            Some(ItemIdOrPower::Item(id)) => match db.get(id) {
                None => html! {
                    <div class="ItemOrPowerDisplay" {title} onclick={edit}>
                        <Icon />
                        <span>{"Unknown Item "}{id}</span>
                    </div>
                },
                Some(item) => html! {
                    <div class="ItemOrPowerDisplay" {title} onclick={edit}>
                        <Icon icon={item.image.clone()} />
                        <span>{&item.name}</span>
                    </div>
                },
            },
        }
    }
}

fn create_item_choices(db: &Database) -> Vec<Choice<ItemIdOrPower>> {
    let mut choices = Vec::with_capacity(db.items().count() + 1);
    choices.push(Choice {
        id: ItemIdOrPower::Power,
        name: "Power".into(),
        image: html! {
            <Icon icon={"power-line"} />
        },
    });
    choices.extend(db.items().map(|item| Choice {
        id: item.id.into(),
        name: item.name.clone().into(),
        image: html! {
            <Icon icon={item.image.clone()}/>
        },
    }));
    choices
}
