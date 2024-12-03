use yew::{function_component, html, use_callback, AttrValue, Callback, Html, Properties};

use crate::inputs::clickedit::{AdjustDir, AdjustScale, ClickEdit, ValueAdjustment};
use crate::inputs::toggle::MaterialRadio;
use crate::user_settings::number_format::{
    NumberFormatMode, NumberFormatSettings, NumberStylingMode,
};
use crate::user_settings::{use_user_settings, use_user_settings_dispatcher};

use super::NumberDisplaySettings;

/// Container for commands that are used to update NumberFormatSettings.
#[repr(transparent)]
pub struct NumberDisplaySettingsMsg {
    /// Inner message (non-pub).
    msg: Msg,
}

impl From<Msg> for NumberDisplaySettingsMsg {
    #[inline]
    fn from(msg: Msg) -> Self {
        Self { msg }
    }
}

/// Inner non-pub message.
enum Msg {
    UpdateBalanceHighlightMode { mode: NumberStylingMode },
    UpdateBalanceHideMode { mode: NumberStylingMode },
    UpdateBalanceFormat { settings: NumberFormatSettings },
    UpdateClockFormat { settings: NumberFormatSettings },
    UpdateMultiplierFormat { settings: NumberFormatSettings },
}

impl NumberDisplaySettings {
    /// Message handler for [Msg::UpdateBalanceHighlightMode].
    fn set_balance_highlight_mode(&mut self, mode: NumberStylingMode) -> bool {
        if self.balance.highlight_style.mode != mode {
            self.balance.highlight_style.mode = mode;
            true
        } else {
            false
        }
    }

    /// Message handler for [Msg::UpdateBalanceHideMode].
    fn set_balance_hide_mode(&mut self, mode: NumberStylingMode) -> bool {
        if self.balance.hide_style.mode != mode {
            self.balance.hide_style.mode = mode;
            true
        } else {
            false
        }
    }

    /// Message handler for [Msg::UpdateBalanceFormat].
    fn set_balance_format(&mut self, settings: NumberFormatSettings) -> bool {
        if self.balance.item_format_settings != settings
            || self.balance.power_format_settings != settings
        {
            self.balance.item_format_settings = settings.clone();
            self.balance.power_format_settings = settings;
            true
        } else {
            false
        }
    }

    /// Message handler for [Msg::UpdateClockFormat].
    fn set_clock_format(&mut self, settings: NumberFormatSettings) -> bool {
        if self.clock.format != settings {
            self.clock.format = settings;
            true
        } else {
            false
        }
    }

    /// Message handler for [Msg::UpdateMultiplierFormat].
    fn set_multiplier_format(&mut self, settings: NumberFormatSettings) -> bool {
        if self.multiplier.format != settings {
            self.multiplier.format = settings;
            true
        } else {
            false
        }
    }

    /// Update the number display settings, return true if settings changed.
    pub(in crate::user_settings) fn update(&mut self, msg: NumberDisplaySettingsMsg) -> bool {
        match msg.msg {
            Msg::UpdateBalanceHighlightMode { mode } => self.set_balance_highlight_mode(mode),
            Msg::UpdateBalanceHideMode { mode } => self.set_balance_hide_mode(mode),
            Msg::UpdateBalanceFormat { settings } => self.set_balance_format(settings),
            Msg::UpdateClockFormat { settings } => self.set_clock_format(settings),
            Msg::UpdateMultiplierFormat { settings } => self.set_multiplier_format(settings),
        }
    }
}

/// Component for the number format settings section of user settings.
#[function_component]
pub fn NumberDisplaySettingsSection() -> Html {
    let user_settings = use_user_settings();
    let num = &user_settings.number_display;
    let user_settings_dispatcher = use_user_settings_dispatcher();

    let change_balance_highlight_mode = use_callback(
        user_settings_dispatcher.clone(),
        |mode, user_settings_dispatcher| {
            user_settings_dispatcher
                .update_number_display_settings(Msg::UpdateBalanceHighlightMode { mode });
        },
    );

    let change_balance_hide_mode = use_callback(
        user_settings_dispatcher.clone(),
        |mode, user_settings_dispatcher| {
            user_settings_dispatcher
                .update_number_display_settings(Msg::UpdateBalanceHideMode { mode });
        },
    );

    let change_balance_format = use_callback(
        user_settings_dispatcher.clone(),
        |settings, user_settings_dispatcher| {
            user_settings_dispatcher
                .update_number_display_settings(Msg::UpdateBalanceFormat { settings });
        },
    );

    let change_clock_format = use_callback(
        user_settings_dispatcher.clone(),
        |settings, user_settings_dispatcher| {
            user_settings_dispatcher
                .update_number_display_settings(Msg::UpdateClockFormat { settings });
        },
    );

    let change_multiplier_format = use_callback(
        user_settings_dispatcher,
        |settings, user_settings_dispatcher| {
            user_settings_dispatcher
                .update_number_display_settings(Msg::UpdateMultiplierFormat { settings });
        },
    );

    html! {
        <div class="NumberFormatSettingsSection settings-section">
            <h2>{"Number Display Settings"}</h2>
            <p>{"This section controls how numbers are displayed and styled."}</p>
            <div class="settings-subsection">
                <h3>{"Balance Display"}</h3>
                <p>{"These settings control how balances are displayed."}</p>
                <h4>{"Rounding of Balances"}</h4>
                <p>{"These settings control how balances are rounded."}</p>
                <p>{"There are currently three display modes. \""}<b>{"Precise"}</b>{"\" displays \
                the value with as much precision as we have available. \""}<b>{"Rounded"}</b>{"\" \
                rounds the value to at most the specified number of decimal digits. \""}<b>
                {"Rounded with Padding"}</b>{"\" rounds to the same number of decimal digits but \
                fills with 0 to keep the number of digits the same."}</p>
                <p>{"Note that the precision of the rounding is limited by the underlying \
                precision of the numbers we use. At present you can expect no more than 6-7 digits,
                regardless of your rouding setting, since we use f32."}</p>
                <FormatSettings current={num.balance.item_format_settings.clone()}
                    on_change={change_balance_format} />
                <h4>{"Coloring Balances and Hiding Zero Balances"}</h4>
                <p>{"These settings control how coloring of balances and hiding of zero balances \
                are affected by the rounding settings."}</p>
                <p>{"You can independently control how coloring of balances and hiding of zero \
                balances are affected by your rounding settings. Both offer the same two modes. \
                \""}<b>{"By Displayed Value"}</b>{"\" means to color or hide the value based on \
                what is displayed, even if the displayed value is rounded. \""}<b>{"By Exact Value"}
                </b>{"\" means to color or hide based on the underlying value, even if that \
                doesn't match what is shown after rounding."}</p>
                <h5>{"Balance Coloring Mode"}</h5>
                <StyleMode current={num.balance.highlight_style.mode}
                    on_change={change_balance_highlight_mode} />
                <h5>{"Balance Hiding Mode"}</h5>
                <StyleMode current={num.balance.hide_style.mode}
                    on_change={change_balance_hide_mode} />
            </div>
            <div class="settings-subsection">
                <h3>{"Clock and Multiplier Display"}</h3>
                <p>{"Clock speed and Multiplier accept can have the same rounding settings as \
                balances (coloring settings don't apply because these values aren't colored). See \
                above for more about these modes."}</p>
                <h4>{"Clock Rounding"}</h4>
                <FormatSettings current={num.clock.format.clone()}
                    on_change={change_clock_format} />
                <h4>{"Multiplier Rounding"}</h4>
                <FormatSettings current={num.multiplier.format.clone()}
                    on_change={change_multiplier_format} />
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct StyleModeProps {
    /// Current styling mode.
    current: NumberStylingMode,
    /// Callback used when the styling mode changes.
    on_change: Callback<NumberStylingMode>,
}

/// Allows selecting between Balance style modes.
#[function_component]
fn StyleMode(props: &StyleModeProps) -> Html {
    let select_displayed = use_callback(props.on_change.clone(), |_, on_change| {
        on_change.emit(NumberStylingMode::DisplayedValue);
    });
    let select_exact = use_callback(props.on_change.clone(), |_, on_change| {
        on_change.emit(NumberStylingMode::ExactValue);
    });

    html! {
        <ul>
            <li>
                <label>
                    <span>{"By Displayed Value"}</span>
                    <MaterialRadio
                        checked={props.current == NumberStylingMode::DisplayedValue}
                        onclick={select_displayed} />
                </label>
            </li>
            <li>
                <label>
                    <span>{"By Exact Value"}</span>
                    <MaterialRadio
                        checked={props.current == NumberStylingMode::ExactValue}
                        onclick={select_exact} />
                </label>
            </li>
        </ul>
    }
}

#[derive(Properties, PartialEq)]
struct FormatSettingsProps {
    current: NumberFormatSettings,
    on_change: Callback<NumberFormatSettings>,
}

#[function_component]
fn FormatSettings(props: &FormatSettingsProps) -> Html {
    let select_precise = use_callback(
        (
            props.on_change.clone(),
            NumberFormatSettings {
                mode: NumberFormatMode::DecimalPrecise,
                ..props.current
            },
        ),
        |_, (on_change, new_value)| on_change.emit(new_value.clone()),
    );
    let select_rounded = use_callback(
        (
            props.on_change.clone(),
            NumberFormatSettings {
                mode: NumberFormatMode::DecimalRounded,
                ..props.current
            },
        ),
        |_, (on_change, new_value)| on_change.emit(new_value.clone()),
    );
    let select_rounded_padded = use_callback(
        (
            props.on_change.clone(),
            NumberFormatSettings {
                mode: NumberFormatMode::DecimalRoundedPadded,
                ..props.current
            },
        ),
        |_, (on_change, new_value)| on_change.emit(new_value.clone()),
    );

    let set_round_decimal_places = use_callback(
        (
            props.on_change.clone(),
            NumberFormatSettings {
                // This field will be overridden so clear it from this template value.
                round_decimal_places: 0,
                ..props.current
            },
        ),
        |edit_text: AttrValue, (on_change, template)| {
            if let Ok(value) = edit_text.parse::<u32>() {
                on_change.emit(NumberFormatSettings {
                    round_decimal_places: value,
                    ..*template
                });
            }
        },
    );

    fn adjust_num_digits(adjustment: ValueAdjustment, current: AttrValue) -> AttrValue {
        let current = match current.parse::<u32>() {
            Ok(current) => current,
            Err(_) => return current,
        };
        let dist = match adjustment.scale {
            AdjustScale::Fine => 1,
            AdjustScale::Coarse => 2,
        };
        match adjustment.dir {
            AdjustDir::Up => current.saturating_add(dist),
            AdjustDir::Down => current.saturating_sub(dist),
        }
        .to_string()
        .into()
    }

    html! {
        <ul>
            <li>
                <label>
                    <span>{"Precise"}</span>
                    <MaterialRadio
                        checked={props.current.mode == NumberFormatMode::DecimalPrecise}
                        onclick={select_precise} />
                </label>
            </li>
            <li>
                <label>
                    <span>{"Rounded"}</span>
                    <MaterialRadio
                        checked={props.current.mode == NumberFormatMode::DecimalRounded}
                        onclick={select_rounded} />
                </label>
            </li>
            <li>
                <label>
                    <span>{"Rounded with Padding"}</span>
                    <MaterialRadio
                        checked={props.current.mode == NumberFormatMode::DecimalRoundedPadded}
                        onclick={select_rounded_padded} />
                </label>
            </li>
            <li>
                <label>
                    <ClickEdit
                        class="num-digits-to-round-to"
                        value={props.current.round_decimal_places.to_string()}
                        on_commit={set_round_decimal_places}
                        prefix={html! {
                            <span class="prefix">{"Number of digits to round to"}</span>
                        }}
                        title="Number of digits to round to"
                        adjust={adjust_num_digits as fn(_,_) -> _} />
                </label>
            </li>
        </ul>
    }
}
