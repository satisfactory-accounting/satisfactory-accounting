use std::fmt;
use std::str::FromStr;

use thiserror::Error;

use self::column::Column;

/// Max num digits is the length of u64 max value. This doesn't accunt for a decimal point.
const MAX_DIGITS: usize = "18446744073709551615".len();

const MAX_INTEGER: u64 = u64::MAX / Column::ONES.multiplier();

mod column {
    /// Column within the fixed point repr, represented as a shifted exponent.
    #[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
    pub(super) struct Column {
        /// Exponent. Column selected corresponds to 10^(exponent - 6).
        exponent: u32,
    }

    impl Column {
        /// Get a column for an exponent. Must be -6..=14
        pub(super) const fn exp(exp: i32) -> Self {
            if exp < -6 {
                panic!("Exponent out of range: exponent must be no less than -6");
            }
            if exp > 14 {
                panic!("Exponent out of range: exponent must be no more than 14");
            }
            Self {
                exponent: (exp + 6) as u32,
            }
        }

        pub(super) const MIN: Self = Self::exp(-6);
        pub(super) const TENTHS: Self = Self::exp(-1);
        pub(super) const ONES: Self = Self::exp(0);
        pub(super) const MAX: Self = Self::exp(14);

        /// Get a multiplier which is equivalent to 1 in this column.
        pub(super) const fn multiplier(self) -> u64 {
            10u64.pow(self.exponent)
        }

        /// Gets the exponent.
        pub(super) const fn exponent(self) -> i32 {
            self.exponent as i32 - 6
        }
    }
}

/// Error from trying to parse a clock multiplier.
#[derive(Debug, Error)]
pub enum ParseClockMultiplierError {
    #[error("Encountered an invalid digit")]
    InvalidDigit,
    #[error("Value was out of range")]
    OutOfRange,
}

/// Clock speed is a percentage stored to 4 decimal digits.
///
/// Internally we represent this as a fixed-point u64.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ClockMultiplier {
    /// Fixed-point clock speed representation. The real multiplier is `multipler / 10^6` (the
    /// percentage is `multiplier / 10^4`).
    multiplier: u64,
}

impl FromStr for ClockMultiplier {
    type Err = ParseClockMultiplierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl fmt::Display for ClockMultiplier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = [0u8; MAX_DIGITS + 1];
        let mut cursor = buf.len();
        // Whether we need to write zeroes at this stage. False until we hit our first nonzero digit
        // or the decimal place, unless trailing digits were requested by the precision.
        let mut write_zero = false;

        let start_column;
        let mut remaining;

        match f.precision() {
            Some(precision @ 0..=5) => {
                // We need to start writing zeros because they're requested by the precision.
                write_zero = true;
                start_column = Column::exp(-(precision as i32));
                remaining = round_to(self.multiplier, start_column) / start_column.multiplier();
            }
            Some(_) => {
                write_zero = true;
                start_column = Column::MIN;
                remaining = self.multiplier;
            }
            None => {
                start_column = Column::MIN;
                remaining = self.multiplier;
            }
        }

        for exp in start_column.exponent()..=Column::MAX.exponent() {
            let column = Column::exp(exp);
            if remaining == 0 && column > Column::ONES {
                break;
            }
            let digit = remaining % 10;
            remaining /= 10;
            if write_zero || digit != 0 || column >= Column::ONES {
                write_zero = true;
                let chr = char::from_digit(digit as u32, 10).unwrap();
                cursor -= 1;
                buf[cursor] = chr as u8;
                if column == Column::TENTHS {
                    cursor -= 1;
                    buf[cursor] = b'.';
                }
            }
        }

        let str = std::str::from_utf8(&buf[cursor..]).expect("created number is not valid utf-8");
        f.pad_integral(true, "", str)
    }
}

/// Round to a column given as a power of 10.
const fn round_to(val: u64, column: Column) -> u64 {
    let multiplier = column.multiplier();
    let fractional_part = val % multiplier;
    val - fractional_part
        + if fractional_part >= multiplier / 2 {
            multiplier
        } else {
            0
        }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a new [`ClockMultiplier`] using a fixed-point value. The actual clock multiplier is
    /// effectively the given multiplier divided by a million.
    fn fixed(multiplier: u64) -> ClockMultiplier {
        ClockMultiplier { multiplier }
    }

    macro_rules! assert_fmt {
        ($fmt:literal, $val:literal, $expect:literal) => {
            let val = fixed($val);
            let res = format!($fmt, val);
            assert_eq!(res, $expect, "Expected {} to format as \"{}\" with format \"{}\", but it was \"{}\"", val.multiplier, $expect, $fmt, res);
        };
    }

    #[test]
    fn format_simple() {
        assert_fmt!("{}", 10_023_000, "10.023");
        assert_fmt!("{}", 10_000_000, "10");
        assert_fmt!("{}", 1_000_000, "1");
        assert_fmt!("{}", 0, "0");
        assert_fmt!("{}", 100_000, "0.1");
        assert_fmt!("{}", 10_000, "0.01");
        assert_fmt!("{}", 1_000, "0.001");
        assert_fmt!("{}", 100, "0.0001");
        assert_fmt!("{}", 10, "0.00001");
        assert_fmt!("{}", 1, "0.000001");

        assert_fmt!("{}", 8_602, "0.008602");
        assert_fmt!("{}", 403_600, "0.4036");
    }

    #[test]
    fn format_round() {
        assert_fmt!("{:.0}", 10_023_000, "10");
        assert_fmt!("{:.0}", 10_500_000, "11");
        assert_fmt!("{:.0}", 10_499_999, "10");
        assert_fmt!("{:.0}", 023_000, "0");
        assert_fmt!("{:.0}", 500_000, "1");
        assert_fmt!("{:.0}", 499_999, "0");
        assert_fmt!("{:.0}", 449_999, "0");

        assert_fmt!("{:.1}", 10_023_000, "10.0");
        assert_fmt!("{:.1}", 10_025_000, "10.0");
        assert_fmt!("{:.1}", 10_500_000, "10.5");
        assert_fmt!("{:.1}", 10_499_999, "10.5");
        assert_fmt!("{:.1}", 10_449_999, "10.4");
        assert_fmt!("{:.1}", 023_000, "0.0");
        assert_fmt!("{:.1}", 023_000, "0.0");
        assert_fmt!("{:.1}", 500_000, "0.5");
        assert_fmt!("{:.1}", 499_999, "0.5");
        assert_fmt!("{:.1}", 449_999, "0.4");

        assert_fmt!("{:.2}", 680_023_000, "680.02");
        assert_fmt!("{:.2}", 680_025_000, "680.03");
        assert_fmt!("{:.2}", 680_500_000, "680.50");
        assert_fmt!("{:.2}", 680_499_999, "680.50");
        assert_fmt!("{:.2}", 680_449_999, "680.45");
        assert_fmt!("{:.2}", 023_000, "0.02");
        assert_fmt!("{:.2}", 025_000, "0.03");
        assert_fmt!("{:.2}", 500_000, "0.50");
        assert_fmt!("{:.2}", 499_999, "0.50");
        assert_fmt!("{:.2}", 449_999, "0.45");
    }

    #[test]
    fn format_pad() {
        assert_fmt!("{:3.1}", 10_023_000, "10.0");
        assert_fmt!("{:5.1}", 10_025_000, " 10.0");
        assert_fmt!("{:06.1}", 10_500_000, "0010.5");
        assert_fmt!("{:+.1}", 10_499_999, "+10.5");
        assert_fmt!("{:.1}", 10_499_999, "10.5");
        assert_fmt!("{:.1}", 10_449_999, "10.4");
        assert_fmt!("{:07}", 023_000, "000.023");
        assert_fmt!("{:+}", 023_000, "+0.023");
        assert_fmt!("{:+}", 0, "+0");
        assert_fmt!("{:+6}", 500_000, "  +0.5");
    }
}
