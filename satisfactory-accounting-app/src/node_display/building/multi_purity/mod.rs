// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use satisfactory_accounting::accounting::ResourcePurity;
use yew::prelude::*;

use crate::inputs::clickedit::ClickEdit;
use crate::node_display::building::purity::purity_icon;

#[derive(Debug, PartialEq, Properties)]
pub struct Props {
    /// Resource purity being set.
    pub purity: ResourcePurity,
    /// Last set value for the number of pads of this type.
    pub num_pads: u32,
    /// Callback to change the actual value.
    pub on_update_pads: Callback<(ResourcePurity, u32)>,
}

#[function_component]
pub fn MultiPurity(props: &Props) -> Html {
    let on_commit = use_callback(
        (props.purity, props.on_update_pads.clone()),
        |edit_text: AttrValue, &(purity, ref on_update_pads)| {
            if let Ok(value) = edit_text.parse::<u32>() {
                on_update_pads.emit((purity, value));
            }
        },
    );
    let (prefix, title): &(Html, AttrValue) = &*use_memo(props.purity, |purity| {
        (
            purity_icon(*purity),
            format!("Number of {} Nodes", props.purity.name()).into(),
        )
    });
    let value: &AttrValue = &*use_memo(props.num_pads, |num_pads| num_pads.to_string().into());

    html! {
        <ClickEdit {value} class="MultiPurity" {title} prefix={prefix.clone()} {on_commit} />
    }
}
