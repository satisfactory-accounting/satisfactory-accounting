// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use std::rc::Rc;

use uuid::Uuid;
use yew::prelude::*;

use satisfactory_accounting::database::Database;

use crate::app::App;

use self::app::UserSettings;
use self::node_display::{NodeMeta, NodeMetadata};

mod app;
mod node_display;
mod clickedit;
mod events;

fn main() {
    console_log::init_with_level(log::Level::Debug).expect("Unable to init logger");
    yew::Renderer::<App>::new().render();
}

/// Helper to grab the database from Context.
trait CtxHelper {
    /// Get the database from context, throw if context is missing.
    fn db(&self) -> Rc<Database>;

    /// Get the metadata from context, throw if context is missing (gets default metadat
    /// if not set).
    fn meta(&self, id: Uuid) -> NodeMeta;

    /// Get the user settings from context, throw if context is missing (gets default if
    /// not set).
    fn settings(&self) -> Rc<UserSettings>;
}

impl<T: Component> CtxHelper for Context<T> {
    fn db(&self) -> Rc<Database> {
        let (db, _) = self
            .link()
            .context::<Rc<Database>>(Callback::noop())
            .expect("database context to be set");
        db
    }

    fn meta(&self, id: Uuid) -> NodeMeta {
        let (meta, _) = self
            .link()
            .context::<NodeMetadata>(Callback::noop())
            .expect("metadata context to be set");
        meta.meta(id)
    }

    fn settings(&self) -> Rc<UserSettings> {
        let (settings, _) = self
            .link()
            .context::<Rc<UserSettings>>(Callback::noop())
            .expect("user settings context to be set");
        settings
    }
}

/// Get the database from context.
#[hook]
fn use_db() -> Rc<Database> {
    use_context::<Rc<Database>>().expect("database context to be set")
}
