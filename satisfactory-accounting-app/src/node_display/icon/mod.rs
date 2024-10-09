// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use yew::prelude::*;

use crate::world::use_db;

#[derive(PartialEq, Properties)]
pub struct Props {
    /// Reference to the icon slug.
    #[prop_or_default]
    pub icon: Option<AttrValue>,
}

#[function_component(Icon)]
pub fn icon(props: &Props) -> Html {
    let db = use_db();

    match &props.icon {
        Some(icon) => html! {
            <img src={slug_to_icon(icon, db.icon_prefix())} class="Icon" alt="?" />
        },
        None => html! {
            <span class="Icon material-icons error">{"error"}</span>
        },
    }
}

/// Get the icon path for a given slug name.
fn slug_to_icon(slug: impl AsRef<str>, icon_prefix: &str) -> String {
    let slug = slug.as_ref();
    format!("/images/{icon_prefix}items/{slug}_64.png")
}
