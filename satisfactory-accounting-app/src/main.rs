// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use std::rc::Rc;

use node_display::{NodeMeta, NodeMetadata};
use uuid::Uuid;
use yew::prelude::*;

use satisfactory_accounting::database::Database;

mod app;
mod node_display;

fn main() {
    console_log::init_with_level(log::Level::Debug).expect("Unable to init logger");
    yew::start_app::<app::App>();
}

/// Helper to grab the database from Context.
trait CtxHelper {
    /// Get the database from context, throw if context is missing.
    fn db(&self) -> Rc<Database>;

    /// Get the metadata from context, throw if context is missing (gets default metadat
    /// if not set).
    fn meta(&self, id: Uuid) -> NodeMeta;
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
}

/// Get the database from context.
fn db_ctx() -> Rc<Database> {
    use_context::<Rc<Database>>().expect("database context to be set")
}
