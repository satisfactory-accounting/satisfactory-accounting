// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use satisfactory_accounting::database::{BuildingId, Database};
use yew::prelude::*;

use crate::context::use_db;
use crate::inputs::choose_from_list::{Choice, ChooseFromList};
use crate::node_display::icon::Icon;

#[derive(PartialEq, Properties)]
pub struct Props {
    /// ID of the selected building, if any.
    pub id: Option<BuildingId>,
    /// Callback to change the type of this building.
    pub change_type: Callback<BuildingId>,
}

/// Displays and allows selection of the Building's Type (BuildingId).
#[function_component]
pub fn BuildingTypeDisplay(Props { id, change_type }: &Props) -> Html {
    let db = use_db();

    let editing = use_state_eq(|| false);
    let setter = editing.setter();

    let selected = use_callback(
        (setter.clone(), change_type.clone()),
        |id, (setter, change_type)| {
            setter.set(false);
            change_type.emit(id);
        },
    );
    let cancelled = use_callback(setter.clone(), |(), setter| setter.set(false));
    let edit = use_callback(setter, |_, setter| setter.set(true));

    if *editing {
        let choices = create_building_choices(&db);
        html! {
            <ChooseFromList<BuildingId> class="BuildingTypeDisplay" title="Building Type"
                {choices} {selected} {cancelled} />
        }
    } else {
        match id {
            None => html! {
                <div class="BuildingTypeDisplay" onclick={edit}>
                    {"select building"}
                </div>
            },
            Some(id) => match db.get(*id) {
                None => html! {
                    <div class="BuildingTypeDisplay" title="Building Type" onclick={edit}>
                        <Icon />
                        <span>{"Unknown Building "}{id}</span>
                    </div>
                },
                Some(building) => html! {
                    <div class="BuildingTypeDisplay" title="Building Type" onclick={edit}>
                        <Icon icon={building.image.clone()}/>
                        <span>{&building.name}</span>
                    </div>
                },
            },
        }
    }
}

fn create_building_choices(db: &Database) -> Vec<Choice<BuildingId>> {
    db.buildings
        .values()
        .map(|building| Choice {
            id: building.id,
            name: building.name.clone().into(),
            image: html! {
                <Icon icon={building.image.clone()}/>
            },
        })
        .collect()
}