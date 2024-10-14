//! Management for user settings.
use serde::{Deserialize, Serialize};

use crate::node_display::BalanceSortMode;
pub use crate::user_settings::manager::{
    use_user_settings, use_user_settings_dispatcher, UserSettingsDispatcher, UserSettingsManager,
};
pub use crate::user_settings::window::{
    use_user_settings_window, UserSettingsWindowDispatcher, UserSettingsWindowManager,
};

mod manager;
mod window;

/// App-wide settings specific to the user rather than the world.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserSettings {
    /// Whether empty balance values should be hidden.
    pub hide_empty_balances: bool,
    /// How to sort the user's balances.
    pub balance_sort_mode: BalanceSortMode,

    /// Whether to show deprecated database versions.
    #[serde(default)]
    pub show_deprecated_databases: bool,
}
