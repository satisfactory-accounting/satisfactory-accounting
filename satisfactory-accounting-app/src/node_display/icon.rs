// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use std::rc::Rc;

use satisfactory_accounting::database::DatabaseVersion;
use yew::prelude::*;

use crate::db_ctx;

#[derive(PartialEq, Properties)]
pub struct Props {
    /// Reference to the icon slug.
    #[prop_or_default]
    pub icon: Option<Rc<str>>,
}

#[function_component(Icon)]
pub fn icon(props: &Props) -> Html {
    let version = db_ctx().version;
    match &props.icon {
        Some(icon) => html! {
            <img src={slug_to_icon(icon, version)} class="icon" alt="?" />
        },
        None => html! {
            <span class="icon material-icons error">{"error"}</span>
        },
    }
}

/// Get the icon path for a given slug name.
fn slug_to_icon(slug: impl AsRef<str>, db_version: Option<DatabaseVersion>) -> String {
    let prefix = match db_version {
        Some(DatabaseVersion::U5(_)) => "u5/",
        Some(DatabaseVersion::U6(_)) => "u6/",
        None => "",
    };
    let slug = slug.as_ref();
    format!("/images/items/{prefix}{slug}_64.png")
}
