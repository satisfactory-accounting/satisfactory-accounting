// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use log::warn;
use satisfactory_accounting::database::{BuildingId, BuildingKind, RecipeId};
use yew::prelude::*;

use crate::node_display::building::choose_from_list::{Choice, ChooseFromList};
use crate::node_display::icon::Icon;
use crate::CtxHelper;

#[derive(PartialEq, Properties)]
pub struct Props {
    /// Building used to choose which recipes are available.
    pub building_id: BuildingId,
    /// ID of the selected building, if any.
    pub recipe_id: Option<RecipeId>,
    /// Callback to change the type of this building.
    pub change_recipe: Callback<RecipeId>,
}

/// Messages for [`BuildingTypeDisplay`]
pub enum Msg {
    /// Switches in or out of editing.
    ToggleEdit {
        /// The new editing state.
        editing: bool,
    },
    /// Select a new recipe ID.
    Select {
        /// The new ID.
        id: RecipeId,
    },
}

/// Displays and allows selection of the Building's recipe.
#[derive(Default)]
pub struct RecipeDisplay {
    /// Whether a recipe is currently being entered.
    editing: bool,
}

impl Component for RecipeDisplay {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Default::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleEdit { editing } => {
                self.editing = editing;
                true
            }
            Msg::Select { id } => {
                ctx.props().change_recipe.emit(id);
                self.editing = false;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let db = ctx.db();
        let &Props {
            building_id,
            recipe_id,
            ..
        } = ctx.props();
        let building = match db.get(building_id) {
            None => {
                warn!(
                    "Cannot show recipes for building {}, it is unknown",
                    building_id
                );
                return html! {};
            }
            Some(building) => building,
        };
        let recipes = if let BuildingKind::Manufacturer(m) = &building.kind {
            &m.available_recipes
        } else {
            warn!(
                "Cannot show recipes for building with kind {:?}",
                building.kind.kind_id()
            );
            return html! {};
        };
        let link = ctx.link();
        if self.editing {
            let choices: Vec<_> = recipes
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
                .collect();

            let selected = link.callback(|id| Msg::Select { id });
            let cancelled = link.callback(|()| Msg::ToggleEdit { editing: false });
            html! {
                <span class="name" title="Recipe">
                    <ChooseFromList<RecipeId> {choices} {selected} {cancelled} />
                </span>
            }
        } else {
            let edit = if recipes.len() > 1 {
                Some(link.callback(|_| Msg::ToggleEdit { editing: true }))
            } else {
                None
            };
            match recipe_id {
                None => html! {
                    <span class="name" title="Recipe" onclick={edit}>{"select recipe"}</span>
                },
                Some(id) => match db.get(id) {
                    None => html! {
                        <span class="name" title="Recipe" onclick={edit}>
                            <Icon />
                            <span>{"Unknown Recipe "}{id}</span>
                        </span>
                    },
                    Some(building) => html! {
                        <span class="name" title="Recipe" onclick={edit}>
                            <Icon icon={building.image.clone()} />
                            <span>{&building.name}</span>
                        </span>
                    },
                },
            }
        }
    }
}
