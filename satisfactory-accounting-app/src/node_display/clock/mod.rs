// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use satisfactory_accounting::accounting::{SplitCopies, MAX_CLOCK, MIN_CLOCK};
use yew::prelude::*;

use crate::inputs::clickedit::{
    AdjustDir, AdjustModifier, AdjustScale, ClickEdit, ValueAdjustment,
};
use crate::material::material_icon_outlined;
use crate::user_settings::number_format::UserConfiguredFormat;
use crate::user_settings::use_user_settings;

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

    let rounding = &use_user_settings().number_display.clock.format;

    let split = SplitCopies::split(props.copies, props.clock_speed);

    let value: AttrValue = props.clock_speed.to_string().into();
    let rounded_value: AttrValue = props.clock_speed.format(rounding).to_string().into();
    let prefix = material_icon_outlined("timer");
    let suffix = if split.last_clock > 0.0 {
        Some(html! {<>
            <span class="extra-multiplier whole">
                {"\u{00d7} "}{split.whole_copies}
            </span>
            <span class="extra-multiplier plus">
                {" + "}
            </span>
            {material_icon_outlined("timer")}
            <span class="extra-multiplier fractional">
                {" "}{split.last_clock.format(rounding)}{" \u{00d7} 1"}
            </span>
        </>})
    } else {
        None
    };

    fn adjust(adjustment: ValueAdjustment, current: AttrValue) -> AttrValue {
        let current = match current.parse::<f32>() {
            Ok(current) => current,
            Err(_) => return current,
        };
        let dir = match adjustment.dir {
            AdjustDir::Up => 1.0,
            AdjustDir::Down => -1.0,
        };
        let dist = match (adjustment.scale, adjustment.modifier) {
            // Fine adjustment by increments of 1 power slug.
            (AdjustScale::Fine, AdjustModifier::None) => 0.5,
            // Coarse adjustment by increments of 100%.
            (AdjustScale::Coarse, AdjustModifier::None) => 1.0,
            // Small scale adjustments are by 1% and 10% respectively.
            (AdjustScale::Fine, AdjustModifier::Smaller) => 0.01,
            (AdjustScale::Coarse, AdjustModifier::Smaller) => 0.1,
        };
        (current + dir * dist).to_string().into()
    }

    html! {
        <ClickEdit {value} {rounded_value} class="ClockSpeed" title="Clock Speed" {on_commit}
            {prefix} {suffix}
            adjust={adjust as fn(_,_) -> _} />
    }
}
