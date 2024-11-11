use satisfactory_accounting::database::{Database, DatabaseVersion};
use serde::{Deserialize, Serialize};
use yew::html::ImplicitClone;

/// Type for selecting a database version. This allows both pinned versions and special versions
/// like "Latest".
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DatabaseVersionSelector {
    /// Pin at the specified database version.
    Pinned(DatabaseVersion),
    /// Follow the latest database.
    // Note: placed last because it should compare greater than any DatabaseVersion.
    Latest,
}

impl DatabaseVersionSelector {
    /// Whether the version this selector chooses is deprecated.
    #[inline]
    pub fn is_deprecated(self) -> bool {
        self.select_version().is_deprecated()
    }

    /// Load the database at this version.
    #[inline]
    pub fn load_database(self) -> Database {
        self.select_version().load_database()
    }

    /// Get the actual database version which this selector chooses in the current version of
    /// Satisfactory Accounting.
    pub fn select_version(self) -> DatabaseVersion {
        match self {
            Self::Latest => DatabaseVersion::LATEST,
            Self::Pinned(version) => version,
        }
    }

    /// Get the name of this selector.
    pub fn name(self) -> &'static str {
        match self {
            Self::Latest => "Latest",
            Self::Pinned(version) => version.name(),
        }
    }

    /// Get a description of this version.
    pub fn description(self) -> &'static str {
        match self {
            Self::Latest => {
                "Choosing this version will always automatically follow the latest database \
                version when it is updated."
            }
            Self::Pinned(version) => version.description(),
        }
    }
}

/// The choice of database for a particular world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DatabaseChoice {
    /// Always use the latest database version.
    Latest,
    /// Use one of the standard databases.
    Standard(DatabaseVersion),
    /// This world uses a custom database.
    Custom(Database),
}

impl DatabaseChoice {
    /// Get the database for this database choice.
    pub(super) fn get(&self) -> Database {
        match *self {
            DatabaseChoice::Latest => Database::load_latest(),
            DatabaseChoice::Standard(version) => version.load_database(),
            DatabaseChoice::Custom(ref db) => db.clone(),
        }
    }

    /// Get the corresponding version selector if this is a standard database.
    pub fn version_selector(&self) -> Option<DatabaseVersionSelector> {
        match *self {
            DatabaseChoice::Latest => Some(DatabaseVersionSelector::Latest),
            DatabaseChoice::Standard(version) => Some(DatabaseVersionSelector::Pinned(version)),
            DatabaseChoice::Custom(_) => None,
        }
    }
}

impl Default for DatabaseChoice {
    fn default() -> Self {
        DatabaseChoice::Latest
    }
}

impl ImplicitClone for DatabaseChoice {}

impl From<DatabaseVersion> for DatabaseChoice {
    fn from(value: DatabaseVersion) -> Self {
        Self::Standard(value)
    }
}

impl From<DatabaseVersionSelector> for DatabaseChoice {
    fn from(value: DatabaseVersionSelector) -> Self {
        match value {
            DatabaseVersionSelector::Latest => DatabaseChoice::Latest,
            DatabaseVersionSelector::Pinned(version) => DatabaseChoice::Standard(version),
        }
    }
}
