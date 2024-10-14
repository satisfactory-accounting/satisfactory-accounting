// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use crate::app::App;

mod app;
mod appheader;
mod collections;
mod inputs;
mod material;
mod node_display;
mod overlay_window;
mod refeqrc;
mod user_settings;
mod world;

fn main() {
    console_log::init_with_level(log::Level::Debug).expect("Unable to init logger");
    let app_root = gloo::utils::document()
        .get_element_by_id("app-host")
        .expect("Missing the app-host element");
    yew::Renderer::<App>::with_root(app_root).render();
}
