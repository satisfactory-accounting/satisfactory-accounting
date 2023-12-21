// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use yew::prelude::*;

use crate::clickedit::ClickEdit;

#[derive(Debug, PartialEq, Properties)]
pub struct Props {
    /// Last set value for the clock speed.
    pub consumption: f32,
    /// Callback to change the actual value.
    pub update_consumption: Callback<f32>,
}

#[function_component]
pub fn StationConsumption(props: &Props) -> Html {
    let on_commit = use_callback(
        props.update_consumption.clone(),
        |edit_text: AttrValue, update_consumption| {
            if let Ok(value) = edit_text.parse::<f32>() {
                update_consumption.emit(value.max(0.0));
            }
        },
    );

    let value: AttrValue = props.consumption.to_string().into();
    let prefix = html! {
        <span class="material-icons">{"trending_down"}</span>
    };
    html! {
        <ClickEdit {value} class="StationConsumption" title="Fuel Consumption of Fueled Vehicles"
            {on_commit} {prefix} />
    }
}
