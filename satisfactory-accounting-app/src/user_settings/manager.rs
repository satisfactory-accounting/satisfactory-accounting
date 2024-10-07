//! User settings manager provides types for owning, loading, and updating the user settings.

use std::rc::Rc;

use gloo::storage::errors::StorageError;
use gloo::storage::{LocalStorage, Storage as _};
use log::warn;
use yew::html::Scope;
use yew::{html, Component, Context, ContextProvider, Html, Properties};

use crate::node_display::BalanceSortMode;
use crate::refeqrc::RefEqRc;
use crate::user_settings::UserSettings;

/// Local storage key used to save user settings.
const USER_SETTINGS_KEY: &str = "zstewart.satisfactorydb.usersettings";

fn load_user_settings() -> Result<UserSettings, StorageError> {
    LocalStorage::get(USER_SETTINGS_KEY)
}

/// Save the given user settings.
fn save_user_settings(settings: &UserSettings) {
    if let Err(e) = LocalStorage::set(USER_SETTINGS_KEY, settings) {
        warn!("Unable to save user settings: {}", e);
    }
}

#[derive(PartialEq, Properties)]
pub struct Props {
    /// Children to render within the context of the UserSettingsManager.
    pub children: Html,
}

pub enum Msg {
    /// Sets `hide_empty_balances` only if `UserSettings` failed to load from storage and hasn't
    /// since been set by the user.
    MaybeInitFromWorld {
        /// The value of `hide_empty_balances` in the deprecated global metadata.
        hide_empty_balances_from_deprecated_global_metadata: bool,
    },
    /// Flips the state of `hide_empty_balances`.
    ToggleHideEmptyBalances,
    /// Updates the balance_sort_mode.
    SetBalanceSortMode {
        /// The new sort mode to use.
        sort_mode: BalanceSortMode,
    },
}

pub struct UserSettingsManager {
    /// Current global settings.
    user_settings: Rc<UserSettings>,
    /// Whether to set the hide_empty_balances setting to the value from GlobalMetadata when the
    /// World is loaded. This is used to implement backwards compatibility with versions from before
    /// v1.2.0, where hide_empty_balances was stored on the World rather than the user settings.
    ///
    /// It is set to false any time hide_empty_balances is explictly set by the user or if
    /// load_user_settings succeeded.
    fallback_to_world_global_metadata: bool,

    /// Settings dispatcher for this instance.
    dispatcher: UserSettingsDispatcher,
}

impl Component for UserSettingsManager {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let mut fallback_to_world_global_metadata = true;
        let user_settings = Rc::new(match load_user_settings() {
            Ok(settings) => {
                fallback_to_world_global_metadata = false;
                settings
            }
            Err(e) => {
                if !matches!(e, StorageError::KeyNotFound(_)) {
                    warn!("Failed to load user settings: {}", e);
                }
                let settings = UserSettings::default();
                // Save the settings immediately. This prevents fallback_to_world_global_metadata
                // from applying to future runs.
                save_user_settings(&settings);
                settings
            }
        });
        let dispatcher = UserSettingsDispatcher::new(ctx.link().clone());
        Self {
            user_settings,
            fallback_to_world_global_metadata,
            dispatcher,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::MaybeInitFromWorld {
                hide_empty_balances_from_deprecated_global_metadata,
            } if self.fallback_to_world_global_metadata => {
                self.fallback_to_world_global_metadata = false;
                if self.user_settings.hide_empty_balances
                    != hide_empty_balances_from_deprecated_global_metadata
                {
                    Rc::make_mut(&mut self.user_settings).hide_empty_balances =
                        hide_empty_balances_from_deprecated_global_metadata;
                    save_user_settings(&self.user_settings);
                    true
                } else {
                    // If the existing global metadata state matches, there's no need to redraw, we
                    // just have to update hte fallback_to_world_global_metadata flag.
                    false
                }
            }
            // If fallback_to_world_global_metadata is false, ignore MaybeInitFromWorld and don't
            // redraw.
            Msg::MaybeInitFromWorld { .. } => false,
            Msg::ToggleHideEmptyBalances => {
                self.fallback_to_world_global_metadata = false;
                let user_settings = Rc::make_mut(&mut self.user_settings);
                user_settings.hide_empty_balances = !user_settings.hide_empty_balances;
                save_user_settings(user_settings);
                true
            }
            Msg::SetBalanceSortMode { sort_mode }
                if self.user_settings.balance_sort_mode != sort_mode =>
            {
                Rc::make_mut(&mut self.user_settings).balance_sort_mode = sort_mode;
                save_user_settings(&self.user_settings);
                true
            }
            // If the current balance sort mode already matches, do nothing and don't redraw.
            Msg::SetBalanceSortMode { .. } => false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            // This context provider will never change for the life of the UserSettingsManager.
            <ContextProvider<UserSettingsDispatcher> context={self.dispatcher.clone()}>
            // This context will change whenever the user settings change.
            <ContextProvider<Rc<UserSettings>> context={Rc::clone(&self.user_settings)}>
                {ctx.props().children.clone()}
            </ContextProvider<Rc<UserSettings>>>
            </ContextProvider<UserSettingsDispatcher>>
        }
    }
}

/// Dispatcher which can be used to update user settings.
#[derive(Clone, Debug, PartialEq)]
pub struct UserSettingsDispatcher {
    inner: RefEqRc<InnerDispatcher>,
}

impl UserSettingsDispatcher {
    /// Wraps the Scope from UserSettingsManager.
    fn new(scope: Scope<UserSettingsManager>) -> Self {
        Self {
            inner: RefEqRc::new(InnerDispatcher { scope }),
        }
    }

    /// Sets the value of `hide_empty_balances` only if `UserSettings` failed to load and the user
    /// has not yet set a new value.
    pub fn maybe_init_from_world(&self, hide_empty_balances_from_deprecated_global_metadata: bool) {
        self.inner.scope.send_message(Msg::MaybeInitFromWorld {
            hide_empty_balances_from_deprecated_global_metadata,
        });
    }

    /// Toggles whether `hide_empty_balances` is set.
    pub fn toggle_hide_empty_balances(&self) {
        self.inner.scope.send_message(Msg::ToggleHideEmptyBalances);
    }

    /// Sets the balance sort mode.
    pub fn set_sort_mode(&self, sort_mode: BalanceSortMode) {
        self.inner
            .scope
            .send_message(Msg::SetBalanceSortMode { sort_mode });
    }
}

#[derive(Debug)]
struct InnerDispatcher {
    scope: Scope<UserSettingsManager>,
}
