// Copyright 2022 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use yew::prelude::*;

use crate::inputs::clickedit::{
    AdjustDir, AdjustModifier, AdjustScale, ClickEdit, ValueAdjustment,
};
use crate::user_settings::number_format::UserConfiguredFormat;
use crate::user_settings::use_user_settings;

#[derive(Debug, PartialEq, Properties)]
pub struct Props {
    /// Last set value for the number of virtual copies.
    pub copies: f32,
    /// Callback to change the actual value.
    pub update_copies: Callback<f32>,
}

/// Display and editing for number of coipes.
#[function_component]
pub fn VirtualCopies(props: &Props) -> Html {
    let on_commit = use_callback(
        props.update_copies.clone(),
        |edit_text: AttrValue, update_copies| {
            if let Ok(value) = edit_text.parse::<f32>() {
                update_copies.emit(value);
            }
        },
    );

    let user_settings = use_user_settings();
    let rounding = &user_settings.number_display.multiplier.format;

    let value: AttrValue = props.copies.to_string().into();
    let rounded_value: AttrValue = props.copies.format(rounding).to_string().into();
    let suffix = html! {
        <span>{"\u{00d7}"}</span>
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
            // Fine adjustment by increments of 1 building.
            (AdjustScale::Fine, AdjustModifier::None) => 1.0,
            // Coarse adjustment by increments of 5 buildings.
            (AdjustScale::Coarse, AdjustModifier::None) => 5.0,
            // Small scale adjustments are by 1% and 10% of clock respectively.
            (AdjustScale::Fine, AdjustModifier::Smaller) => 0.01,
            (AdjustScale::Coarse, AdjustModifier::Smaller) => 0.1,
        };
        (current + dir * dist).to_string().into()
    }

    html! {
        <ClickEdit {value} {rounded_value} class="VirtualCopies" title="Multiplier" {on_commit}
            {suffix} adjust={adjust as fn(_,_)->_} />
    }
}
