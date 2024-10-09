//! Provides the user settings window.

use yew::{function_component, html, use_callback, Html};

use crate::node_display::BalanceSortMode;
use crate::overlay_window::controller::{ShowWindowDispatcher, WindowManager};
use crate::overlay_window::OverlayWindow;
use crate::user_settings::{
    use_user_settings, use_user_settings_dispatcher, use_user_settings_window,
};

pub type UserSettingsWindowManager = WindowManager<UserSettingsWindow>;
pub type UserSettingsWindowDispatcher = ShowWindowDispatcher<UserSettingsWindow>;

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
                                <input class="hidden-checkbox" type="checkbox"
                                    checked={user_settings.hide_empty_balances}
                                    onclick={toggle_hide_empty} />
                                <span class="input-display material-icons">
                                    if user_settings.hide_empty_balances {
                                        {"check_box"}
                                    } else {
                                        {"check_box_outline_blank"}
                                    }
                                </span>
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
                                <input class="hidden-radio" type="radio" name="balance-sort" value="item"
                                    checked={user_settings.balance_sort_mode == BalanceSortMode::Item}
                                    onclick={set_sort_mode_item} />
                                <span class="input-display material-icons">
                                    if user_settings.balance_sort_mode == BalanceSortMode::Item {
                                        {"radio_button_checked"}
                                    } else {
                                        {"radio_button_unchecked"}
                                    }
                                </span>
                            </label>
                        </li>
                        <li>
                            <label>
                                <span>{"Sort by inputs vs outputs, then by item"}</span>
                                <input class="hidden-radio" type="radio" name="balance-sort" value="io-item"
                                    checked={user_settings.balance_sort_mode == BalanceSortMode::IOItem}
                                    onclick={set_sort_mode_ioitem} />
                                <span class="input-display material-icons">
                                    if user_settings.balance_sort_mode == BalanceSortMode::IOItem {
                                        {"radio_button_checked"}
                                    } else {
                                        {"radio_button_unchecked"}
                                    }
                                </span>
                            </label>
                        </li>
                    </ul>
                </div>
            </div>
        </OverlayWindow>
    }
}
