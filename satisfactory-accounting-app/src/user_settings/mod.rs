//! Management for user settings.
use serde::{Deserialize, Serialize};

use crate::node_display::BalanceSortMode;
pub use crate::user_settings::manager::{
    use_user_settings, use_user_settings_dispatcher, UserSettingsDispatcher, UserSettingsManager,
};
#[allow(unused_imports)]
pub use crate::user_settings::window::{
    use_user_settings_window, UserSettingsWindowDispatcher, UserSettingsWindowManager,
};

mod manager;
mod storagemanager;
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

    /// Whether the user has acknowledged the use of local storage.
    #[serde(default)]
    pub acked_local_storage_notice_version: u32,

    /// Which welcome notice version the user has acked. Note that notice 1 is reserved for the
    /// new-user welcome message while later messages are used for updates to retuning users.
    /// Therefore the serde default is different from the Default::default, because for new users we
    /// want to get 0 so they receive the welcome notice, while for returning users we want to start
    /// with 1 if they've used a prior version of satisfactory accounting so they don't get the new
    /// user message.
    #[serde(default = "welcome_message_serde_default")]
    pub acked_welcome_message: u32,
}

/// Serde default for acked_welcome_notice.
#[inline]
const fn welcome_message_serde_default() -> u32 {
    1
}
