// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use log::warn;
use satisfactory_accounting::database::{BuildingId, BuildingKind, ItemId};
use yew::prelude::*;

use crate::node_display::building::choose_from_list::{Choice, ChooseFromList};
use crate::node_display::icon::Icon;
use crate::CtxHelper;

#[derive(PartialEq, Properties)]
pub struct Props {
    /// Building used to choose which recipes are available.
    pub building_id: BuildingId,
    /// ID of the selected building, if any.
    pub item_id: Option<ItemId>,
    /// Callback to change the type of this building.
    pub change_item: Callback<ItemId>,
}

/// Messages for [`BuildingTypeDisplay`]
pub enum Msg {
    /// Switches in or out of editing.
    ToggleEdit {
        /// The new editing state.
        editing: bool,
    },
    /// Select a new item ID.
    Select {
        /// The new ID.
        id: ItemId,
    },
}

/// Displays and allows selection of the Building's item (fuel or resource).
#[derive(Default)]
pub struct ItemDisplay {
    /// Whether an item is currently being entered.
    editing: bool,
}

impl Component for ItemDisplay {
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
                ctx.props().change_item.emit(id);
                self.editing = false;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let db = ctx.db();
        let &Props {
            building_id,
            item_id,
            ..
        } = ctx.props();
        let building = match db.get(building_id) {
            None => {
                warn!(
                    "Cannot show items for building {}, it is unknown",
                    building_id
                );
                return html! {};
            }
            Some(building) => building,
        };
        let (items, title) = match &building.kind {
            BuildingKind::Miner(m) => (&m.allowed_resources, "Mined Resource"),
            BuildingKind::Generator(g) => (&g.allowed_fuel, "Consumed Fuel"),
            BuildingKind::Pump(p) => (&p.allowed_resources, "Extracted Resource"),
            BuildingKind::Station(s) => (&s.allowed_fuel, "Consumed Fuel"),
            _ => {
                warn!(
                    "Cannot show items for building with kind {:?}",
                    building.kind.kind_id()
                );
                return html! {};
            }
        };
        let link = ctx.link();
        if self.editing {
            let choices: Vec<_> = items
                .iter()
                .map(|&item_id| match db.get(item_id) {
                    Some(item) => Choice {
                        id: item.id,
                        name: item.name.clone(),
                        image: html! {
                            <Icon icon={item.image.clone()} alt={item.name.clone()} />
                        },
                    },
                    None => Choice {
                        id: item_id,
                        name: format!("Unknown Item {}", item_id).into(),
                        image: html! { <Icon /> },
                    },
                })
                .collect();

            let selected = link.callback(|id| Msg::Select { id });
            let cancelled = link.callback(|()| Msg::ToggleEdit { editing: false });
            html! {
                <span class="name" {title}>
                    <ChooseFromList<ItemId> {choices} {selected} {cancelled} />
                </span>
            }
        } else {
            let edit = if items.len() > 1 {
                Some(link.callback(|_| Msg::ToggleEdit { editing: true }))
            } else {
                None
            };
            match item_id {
                None => html! {
                    <span class="name" {title} onclick={edit}>{"select item"}</span>
                },
                Some(id) => match db.get(id) {
                    None => html! {
                        <span class="name" {title} onclick={edit}>
                            <Icon />
                            <span>{"Unknown Item "}{id}</span>
                        </span>
                    },
                    Some(building) => html! {
                        <span class="name" {title} onclick={edit}>
                            <Icon icon={building.image.clone()}
                                alt={building.name.clone()} />
                            <span>{&building.name}</span>
                        </span>
                    },
                },
            }
        }
    }
}
