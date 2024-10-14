//! Provides the user settings window.

use yew::{function_component, hook, html, use_callback, use_context, Html};

use crate::inputs::toggle::{MaterialCheckbox, MaterialRadio};
use crate::node_display::BalanceSortMode;
use crate::overlay_window::controller::{ShowWindowDispatcher, WindowManager};
use crate::overlay_window::OverlayWindow;
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
fn UserSettingsWindow() -> Html {
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

    html! {
        <OverlayWindow title="Settings" class="UserSettingsWindow" on_close={close}>
            <div class="balances">
                <h2>{"Balance Display"}</h2>
                <div class="empty-balances">
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
                <div class="balance-sort-mode">
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
        </OverlayWindow>
    }
}
