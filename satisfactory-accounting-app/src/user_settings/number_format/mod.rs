use serde::{Deserialize, Serialize};

/// Determines how roudning affect the style of displayed balances.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumberStylingMode {
    /// Style based on the rounded value.
    #[default]
    Rounded,
    /// Style based on the exact value, even if that doesn't match the rounded value.
    Exact,
}

/// How to display values in the balance, clock speed, etc.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumberFormatMode {
    /// Display a rounded value.
    Rounded,
    /// Display the exact value.
    Exact,
}

/// Number display settings for a particular kind of number (clock speed, balances, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumberFormatSettings {
    /// How to format the numbers.
    mode: NumberFormatMode,
    /// Number of decimal places to keep when rounding.
    round_decimal_places: u32,
}

/// Settings to apply to balance display.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceDisplaySettings {
    /// Whether the red/green/black coloring is based on the exact value or rounded value.
    highlight_style: NumberStylingMode,
    /// Whether hide-empty-balances should be based on the exact value or rounded value.
    hide_style: NumberStylingMode,
    /// Format settings to use for power.
    ///
    /// This is broken out in anticipation of fraction mode, where power will still be floating
    /// point. For now it is locked to follow item_format_settings.
    power_format_settings: NumberFormatSettings,
    /// Format settings to use for items.
    item_format_settings: NumberFormatSettings,
}

impl Default for BalanceDisplaySettings {
    fn default() -> Self {
        let format = NumberFormatSettings {
            mode: NumberFormatMode::Rounded,
            round_decimal_places: 2,
        };
        Self {
            highlight_style: Default::default(),
            hide_style: Default::default(),
            power_format_settings: format.clone(),
            item_format_settings: format,
        }
    }
}

/// Settings to apply to clock display.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClockDisplaySettings {
    /// Number format settings to apply to the clock.
    format: NumberFormatSettings,
}

impl Default for ClockDisplaySettings {
    fn default() -> Self {
        Self {
            format: NumberFormatSettings {
                mode: NumberFormatMode::Exact,
                round_decimal_places: 6,
            },
        }
    }
}

/// Settings related to how various numbers are displayed.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumberDisplaySettings {
    /// How to display balances.
    balance: BalanceDisplaySettings,
    /// How to display the clock speed.
    clock: ClockDisplaySettings,
}
