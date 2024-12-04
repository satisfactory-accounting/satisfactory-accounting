use std::fmt;

use super::{NumberFormatMode, NumberFormatSettings};

/// Helper trait for applying user configured formatting.
pub trait UserConfiguredFormat {
    type Formatter<'s>: fmt::Display + 's
    where
        Self: 's;

    /// Get a formatter which formats self according to the given format settings.
    fn format<'s>(&self, settings: &'s NumberFormatSettings) -> Self::Formatter<'s>;

    /// Round self according to the format settings to get a value which can be compared in order to
    /// make conditional formats like "color if < 0" match the value produced by format.
    fn round_by_format(&self, settings: &NumberFormatSettings) -> Self
    where
        Self: Sized;
}

impl UserConfiguredFormat for f32 {
    type Formatter<'s> = F32Formatter<'s>;

    #[inline]
    fn format<'s>(&self, settings: &'s NumberFormatSettings) -> Self::Formatter<'s> {
        F32Formatter {
            settings,
            val: *self,
        }
    }

    fn round_by_format(&self, settings: &NumberFormatSettings) -> Self {
        match settings.mode {
            NumberFormatMode::DecimalPrecise => *self,
            NumberFormatMode::DecimalRounded | NumberFormatMode::DecimalRoundedPadded => {
                let places = 10u32.pow(settings.round_decimal_places) as f32;
                (*self * places).round() / places
            }
        }
    }
}

/// Formatter which formats an f32 according to some format settings.
#[derive(Copy, Clone)]
pub struct F32Formatter<'s> {
    settings: &'s NumberFormatSettings,
    val: f32,
}

impl<'s> fmt::Display for F32Formatter<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.settings.mode {
            NumberFormatMode::DecimalPrecise => write!(f, "{}", self.val),
            NumberFormatMode::DecimalRounded => {
                let val = self.val.round_by_format(self.settings);
                write!(f, "{}", val)
            }
            NumberFormatMode::DecimalRoundedPadded => {
                // We round explicitly in addition to specifying a displayed precision because the
                // f32 formatter precision setting uses round ties even instead of round ties away
                // from zero.
                let val = self.val.round_by_format(self.settings);
                write!(
                    f,
                    "{val:.precision$}",
                    precision = self.settings.round_decimal_places as usize
                )
            }
        }
    }
}
