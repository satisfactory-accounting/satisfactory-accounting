//! Provides the user settings window.

use yew::{function_component, hook, html, use_callback, use_context, Html};

use crate::inputs::button::Button;
use crate::inputs::toggle::{MaterialCheckbox, MaterialRadio};
use crate::node_display::{BackdriveSettingsSection, BalanceSortMode};
use crate::overlay_window::controller::{ShowWindowDispatcher, WindowManager};
use crate::overlay_window::OverlayWindow;
use crate::user_settings::number_format::NumberDisplaySettingsSection;
use crate::user_settings::{use_user_settings, use_user_settings_dispatcher};

pub type UserSettingsWindowManager = WindowManager<UserSettingsWindow>;
pub type UserSettingsWindowDispatcher = ShowWindowDispatcher<UserSettingsWindow>;

/// Gets access to the user settings window dispatcher which controls showing the user settings
/// window.
#[hook]
pub fn use_user_settings_window() -> UserSettingsWindowDispatcher {
    use_context::<UserSettingsWindowDispatcher>().expect(
        "use_user_settings_window can only be used from within a child of \
        UserSettingsWindowManager.",
    )
}

#[function_component]
pub fn UserSettingsWindow() -> Html {
    let window_dispatcher = use_user_settings_window();
    let close = use_callback(window_dispatcher, |(), window_dispatcher| {
        window_dispatcher.hide_window();
    });
    let user_settings = use_user_settings();
    let settings_dispatcher = use_user_settings_dispatcher();

    let toggle_hide_empty = use_callback(settings_dispatcher.clone(), |_, settings_dispatcher| {
        settings_dispatcher.toggle_hide_empty_balances();
    });

    let set_sort_mode_item = use_callback(settings_dispatcher.clone(), |_, settings_dispatcher| {
        settings_dispatcher.set_sort_mode(BalanceSortMode::Item);
    });

    let set_sort_mode_ioitem =
        use_callback(settings_dispatcher.clone(), |_, settings_dispatcher| {
            settings_dispatcher.set_sort_mode(BalanceSortMode::IOItem);
        });

    let persist = use_callback(settings_dispatcher, |(), settings_dispatcher| {
        settings_dispatcher.persist_local_storage();
    });

    html! {
        <OverlayWindow title="Settings" class="UserSettingsWindow" on_close={close}>
            <div class="settings-section">
                <h2>{"Balance Display"}</h2>
                <div class="settings-subsection">
                    <h3>{"Display of Neutral (0) Balances"}</h3>
                    <p>{"Whether balance entries with a value of 0 should be shown. Hiding neutral \
                    balances lets you filter out fully-consumed intermediate products form higher \
                    level groups, but can make it harder to tell when a group actually has \
                    something internally that just happens to be used up."}</p>
                    <ul>
                        <li>
                            <label>
                                <span>{"Hide Neutral Balances"}</span>
                                <MaterialCheckbox checked={user_settings.hide_empty_balances}
                                    onclick={toggle_hide_empty} />
                            </label>
                        </li>
                    </ul>
                </div>
                <div class="settings-subsection">
                    <h3>{"Balance Sort Order"}</h3>
                    <p>{"Whether balances should be sorted purely by the item or grouped into \
                    inputs and outputs, with the inputs and outputs then sorted by item"}</p>
                    <ul>
                        <li>
                            <label>
                                <span>{"Sort by item"}</span>
                                <MaterialRadio
                                    checked={user_settings.balance_sort_mode == BalanceSortMode::Item}
                                    onclick={set_sort_mode_item} />
                            </label>
                        </li>
                        <li>
                            <label>
                                <span>{"Sort by inputs vs outputs, then by item"}</span>
                                <MaterialRadio
                                    checked={user_settings.balance_sort_mode == BalanceSortMode::IOItem}
                                    onclick={set_sort_mode_ioitem} />
                            </label>
                        </li>
                    </ul>
                </div>
            </div>
            <BackdriveSettingsSection />
            <NumberDisplaySettingsSection />
            <div class="settings-section">
                <h2>{"Storage Persistence"}</h2>
                <p>{"Satisfactory Accounting stores your worlds and user settings in "}
                    <a target="_blank" href="https://developer.mozilla.org/en-US/docs/Web/API/Web_Storage_API">
                        {"Local Storage"}
                    </a>
                    {" which Browsers provide on a Best Effort basis by default. This means that \
                    the browser can choose to delete your stored data if storage is low. In \
                    practice this rarely happens, however you can change the storage mode to \
                    \"Persisted\" which prevents the browser from deleting your data."}</p>
                <p>{"Changing from \"Best Effort\" to \"Persisted\" can be done by choosing \
                    \"Persist\" below and granting permission in your browser. The permission can \
                    be revoked at any time from your browser's permission manager."}</p>
                <p>{"We don't display the current persistence status because for your privacy, \
                    many browsers do not accurately report the current persistence status to web \
                    applications."}</p>
                <div class="persistence-enable">
                    <Button title="Enable persistence (requires you to grant permission)" onclick={persist}>
                        {"Enable Persistence"}
                    </Button>
               </div>
            </div>
        </OverlayWindow>
    }
}
