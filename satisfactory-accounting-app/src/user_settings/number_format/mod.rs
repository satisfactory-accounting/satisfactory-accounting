use serde::{Deserialize, Serialize};

pub use formatters::UserConfiguredFormat;
pub use settings_page::{NumberDisplaySettingsMsg, NumberDisplaySettingsSection};

mod formatters;
mod settings_page;

/// How to style numbers (e.g. color them for positive/negative) in relation to their rounding.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumberStylingMode {
    /// Style based on the displayed value.
    #[default]
    DisplayedValue,
    /// Style based on the exact value, even if that doesn't match the rounded value.
    ExactValue,
}

/// How to style numbers based on their rounded values.
///
/// This is a struct wrapping the [`NumberStylingMode`] enum for future expandability.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NumberStylingSettings {
    /// The styling mode to use.
    pub mode: NumberStylingMode,
}

/// How to format numbers for display in the balance, clock speed, etc.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumberFormatMode {
    /// Format the numbers in decimal, with as much precision as is available.
    DecimalPrecise,
    /// Format the numbers in decimal, rounded to some number of digits after the decimal point.
    DecimalRounded,
    /// Format the numbers in decimal, rouned to some number of digits after the decimal point. Also
    /// pad out to that number of digits with zeros.
    DecimalRoundedPadded,
}

/// Number display settings for a particular kind of number (clock speed, balances, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumberFormatSettings {
    /// How to format the numbers.
    pub mode: NumberFormatMode,
    /// Number of decimal places to keep when rounding.
    pub round_decimal_places: u32,
}

/// Settings to apply to balance display.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceDisplaySettings {
    /// Whether the red/green/black coloring is based on the exact value or rounded value.
    pub highlight_style: NumberStylingSettings,
    /// Whether hide-empty-balances should be based on the exact value or rounded value.
    pub hide_style: NumberStylingSettings,
    /// Format settings to use for power.
    ///
    /// This is broken out in anticipation of fraction mode, where power will still be floating
    /// point. For now it is locked to follow item_format_settings.
    pub power_format_settings: NumberFormatSettings,
    /// Format settings to use for items.
    pub item_format_settings: NumberFormatSettings,
}

impl Default for BalanceDisplaySettings {
    fn default() -> Self {
        let format = NumberFormatSettings {
            mode: NumberFormatMode::DecimalRounded,
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
    pub format: NumberFormatSettings,
}

impl Default for ClockDisplaySettings {
    fn default() -> Self {
        Self {
            format: NumberFormatSettings {
                mode: NumberFormatMode::DecimalPrecise,
                round_decimal_places: 6,
            },
        }
    }
}

/// Settings to apply to multiplier display.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiplierDisplaySettings {
    /// Number format settings to apply to the multiplier.
    pub format: NumberFormatSettings,
}

impl Default for MultiplierDisplaySettings {
    fn default() -> Self {
        Self {
            format: NumberFormatSettings {
                mode: NumberFormatMode::DecimalPrecise,
                round_decimal_places: 6,
            },
        }
    }
}

/// Settings related to how various numbers are displayed.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumberDisplaySettings {
    /// How to display balances.
    #[serde(default)]
    pub balance: BalanceDisplaySettings,
    /// How to display the clock speed.
    #[serde(default)]
    pub clock: ClockDisplaySettings,
    /// Display settings to apply to multipliers.
    #[serde(default)]
    pub multiplier: MultiplierDisplaySettings,
}
