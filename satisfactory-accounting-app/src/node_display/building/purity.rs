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

#[derive(Default)]
pub struct Purity {
    /// Whether a value is being chosen (render a select input).
    editing: bool,
}

pub enum Msg {
    /// Switches in or out of editing.
    ToggleEdit { editing: bool },
    /// Select a new purity.
    Select { purity: ResourcePurity },
}

impl Component for Purity {
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
            Msg::Select { purity } => {
                ctx.props().set_purity.emit(purity);
                self.editing = false;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        if self.editing {
            let choices: Vec<_> = ResourcePurity::values()
                .map(|purity| Choice {
                    id: purity,
                    name: purity.name().into(),
                    image: purity_icon(purity),
                })
                .collect();
            let selected = link.callback(|purity| Msg::Select { purity });
            let cancelled = link.callback(|()| Msg::ToggleEdit { editing: false });
            html! {
                <div class="Purity" title="Resource Node Purity">
                    <ChooseFromList<ResourcePurity> {choices} {selected} {cancelled} />
                </div>
            }
        } else {
            let purity = ctx.props().purity;
            let onclick = link.callback(|_| Msg::ToggleEdit { editing: true });
            html! {
                <div class="Purity" {onclick} title="Resource Node Purity">
                    {purity_icon(purity)}
                    <span>{purity.name()}</span>
                </div>
            }
        }
    }
}

pub fn purity_icon(purity: ResourcePurity) -> Html {
    match purity {
        ResourcePurity::Impure => html! {
            <span class="icon material-icons impure-node">
                {"remove_circle"}
            </span>
        },
        ResourcePurity::Normal => html! {
            <span class="icon material-icons normal-node">
                {"circle"}
            </span>
        },
        ResourcePurity::Pure => html! {
            <span class="icon material-icons pure-node">
                {"add_circle"}
            </span>
        },
    }
}
