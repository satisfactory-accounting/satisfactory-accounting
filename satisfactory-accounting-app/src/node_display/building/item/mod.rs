// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use log::warn;
use satisfactory_accounting::database::{BuildingId, BuildingKind, Database, ItemId};
use yew::prelude::*;

use crate::world::use_db;
use crate::inputs::choose_from_list::{Choice, ChooseFromList};
use crate::node_display::icon::Icon;

#[derive(PartialEq, Properties)]
pub struct Props {
    /// Building used to choose which recipes are available.
    pub building_id: BuildingId,
    /// ID of the selected building, if any.
    pub item_id: Option<ItemId>,
    /// Callback to change the type of this building.
    pub on_change_item: Callback<ItemId>,
}

/// Displays and allows selection of the Building's item (fuel or resource).
#[function_component]
pub fn ItemDisplay(
    &Props {
        building_id,
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

    let (items, title) = match look_up_items(&db, building_id) {
        Some(i) => i,
        None => return html! {},
    };

    if *editing {
        let choices = create_item_choices(&db, items);

        html! {
            <ChooseFromList<ItemId> class="ItemDisplay" {title} {choices} {on_selected} {on_cancelled} />
        }
    } else {
        // Don't allow editing if only 1 choice is available.
        let edit = (items.len() > 1).then(|| edit);
        match item_id {
            None => html! {
                <div class="ItemDisplay" {title} onclick={edit}>
                    {"select item"}
                </div>
            },
            Some(id) => match db.get(id) {
                None => html! {
                    <div class="ItemDisplay" {title} onclick={edit}>
                        <Icon />
                        <span>{"Unknown Item "}{id}</span>
                    </div>
                },
                Some(item) => html! {
                    <div class="ItemDisplay" {title} onclick={edit}>
                        <Icon icon={item.image.clone()} />
                        <span>{&item.name}</span>
                    </div>
                },
            },
        }
    }
}

fn look_up_items(db: &Database, building_id: BuildingId) -> Option<(&[ItemId], &'static str)> {
    let building = db.get(building_id).or_else(|| {
        warn!(
            "Cannot show items for building {}, it is unknown",
            building_id
        );
        None
    })?;
    match &building.kind {
        BuildingKind::Miner(m) => Some((&m.allowed_resources, "Mined Resource")),
        BuildingKind::Generator(g) => Some((&g.allowed_fuel, "Consumed Fuel")),
        BuildingKind::Pump(p) => Some((&p.allowed_resources, "Extracted Resource")),
        BuildingKind::Station(s) => Some((&s.allowed_fuel, "Consumed Fuel")),
        _ => {
            warn!(
                "Cannot show items for building with kind {:?}",
                building.kind.kind_id()
            );
            None
        }
    }
}

fn create_item_choices(db: &Database, items: &[ItemId]) -> Vec<Choice<ItemId>> {
    items
        .iter()
        .map(|&item_id| match db.get(item_id) {
            Some(item) => Choice {
                id: item.id,
                name: item.name.clone().into(),
                image: html! {
                    <Icon icon={item.image.clone()}/>
                },
            },
            None => Choice {
                id: item_id,
                name: format!("Unknown Item {}", item_id).into(),
                image: html! { <Icon /> },
            },
        })
        .collect()
}
