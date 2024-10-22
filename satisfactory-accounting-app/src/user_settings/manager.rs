//! User settings manager provides types for owning, loading, and updating the user settings.

use std::rc::Rc;

use gloo::storage::errors::StorageError;
use gloo::storage::{LocalStorage, Storage as _};
use log::warn;
use yew::html::Scope;
use yew::{hook, html, use_context, Component, Context, ContextProvider, Html, Properties};

use crate::node_display::BalanceSortMode;
use crate::refeqrc::RefEqRc;
use crate::user_settings::storagemanager::persist_local_storage;
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
    /// Toggles the show deprecated databases setting.
    ToggleShowDeprecated,
    /// Acknowledges the use of LocalStorage.
    AckLocalStorage { version: u32 },
    /// Acknowledges a particular welcome message version.
    AckNotification { version: u32 },
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

impl UserSettingsManager {
    /// Message handler for MaybeInitFromWorld.
    fn maybe_init_from_world(
        &mut self,
        hide_empty_balances_from_deprecated_global_metadata: bool,
    ) -> bool {
        if self.fallback_to_world_global_metadata {
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
                // just have to update the fallback_to_world_global_metadata flag.
                false
            }
        } else {
            // If fallback_to_world_global_metadata is false, ignore MaybeInitFromWorld and don't
            // redraw.
            false
        }
    }

    /// Message handler for ToggleHideEmptyBalances
    fn toggle_hide_empty_balances(&mut self) -> bool {
        self.fallback_to_world_global_metadata = false;
        let user_settings = Rc::make_mut(&mut self.user_settings);
        user_settings.hide_empty_balances = !user_settings.hide_empty_balances;
        save_user_settings(user_settings);
        true
    }

    /// Message handler for SetBalanceSortMode.
    fn set_balance_sort_mode(&mut self, sort_mode: BalanceSortMode) -> bool {
        if self.user_settings.balance_sort_mode != sort_mode {
            Rc::make_mut(&mut self.user_settings).balance_sort_mode = sort_mode;
            save_user_settings(&self.user_settings);
            true
        } else {
            // If the current balance sort mode already matches, do nothing and don't redraw.
            false
        }
    }

    /// Message handler for ToggleShowDeprecated.
    fn toggle_show_deprecated(&mut self) -> bool {
        let user_settings = Rc::make_mut(&mut self.user_settings);
        user_settings.show_deprecated_databases = !user_settings.show_deprecated_databases;
        save_user_settings(user_settings);
        true
    }

    /// Message handler for AckLocalStorage.
    fn ack_local_storage(&mut self, version: u32) -> bool {
        // Don't allow backsliding.
        if self.user_settings.acked_local_storage_notice_version < version {
            Rc::make_mut(&mut self.user_settings).acked_local_storage_notice_version = version;
            save_user_settings(&self.user_settings);
            true
        } else {
            false
        }
    }

    /// Message handler for AckWelcomeMessage.
    fn ack_notification(&mut self, version: u32) -> bool {
        // Don't allow backsliding.
        if self.user_settings.acked_notification < version {
            Rc::make_mut(&mut self.user_settings).acked_notification = version;
            save_user_settings(&self.user_settings);
            true
        } else {
            false
        }
    }
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
                // Don't save settings during create: this way we don't store any data on the user's
                // computer until they interact with the app.
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
            } => self.maybe_init_from_world(hide_empty_balances_from_deprecated_global_metadata),
            Msg::ToggleHideEmptyBalances => self.toggle_hide_empty_balances(),
            Msg::SetBalanceSortMode { sort_mode } => self.set_balance_sort_mode(sort_mode),
            Msg::ToggleShowDeprecated => self.toggle_show_deprecated(),
            Msg::AckLocalStorage { version } => self.ack_local_storage(version),
            Msg::AckNotification { version } => self.ack_notification(version),
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
    scope: RefEqRc<Scope<UserSettingsManager>>,
}

impl UserSettingsDispatcher {
    /// Wraps the Scope from UserSettingsManager.
    fn new(scope: Scope<UserSettingsManager>) -> Self {
        Self {
            scope: RefEqRc::new(scope),
        }
    }

    /// Sets the value of `hide_empty_balances` only if `UserSettings` failed to load and the user
    /// has not yet set a new value.
    pub fn maybe_init_from_world(&self, hide_empty_balances_from_deprecated_global_metadata: bool) {
        self.scope.send_message(Msg::MaybeInitFromWorld {
            hide_empty_balances_from_deprecated_global_metadata,
        });
    }

    /// Toggles whether `hide_empty_balances` is set.
    pub fn toggle_hide_empty_balances(&self) {
        self.scope.send_message(Msg::ToggleHideEmptyBalances);
    }

    /// Sets the balance sort mode.
    pub fn set_sort_mode(&self, sort_mode: BalanceSortMode) {
        self.scope
            .send_message(Msg::SetBalanceSortMode { sort_mode });
    }

    /// Toggles whether deprecated databases are shown in the database chooser window.
    pub fn toggle_show_deprecated(&self) {
        self.scope.send_message(Msg::ToggleShowDeprecated);
    }

    /// Ack the given local storage notice version.
    pub fn ack_local_storage(&self, version: u32) {
        self.scope.send_message(Msg::AckLocalStorage { version });
    }

    /// Ack the given welcome message version.
    pub fn ack_notification(&self, version: u32) {
        self.scope.send_message(Msg::AckNotification { version });
    }

    /// Attempts to make local storage persisted.
    pub fn persist_local_storage(&self) {
        wasm_bindgen_futures::spawn_local(async {
            if let Err(e) = persist_local_storage().await {
                warn!("Unable to set local storage mode: {e}");
            }
        });
    }
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
