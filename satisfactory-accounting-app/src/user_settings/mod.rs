//! Management for user settings.

use serde::{Deserialize, Serialize};

use crate::node_display::BalanceSortMode;

/// App-wide settings specific to the user rather than the world.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserSettings {
    /// Whether empty balance values should be hidden.
    pub hide_empty_balances: bool,
    /// How to sort the user's balances.
    pub balance_sort_mode: BalanceSortMode,
}
