use std::rc::Rc;

use satisfactory_accounting::database::{Database, DatabaseVersion};
use serde::{Deserialize, Serialize};
use yew::html::ImplicitClone;

/// The choice of database for a particular world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DatabaseChoice {
    /// Use one of the standard databases.
    Standard(DatabaseVersion),
    /// This world uses a custom database.
    Custom(Rc<Database>),
}

impl DatabaseChoice {
    /// Get the database for this database choice.
    pub(super) fn get(&self) -> Rc<Database> {
        match *self {
            DatabaseChoice::Standard(version) => Rc::new(version.load_database()),
            DatabaseChoice::Custom(ref db) => Rc::clone(db),
        }
    }

    /// Return true if this is a standard database with the specified version.
    fn is_standard_version(&self, version: DatabaseVersion) -> bool {
        match *self {
            DatabaseChoice::Standard(v) => v == version,
            _ => false,
        }
    }
}

impl Default for DatabaseChoice {
    fn default() -> Self {
        DatabaseChoice::Standard(DatabaseVersion::LATEST)
    }
}

impl ImplicitClone for DatabaseChoice {}
