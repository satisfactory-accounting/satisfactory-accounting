use satisfactory_accounting::accounting::{SplitCopies, MAX_CLOCK, MIN_CLOCK};
// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use yew::prelude::*;

use crate::inputs::clickedit::ClickEdit;
use crate::material::material_icon_outlined;

#[derive(Debug, PartialEq, Properties)]
pub struct Props {
    /// Last set value for the clock speed.
    pub clock_speed: f32,
    /// Number of virtual copies of the building.
    pub copies: f32,
    /// Callback to change the actual value.
    pub on_update_speed: Callback<f32>,
}

/// Display and editing for clock speed.
#[function_component]
pub fn ClockSpeed(props: &Props) -> Html {
    let on_commit = use_callback(
        props.on_update_speed.clone(),
        |edit_text: AttrValue, on_update_speed| {
            if let Ok(value) = edit_text.parse::<f32>() {
                on_update_speed.emit(value.clamp(MIN_CLOCK, MAX_CLOCK));
            }
        },
    );

    let split = SplitCopies::split(props.copies, props.clock_speed);

    let value: AttrValue = props.clock_speed.to_string().into();
    let prefix = material_icon_outlined("timer");
    let suffix = if split.last_clock > 0.0 {
        Some(html! {
            <span class="extra-multiplier">
                {"\u{00d7}"}{split.whole_copies}
                {" + "}
                {material_icon_outlined("timer")}
                {" "}{split.last_clock}{" \u{00d7}1"}
            </span>
        })
    } else {
        None
    };
    html! {
        <ClickEdit {value} class="ClockSpeed" title="Clock Speed" {on_commit} {prefix} {suffix} />
    }
}
