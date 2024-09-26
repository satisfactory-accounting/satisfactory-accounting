// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use log::warn;
use satisfactory_accounting::database::{BuildingId, BuildingKind, Database, RecipeId};
use yew::prelude::*;

use crate::context::use_db;
use crate::inputs::choose_from_list::{Choice, ChooseFromList};
use crate::node_display::icon::Icon;

#[derive(PartialEq, Properties)]
pub struct Props {
    /// Building used to choose which recipes are available.
    pub building_id: BuildingId,
    /// ID of the selected building, if any.
    pub recipe_id: Option<RecipeId>,
    /// Callback to change the type of this building.
    pub change_recipe: Callback<RecipeId>,
}

/// Displays and allows selection of the Building's recipe.
#[function_component]
pub fn RecipeDisplay(
    &Props {
        building_id,
        recipe_id,
        ref change_recipe,
    }: &Props,
) -> Html {
    let db = use_db();
    let editing = use_state_eq(|| false);
    let setter = editing.setter();

    let selected = use_callback(
        (setter.clone(), change_recipe.clone()),
        |id, (setter, change_recipe)| {
            setter.set(false);
            change_recipe.emit(id);
        },
    );
    let cancelled = use_callback(setter.clone(), |(), setter| setter.set(false));
    let edit = use_callback(setter, |_, setter| setter.set(true));

    let recipes = match look_up_recipes(&db, building_id) {
        Some(r) => r,
        None => return html! {},
    };

    if *editing {
        let choices = create_recipe_choices(&db, recipes);

        html! {
            <span class="RecipeDisplay" title="Recipe">
                <ChooseFromList<RecipeId> {choices} {selected} {cancelled} />
            </span>
        }
    } else {
        // Don't allow editing if only 1 choice is available.
        let edit = (recipes.len() > 1).then(move || edit);
        match recipe_id {
            None => html! {
                <span class="RecipeDisplay" title="Recipe" onclick={edit}>
                    <div class="inner-flex">
                        <span>{"select recipe"}</span>
                    </div>
                </span>
            },
            Some(id) => match db.get(id) {
                None => html! {
                    <span class="RecipeDisplay" title="Recipe" onclick={edit}>
                        <div class="inner-flex">
                            <Icon />
                            <span>{"Unknown Recipe "}{id}</span>
                        </div>
                    </span>
                },
                Some(recipe) => html! {
                    <span class="RecipeDisplay" title="Recipe" onclick={edit}>
                        <div class="inner-flex">
                            <Icon icon={recipe.image.clone()} />
                            <span>{&recipe.name}</span>
                        </div>
                    </span>
                },
            },
        }
    }
}

fn look_up_recipes(db: &Database, building_id: BuildingId) -> Option<&[RecipeId]> {
    let building = db.get(building_id).or_else(|| {
        warn!(
            "Cannot show recipes for building {}, it is unknown",
            building_id
        );
        None
    })?;
    if let BuildingKind::Manufacturer(m) = &building.kind {
        Some(&m.available_recipes)
    } else {
        warn!(
            "Cannot show recipes for building with kind {:?}",
            building.kind.kind_id()
        );
        None
    }
}

fn create_recipe_choices(db: &Database, recipes: &[RecipeId]) -> Vec<Choice<RecipeId>> {
    recipes
        .iter()
        .map(|&recipe_id| match db.get(recipe_id) {
            Some(recipe) => Choice {
                id: recipe.id,
                name: recipe.name.clone().into(),
                image: html! {
                    <Icon icon={recipe.image.clone()} />
                },
            },
            None => Choice {
                id: recipe_id,
                name: format!("Unknown Recipe {}", recipe_id).into(),
                image: html! { <Icon /> },
            },
        })
        .collect()
}
