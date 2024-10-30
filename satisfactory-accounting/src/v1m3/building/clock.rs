/// Clock speed is a percentage stored to 4 decimal digits.
///
/// Internally we represent this as a fixed-point u64.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ClockMultiplier {
    /// Fixed-point clock speed representation. The real multiplier is `multipler / 10^6` (the
    /// percentage is `multiplier / 10^4`).
    multiplier: u64,
}
