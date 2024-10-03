//! Management for user settings.
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use yew::{hook, use_context};

use crate::node_display::BalanceSortMode;
pub use crate::user_settings::manager::{UserSettingsDispatcher, UserSettingsManager};
pub use crate::user_settings::window::{UserSettingsWindowDispatcher, UserSettingsWindowManager};

mod manager;
mod window;

/// App-wide settings specific to the user rather than the world.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserSettings {
    /// Whether empty balance values should be hidden.
    pub hide_empty_balances: bool,
    /// How to sort the user's balances.
    pub balance_sort_mode: BalanceSortMode,
}

/// Get the current settings from the context and respond to changes to the settings.
#[hook]
pub fn use_user_settings() -> Rc<UserSettings> {
    use_context::<Rc<UserSettings>>()
        .expect("use_user_settings can only be used from within a child of UserSettingsManager.")
}

/// Get the UserSettingsDispatcher. Only triggers redraw if the UserSettingsManager is replaced
/// somehow which shouldn't happen.
#[hook]
pub fn use_user_settings_dispatcher() -> UserSettingsDispatcher {
    use_context::<UserSettingsDispatcher>().expect(
        "use_user_settings_dispatcher can only be used from within a child of UserSettingsManager.",
    )
}

/// Gets access to the user settings window dispatcher which controls showing the user settings
/// window.
#[hook]
pub fn use_user_settings_window() -> UserSettingsWindowDispatcher {
    use_context::<UserSettingsWindowDispatcher>().expect(
        "use_user_settings_window can only be used from within a child of \
        UserSettingsWindowManager.",
    )
}
