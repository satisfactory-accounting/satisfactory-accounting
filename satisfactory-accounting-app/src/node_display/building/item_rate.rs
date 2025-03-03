// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use yew::prelude::*;

use crate::inputs::clickedit::ClickEdit;

#[derive(Debug, PartialEq, Properties)]
pub struct Props {
    /// Last set value for the item rate.
    pub rate: f32,
    /// Callback to change the actual value.
    pub update_rate: Callback<f32>,
    /// Whether negative values are allowed.
    #[prop_or_default]
    pub allow_negative: bool,
    /// Title to apply to the field.
    #[prop_or_default]
    pub title: AttrValue,
}

#[function_component]
pub fn ItemRate(props: &Props) -> Html {
    let on_commit = use_callback(
        (props.update_rate.clone(), props.allow_negative),
        |edit_text: AttrValue, (update_rate, allow_negative)| {
            if let Ok(value) = edit_text.parse::<f32>() {
                update_rate.emit(if *allow_negative {
                    value
                } else {
                    value.max(0.0)
                });
            }
        },
    );

    let value: AttrValue = props.rate.to_string().into();
    let prefix = html! {
        if !props.allow_negative || props.rate < 0.0 {
            <span class="material-icons">{"trending_down"}</span>
        } else {
            <span class="material-icons">{"trending_up"}</span>
        }
    };
    html! {
        <ClickEdit {value} class="ItemRate" title={&props.title} {on_commit} {prefix} />
    }
}
