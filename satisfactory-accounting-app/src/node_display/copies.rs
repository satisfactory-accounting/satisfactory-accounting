// Copyright 2022 Zachary Stewart
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
    /// Last set value for the number of virtual copies.
    pub copies: u32,
    /// Callback to change the actual value.
    pub update_copies: Callback<u32>,
}

/// Display and editing for number of coipes.
#[function_component]
pub fn VirtualCopies(props: &Props) -> Html {
    let on_commit = use_callback(
        props.update_copies.clone(),
        |edit_text: AttrValue, update_copies| {
            if let Ok(value) = edit_text.parse::<u32>() {
                update_copies.emit(value);
            }
        },
    );

    let value: AttrValue = props.copies.to_string().into();
    let suffix = html! {
        <span>{"\u{00d7}"}</span>
    };
    html! {
        <ClickEdit {value} class="VirtualCopies" title="Multiplier" {on_commit} {suffix} />
    }
}
