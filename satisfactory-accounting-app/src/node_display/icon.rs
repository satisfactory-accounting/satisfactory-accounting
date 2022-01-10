// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use std::rc::Rc;

use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct Props {
    /// Reference to the icon slug.
    #[prop_or_default]
    pub icon: Option<Rc<str>>,
    #[prop_or_default]
    pub alt: Option<Rc<str>>,
}

#[function_component(Icon)]
pub fn icon(props: &Props) -> Html {
    match &props.icon {
        Some(icon) => html! {
            <img src={slug_to_icon(icon)} class="icon" alt={props.alt.clone()} />
        },
        None => html! {
            <span class="icon material-icons error">{"error"}</span>
        },
    }
}

/// Get the icon path for a given slug name.
fn slug_to_icon(slug: impl AsRef<str>) -> String {
    let mut icon = slug.as_ref().to_owned();
    icon.insert_str(0, "/images/items/");
    icon.push_str("_64.png");
    icon
}
