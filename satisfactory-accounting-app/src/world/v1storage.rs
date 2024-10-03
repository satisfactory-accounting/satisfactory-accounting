//! Provides backwards compatibility with the V1 version of Satisfactory Accounting, when separate
//! storage keys were used for database choice and world.

// This code explicitly handles legacy stuff, so allow deprecated.
#![allow(deprecated)]

use std::rc::Rc;

use gloo::storage::errors::StorageError;
use gloo::storage::{LocalStorage, Storage as _};
use log::warn;
use satisfactory_accounting::accounting::{Group, Node};
use satisfactory_accounting::database::{Database, DatabaseVersion};

use crate::world::{DatabaseChoice, GlobalMetadata, NodeMetadata, World};

/// Key wehere the v1 database was stored.
const DB_KEY: &str = "zstewart.satisfactorydb.state.database";
/// Key where the v1 root node was stored.
const GRAPH_KEY: &str = "zstewart.satisfactorydb.state.graph";
/// Key where the v1 per-node metadata was stored.
const METADATA_KEY: &str = "zstewart.satisfactorydb.state.metadata";
/// Key where the v1 world/global metadata was stored.
const GLOBAL_METADATA_KEY: &str = "zstewart.satisfactorydb.state.globalmetadata";

/// Try to load a V1 world, replacing any missing components with defaults.
pub fn try_load_v1() -> World {
    let database = load_v1_db_or_fallback();
    let root = load_v1_root_node_or_empty();
    let mut metadata = load_v1_node_metadata_or_empty();
    metadata.prune(&root);
    let global_metadata = load_v1_global_metadata_or_default();

    World {
        database,
        root,
        node_metadata: metadata,
        global_metadata,
    }
}

/// Try to load a v1 database, or fall back to defaults.
fn load_v1_db_or_fallback() -> DatabaseChoice {
    match LocalStorage::get::<Database>(DB_KEY) {
        Ok(mut database) => {
            // All databases in the DB_KEY should be pre-U6 which means they shouldn't
            // have an icon prefix, and we can set the icon prefix to u5, unless for
            // some reason it's already set.
            if database.icon_prefix.is_empty() {
                database.icon_prefix = "u5/".to_string();
            }
            DatabaseVersion::ALL
                .iter()
                .find_map(|&version| match version.load_database() {
                    db if database.compare_ignore_prefix(&db) => {
                        Some(DatabaseChoice::Standard(version))
                    }
                    _ => None,
                })
                .unwrap_or_else(move || DatabaseChoice::Custom(Rc::new(database)))
        }
        Err(e) => {
            if !matches!(e, StorageError::KeyNotFound(_)) {
                warn!("Failed to load database: {}", e);
            }
            DatabaseChoice::default()
        }
    }
}

/// Try to load a v1 graph's root node.
fn load_v1_root_node_or_empty() -> Node {
    LocalStorage::get(GRAPH_KEY).unwrap_or_else(|e| {
        if !matches!(e, StorageError::KeyNotFound(_)) {
            warn!("Failed to load graph: {}", e);
        }
        Group::empty_node()
    })
}

/// Try to load a v1 world's per-node metadata.
fn load_v1_node_metadata_or_empty() -> NodeMetadata {
    LocalStorage::get(METADATA_KEY).unwrap_or_else(|e| {
        if !matches!(e, StorageError::KeyNotFound(_)) {
            warn!("Failed to load metadata: {}", e);
        }
        Default::default()
    })
}

/// Try to load a v1 world's global metadata.
fn load_v1_global_metadata_or_default() -> GlobalMetadata {
    LocalStorage::get(GLOBAL_METADATA_KEY).unwrap_or_else(|e| {
        if !matches!(e, StorageError::KeyNotFound(_)) {
            warn!("Failed to load global metadata: {}", e);
        }
        Default::default()
    })
}
