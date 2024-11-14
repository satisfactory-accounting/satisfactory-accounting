use log::{info, warn};
use satisfactory_accounting::accounting::{
    BuildNode, Building, BuildingSettings, ManufacturerSettings, Node,
};
use satisfactory_accounting::database::{BuildingKind, ItemIdOrPower, Manufacturer, Power};
use serde::{Deserialize, Serialize};

use crate::node_display::NodeDisplay;

/// Container for settings related to backdriving.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackdriveSettings {
    /// Backdrive settings for manufacturers (constructors, assemblers, etc)
    manufacturer: BuildingBackdriveSettings,
    /// Backdrive settings for extractors (miners, wells, etc)
    extractor: BuildingBackdriveSettings,
    /// Backdrive settings for generators.
    generator: BuildingBackdriveSettings,
}

impl Default for BackdriveSettings {
    fn default() -> Self {
        Self {
            manufacturer: BuildingBackdriveSettings {
                mode: BackdriveMode::VariableClock,
                uniform_max_clock: 1.0,
            },
            extractor: BuildingBackdriveSettings {
                mode: BackdriveMode::UniformClock,
                uniform_max_clock: 2.5,
            },
            generator: BuildingBackdriveSettings {
                mode: BackdriveMode::VariableClock,
                uniform_max_clock: 1.0,
            },
        }
    }
}

/// Which mode backdriving operates in.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum BackdriveMode {
    /// The clock speed will not be modified, and we will find the minimum number of machines needed
    /// to reach a certain output rate. Any overflow will be handled by having one extra machine
    /// with a reduced clock rate.
    VariableClock,
    /// The multiplier will be set to an integer, and all machines will have a uniform clock speed.
    UniformClock,
}

/// Settings to use for a particular building type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct BuildingBackdriveSettings {
    /// Which backdrive mode to use for this equipment.
    mode: BackdriveMode,
    /// Maximum clock speed to use when operating in uniform clock mode.
    uniform_max_clock: f32,
}

impl NodeDisplay {
    /// Tries to backdrive this node to make the given item have the given rate. Returns a new node
    /// if backdriving succeeds, or None if backdriving fails.
    pub(super) fn backdrive(&self, node: &Node, id: ItemIdOrPower, rate: f32) -> Option<Node> {
        info!("Backdrive {id:?} to {rate}");
        let building = node.building().or_else(|| {
            warn!("Cannot backdrive non-buildings.");
            None
        })?;
        let building_id = building.building.or_else(|| {
            warn!("Cannot backdrive, buiilding not set");
            None
        })?;

        let building_type = self.db.get(building_id).or_else(|| {
            warn!("Cannot backdrive, building not recognized");
            None
        })?;

        let new_bldg = match (&building.settings, &building_type.kind) {
            (BuildingSettings::Manufacturer(ms), BuildingKind::Manufacturer(m)) => {
                let (copies, ms) = self.backdrive_manufacturer(id, rate, ms, m)?;
                Building {
                    copies,
                    settings: ms.into(),
                    ..building.clone()
                }
            }
            _ => {
                warn!("Unsupported backdrive combo");
                return None;
            }
        };

        new_bldg
            .build_node(&self.db)
            .inspect_err(|e| warn!("Unable to build node after backdriving: {}", e))
            .ok()
    }

    /// Backdrive a manufacturer, returning the new number of virtual copies and the new
    /// manufacturer settings.
    fn backdrive_manufacturer(
        &self,
        id: ItemIdOrPower,
        rate: f32,
        ms: &ManufacturerSettings,
        m: &Manufacturer,
    ) -> Option<(f32, ManufacturerSettings)> {
        match id {
            ItemIdOrPower::Power => {
                let res = backdrive_power_consumer(
                    ms.clock_speed,
                    rate,
                    &m.power_consumption,
                    &self.user_settings.backdrive_settings.manufacturer,
                )?;
                let mut ms = ms.clone();
                ms.clock_speed = res.clock;
                Some((res.copies, ms))
            }
            ItemIdOrPower::Item(item_id) => {
                todo!()
            }
        }
    }
}

/// Result of backdriving for power.
struct PowerBackdriveResult {
    /// New number of virtual copies.
    copies: f32,
    /// New clock speed.
    clock: f32,
}

/// Calculate the new clock speed virtual copies for a power-consuming building.
fn backdrive_power_consumer(
    current_clock: f32,
    rate: f32,
    power: &Power,
    settings: &BuildingBackdriveSettings,
) -> Option<PowerBackdriveResult> {
    if power.power == 0.0 {
        warn!("Cannot backdrive power consumption, because the power consumption is 0");
        return None;
    }
    // Compute the multiplier using only positive values.
    let rate = rate.abs();
    if power.power_exponent == 0.0 {
        // If overclocking isn't allowed, then both variable clock and uniform clock end up donig
        // the same thing; we just find the integer multiplier that gets the closest power
        // consumption.
        let copies = (rate / power.power).round();
        return Some(PowerBackdriveResult {
            copies,
            // any_clock^0 = 1, so this is fine.
            clock: 1.0,
        });
    }
    match settings.mode {
        BackdriveMode::VariableClock => {
            // For variable clock speed, we keep the current clock constant, so we need to solve
            // this equation for whole and fractional copies:
            //
            // rate = whole_copes * power * clock_speed ^ power_exponent
            //        + power * (fractional_copies * clock_speed) ^ power_exponent
            //
            // Factor out:
            //
            // rate = power * (whole_copies * clock_speed ^ power_exponent
            //                 + (fractional_copies * clock_speed) ^ power_exponent)
            //
            // Distribute the exponent, then factor out more:
            //
            // rate = power * clock_speed ^ power_exponent
            //        * (whole_copies + fractional_copies ^ power_exponent)
            //
            // Divide:
            //
            // rate / (power * clock_speed ^ power_expoent)
            //     = whole_copies + fractional_copies ^ power_exponent
            //
            // Since we have fractional_copies < 1, we know that
            // fractional_copies ^ power_exponent < 1.
            // This means that if we solve for:
            //
            // combined_multiplier = whole_copies + partial_copies
            // where
            //   whole_copies = combined_multiplier.trunc()
            //   partial_copies = combined_multiplier.fract()
            //
            // Since partial_copies is < 1, we can then just do:
            //
            // fractional_copies = partial_copies ^ (1/power_exponent)
            //
            // then add the result back into whole_copies to get our final multiplier, accounting
            // for partial clocks.
            let rate_per_power_clock =
                rate / (power.power * current_clock.powf(power.power_exponent));
            let whole_copies = rate_per_power_clock.trunc();
            let fractional_copies = rate_per_power_clock
                .fract()
                .powf(1.0 / power.power_exponent);
            Some(PowerBackdriveResult {
                copies: whole_copies + fractional_copies,
                clock: current_clock,
            })
        }
        BackdriveMode::UniformClock => {
            // We want to solve:
            // rate = multiplier * power * clock_speed ^ power_exponent
            // such that clock_speed <= 1.
            // We can ensure this by choosing the smallest multiplier such that
            // that power is greater than the requested amount, then calculating
            // a clock speed that reduces it appropriately.
            let multiplier = (rate / power.power).ceil();
            // We now need to solve the above equation for clock_speed, but with
            // multiplier treated as a constant rather than a variable.
            // rate / multiplier = power * clock_speed ^ power_exponent
            // rate / (multiplier * power) = clock_speed ^ power_exponent
            // (rate / (multiplier * power)) ^ (1/power_exponent) = clock_speed
            // Special case: if the building is not overclockable
            // (power_exponent == 0), we just set the clock_speed to 1.
            let clock_speed = {
                let rate_per_multiplied_power = rate / (multiplier * power.power);
                rate_per_multiplied_power.powf(1.0 / power.power_exponent)
            };
            todo!()
        }
    }
}
