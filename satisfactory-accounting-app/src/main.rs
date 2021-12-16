use std::rc::Rc;

use yew::prelude::*;

use satisfactory_accounting::database::Database;

mod app;
mod node_display;

fn main() {
    console_log::init_with_level(log::Level::Debug).expect("Unable to init logger");
    yew::start_app::<app::App>();
}

/// Helper to grab the database from Context.
trait GetDb {
    /// Get the database from context, throw if not set.
    fn db(&self) -> Rc<Database>;
}

impl<T: Component> GetDb for Context<T> {
    fn db(&self) -> Rc<Database> {
        let (db, _) = self
            .link()
            .context::<Rc<Database>>(Callback::noop())
            .expect("database context to be set");
        db
    }
}
