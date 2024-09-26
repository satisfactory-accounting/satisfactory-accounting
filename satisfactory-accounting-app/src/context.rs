use std::rc::Rc;

use uuid::Uuid;
use yew::prelude::*;

use satisfactory_accounting::database::Database;

use crate::app::UserSettings;
use crate::node_display::{NodeMeta, NodeMetadata};

/// Helper to grab the database from Context.
pub(crate) trait CtxHelper {
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
            .expect("expected database context to be set");
        db
    }

    fn meta(&self, id: Uuid) -> NodeMeta {
        let (meta, _) = self
            .link()
            .context::<NodeMetadata>(Callback::noop())
            .expect("expected metadata context to be set");
        meta.meta(id)
    }
}

/// Get the database from context.
#[hook]
pub(crate) fn use_db() -> Rc<Database> {
    use_context::<Rc<Database>>().expect("expected database context to be set")
}

/// Get the settings from context.
#[hook]
pub(crate) fn use_settings() -> Rc<UserSettings> {
    use_context::<Rc<UserSettings>>().expect("expected user settings context to be set")
}
