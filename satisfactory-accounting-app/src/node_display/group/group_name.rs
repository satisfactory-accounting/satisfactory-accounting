// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use yew::prelude::*;

use crate::inputs::clickedit::ClickEdit;

#[derive(PartialEq, Properties)]
pub struct Props {
    /// Current name of the Node.
    pub name: AttrValue,
    /// Callback to rename the node.
    pub rename: Callback<AttrValue>,
}

/// Display and editing for number of coipes.
#[function_component]
pub fn GroupName(props: &Props) -> Html {
    let (value, class) = if props.name.is_empty() {
        ("unnamed".into(), classes!("GroupName", "unnamed"))
    } else {
        (props.name.clone(), classes!("GroupName"))
    };
    html! {
        <ClickEdit {value} {class} title="Group Name" on_commit={props.rename.clone()} />
    }
}
