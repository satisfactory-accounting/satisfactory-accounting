use satisfactory_accounting::database::{Database, DatabaseVersion};
use serde::{Deserialize, Serialize};
use yew::html::ImplicitClone;

/// The choice of database for a particular world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DatabaseChoice {
    /// Use one of the standard databases.
    Standard(DatabaseVersion),
    /// This world uses a custom database.
    Custom(Database),
}

impl DatabaseChoice {
    /// Get the database for this database choice.
    pub(super) fn get(&self) -> Database {
        match *self {
            DatabaseChoice::Standard(version) => version.load_database(),
            DatabaseChoice::Custom(ref db) => db.clone(),
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

impl From<DatabaseVersion> for DatabaseChoice {
    fn from(value: DatabaseVersion) -> Self {
        Self::Standard(value)
    }
}
