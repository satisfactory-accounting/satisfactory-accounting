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
    /// Last set value for the clock speed.
    pub clock_speed: f32,
    /// Callback to change the actual value.
    pub update_speed: Callback<f32>,
}

/// Display and editing for clock speed.
#[function_component]
pub fn ClockSpeed(props: &Props) -> Html {
    let on_commit = use_callback(
        props.update_speed.clone(),
        |edit_text: AttrValue, update_speed| {
            if let Ok(value) = edit_text.parse::<f32>() {
                update_speed.emit(value.clamp(0.01, 2.5));
            }
        },
    );

    let value: AttrValue = props.clock_speed.to_string().into();
    let prefix = html! {
        <span class="material-icons-outlined">{"timer"}</span>
    };
    html! {
        <ClickEdit {value} class="ClockSpeed" title="Clock Speed" {on_commit} {prefix} />
    }
}
