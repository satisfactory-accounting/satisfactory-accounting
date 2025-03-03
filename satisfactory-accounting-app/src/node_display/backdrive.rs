use log::{info, warn};
use satisfactory_accounting::accounting::{
    BalanceAdjustmentSettings, BuildNode, Building, BuildingSettings, GeneratorSettings,
    GeothermalSettings, ManufacturerSettings, MinerSettings, Node, PumpSettings, ResourcePurity,
    MAX_CLOCK, MIN_CLOCK,
};
use satisfactory_accounting::database::{
    BuildingKind, Generator, Geothermal, ItemId, ItemIdOrPower, Manufacturer, Miner, Power,
    PowerConsumer, Pump,
};
use serde::{Deserialize, Serialize};
use yew::{function_component, html, use_callback, Html, Properties};

use crate::inputs::toggle::MaterialRadio;
use crate::node_display::clock::ClockSpeed;
use crate::node_display::NodeDisplay;
use crate::user_settings::{use_user_settings, use_user_settings_dispatcher};

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

impl BackdriveSettings {
    /// Updates the backdriving settings. Returns true if the settings changed.
    pub fn update(&mut self, msg: BackdriveSettingsMsg) -> bool {
        match msg.action {
            BackdriveSettingsAction::SetMaxClock {
                building_type,
                uniform_max_clock,
            } => {
                let uniform_max_clock = uniform_max_clock.clamp(MIN_CLOCK, MAX_CLOCK);
                let settings = self.select_building_type_mut(building_type);
                if settings.uniform_max_clock == uniform_max_clock {
                    false
                } else {
                    settings.uniform_max_clock = uniform_max_clock;
                    true
                }
            }
            BackdriveSettingsAction::SetMode {
                building_type,
                mode,
            } => {
                let settings = self.select_building_type_mut(building_type);
                if settings.mode == mode {
                    false
                } else {
                    settings.mode = mode;
                    true
                }
            }
        }
    }

    /// Gets the backdrive building settings for the given building type category.
    fn select_building_type(
        &self,
        building_type: BackdriveSettingsType,
    ) -> &BuildingBackdriveSettings {
        match building_type {
            BackdriveSettingsType::Manufacturer => &self.manufacturer,
            BackdriveSettingsType::Extractor => &self.extractor,
            BackdriveSettingsType::Generator => &self.generator,
        }
    }

    /// Gets the backdrive building settings for the given building type category.
    fn select_building_type_mut(
        &mut self,
        building_type: BackdriveSettingsType,
    ) -> &mut BuildingBackdriveSettings {
        match building_type {
            BackdriveSettingsType::Manufacturer => &mut self.manufacturer,
            BackdriveSettingsType::Extractor => &mut self.extractor,
            BackdriveSettingsType::Generator => &mut self.generator,
        }
    }
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

/// Message to update the backdriving settings.
pub struct BackdriveSettingsMsg {
    action: BackdriveSettingsAction,
}

/// Actions to apply to the backdrive settings.
enum BackdriveSettingsAction {
    /// Set the maximum clock rate for one of the categories.
    SetMaxClock {
        building_type: BackdriveSettingsType,
        uniform_max_clock: f32,
    },
    /// Set the mode for one of the categories.
    SetMode {
        building_type: BackdriveSettingsType,
        mode: BackdriveMode,
    },
}

/// For actions that act on BuildingBackdriveSettings, selects which buildng type to apply to.
#[derive(Copy, Clone, PartialEq, Eq)]
enum BackdriveSettingsType {
    Manufacturer,
    Extractor,
    Generator,
}

/// Displays the settings section for controlling backdrive settings.
#[function_component]
pub fn BackdriveSettingsSection() -> Html {
    html! {
        <div class="settings-section">
            <h2>{"Backdriving Settings"}</h2>
            <p>{"Backdriving allows you to click on the outputs of a building and type the number \
            of items you want it to produce and have it calculate a clock speed and building \
            multiplier to get that production rate. There are two supported modes for backdriving:"}</p>
            <ul class="description">
                <li><p><b>{"Mixed Clock Mode:"}</b>{" In this mode, the clock speed that is set on \
                the building doesn't get modified. Instead, we set only the building's multiplier \
                to produce the correct number of items. If the number of items isn't an exact \
                multiple of the production rate at the current clock speed, we use a fractional \
                building  multiplier."}</p>
                <p>{"Fractional building multipliers represent N buildings at the stated clock \
                speed + 1 buiding at a fractional clock speed, for example, if a building is set \
                to a clock speed of 2.0, and you set the multiplier to 3.75, that means 3 \
                buildings with a clock speed of 2.0 plus 1 building with a clock speed of 1.5."}</p>
                <p>{"Mixed clock mode allows you to minimize the number of buildings that need \
                overclocking, since the clock speed stays the same except for the last building."}
                </p></li>
                <li><p><b>{"Uniform Clock Mode:"}</b>{" In this mode, both the clock speed and \
                number of buildings are modified. The multiplier will always be set to a whole \
                number of buildings, and then all buildings will be over or under clocked to get \
                the correct number of output items."}</p>
                <p>{"Uniform clock mode requires overclocking on more buildings, but is nice when \
                you want to minimize the number of buildings, for example for miners. You can set \
                a limit on the clock speed it is allowed to ramp up to."}</p></li>
            </ul>
            <p>{"You can set separate backdriving settings for three categories of buildings: \
            production buildings, such as Constructors, Assemblers, and Manufacturers, extraction \
            buildings such as Miners and Resource Wells, and Generators."}</p>
            <p>{"Note that for buildings that don't allow overclocking, neither mode works, and we \
            instead just choose enough buildings to get at least as many items as you requested."}</p>
            <div class="settings-subsection">
                <h3>{"Production Building Settings"}</h3>
                <BuildingSubsection section={BackdriveSettingsType::Manufacturer} />
            </div>
            <div class="settings-subsection">
                <h3>{"Extraction Building Settings"}</h3>
                <BuildingSubsection section={BackdriveSettingsType::Extractor} />
            </div>
            <div class="settings-subsection">
                <h3>{"Generator Building Settings"}</h3>
                <BuildingSubsection section={BackdriveSettingsType::Generator} />
            </div>
        </div>
    }
}

#[derive(PartialEq, Properties)]
struct BuildingSubsectionProps {
    /// Which settings type to show in this section.
    section: BackdriveSettingsType,
}

/// Displays building settings for one building type.
#[function_component]
fn BuildingSubsection(props: &BuildingSubsectionProps) -> Html {
    let user_settings = use_user_settings();
    let user_settings_dispatcher = use_user_settings_dispatcher();

    let update_max_speed = use_callback(
        (props.section, user_settings_dispatcher.clone()),
        |speed, (section, user_settings_dispatcher)| {
            user_settings_dispatcher.update_backdrive_settings(BackdriveSettingsMsg {
                action: BackdriveSettingsAction::SetMaxClock {
                    building_type: *section,
                    uniform_max_clock: speed,
                },
            });
        },
    );

    let set_mixed = use_callback(
        (props.section, user_settings_dispatcher.clone()),
        |_, (section, user_settings_dispatcher)| {
            user_settings_dispatcher.update_backdrive_settings(BackdriveSettingsMsg {
                action: BackdriveSettingsAction::SetMode {
                    building_type: *section,
                    mode: BackdriveMode::VariableClock,
                },
            });
        },
    );

    let set_uniform = use_callback(
        (props.section, user_settings_dispatcher.clone()),
        |_, (section, user_settings_dispatcher)| {
            user_settings_dispatcher.update_backdrive_settings(BackdriveSettingsMsg {
                action: BackdriveSettingsAction::SetMode {
                    building_type: *section,
                    mode: BackdriveMode::UniformClock,
                },
            });
        },
    );

    let settings = user_settings
        .backdrive_settings
        .select_building_type(props.section);

    html! {
        <ul>
            <li>
                <label>
                    <span>{"Mixed Clock Speed"}</span>
                    <MaterialRadio
                        checked={settings.mode == BackdriveMode::VariableClock}
                        onclick={set_mixed} />
                </label>
            </li>
            <li>
                <label>
                    <span>{"Uniform Clock Speed"}</span>
                    <span class="max-uniform-clock">
                        <span>{"Max clock speed"}</span>
                        <ClockSpeed clock_speed={settings.uniform_max_clock} copies=1.0
                            on_update_speed={update_max_speed} />
                    </span>
                    <MaterialRadio
                        checked={settings.mode == BackdriveMode::UniformClock}
                        onclick={set_uniform} />
                </label>
            </li>
        </ul>
    }
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
            warn!("Cannot backdrive, building not set");
            None
        })?;

        let building_type = self.db.get(building_id).or_else(|| {
            warn!("Cannot backdrive, building not recognized");
            None
        })?;
        // Backdriving calculations never care about positive vs negative, since that's fixed by the
        // recipie/building except for Backdrive.
        let sign = rate.signum();
        let rate = rate.abs();

        let (copies, settings) = match (&building.settings, &building_type.kind) {
            (BuildingSettings::Manufacturer(ms), BuildingKind::Manufacturer(m)) => {
                let (copies, ms) = self.backdrive_manufacturer(id, rate, ms, m)?;
                (copies, ms.into())
            }
            (BuildingSettings::Miner(ms), BuildingKind::Miner(m)) => {
                let (copies, ms) = self.backdrive_miner(id, rate, ms, m)?;
                (copies, ms.into())
            }
            (BuildingSettings::Generator(gs), BuildingKind::Generator(g)) => {
                let (copies, gs) = self.backdrive_generator(id, rate, gs, g)?;
                (copies, gs.into())
            }
            (BuildingSettings::Pump(ps), BuildingKind::Pump(p)) => {
                let (copies, ps) = self.backdrive_pump(id, rate, ps, p)?;
                (copies, ps.into())
            }
            (BuildingSettings::Geothermal(gs), BuildingKind::Geothermal(g)) => (
                self.backdrive_geothermal(id, rate, gs, g)?,
                gs.clone().into(),
            ),
            (BuildingSettings::PowerConsumer, BuildingKind::PowerConsumer(p)) => (
                self.backdrive_power_consumer(id, rate, p)?,
                BuildingSettings::PowerConsumer.into(),
            ),
            (BuildingSettings::Station(_), BuildingKind::Station(_)) => {
                warn!("Stations do not support backdriving");
                return None;
            }
            (BuildingSettings::BalanceAdjustment(bas), BuildingKind::BalanceAdjustment(_)) => {
                let (copies, bas) =
                    backdrive_balance_adjustment(id, rate * sign, bas, building.copies)?;
                (copies, bas.into())
            }
            _ => {
                warn!("Building Settings don't match Building Kind");
                return None;
            }
        };
        let new_bldg = Building {
            copies,
            settings,
            ..building.clone()
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
        let res = match id {
            ItemIdOrPower::Power => backdrive_power_consumer(
                ms.clock_speed,
                rate,
                &m.power_consumption,
                &self.user_settings.backdrive_settings.manufacturer,
            )?,
            ItemIdOrPower::Item(item_id) => {
                let recipe_id = ms.recipe.or_else(|| {
                    warn!("Unable to backdrive - no recipe set");
                    None
                })?;
                let recipe = self.db.get(recipe_id).or_else(|| {
                    warn!("Unable to backdrive - recipe  not recognized");
                    None
                })?;

                let total_input_count: f32 = recipe
                    .ingredients
                    .iter()
                    .filter(|ing| ing.item == item_id)
                    .map(|ing| ing.amount)
                    .sum();
                let total_output_count: f32 = recipe
                    .products
                    .iter()
                    .filter(|ing| ing.item == item_id)
                    .map(|ing| ing.amount)
                    .sum();
                // For backdriving calculations, we don't care if it's an input or an output, so we
                // just use abs here.
                let item_net_rate =
                    (total_input_count - total_output_count).abs() / recipe.time * 60.0;

                backdrive_production_consumption(
                    ms.clock_speed,
                    rate,
                    item_net_rate,
                    m.overclockable(),
                    &self.user_settings.backdrive_settings.manufacturer,
                )?
            }
        };
        let mut ms = ms.clone();
        ms.clock_speed = res.clock;
        Some((res.copies, ms))
    }

    /// Backdrive a miner, returning the new number of virtual copies and the new miner settings.
    fn backdrive_miner(
        &self,
        id: ItemIdOrPower,
        rate: f32,
        ms: &MinerSettings,
        m: &Miner,
    ) -> Option<(f32, MinerSettings)> {
        let res = match id {
            ItemIdOrPower::Power => backdrive_power_consumer(
                ms.clock_speed,
                rate,
                &m.power_consumption,
                &self.user_settings.backdrive_settings.extractor,
            )?,
            ItemIdOrPower::Item(item_id) => {
                let resource_id = ms.resource.or_else(|| {
                    warn!("Unable to backdrive - no resource selected");
                    None
                })?;
                if item_id != resource_id {
                    warn!("Unable to backdrive - backdriving resource doesn't match selected resource");
                    return None;
                }

                // For backdriving calculations, we don't care if it's an input or an output, so we
                // just use abs here.
                let base_cycles_per_minute = 60.0 / m.cycle_time * ms.purity.speed_multiplier();
                let base_item_rate = (base_cycles_per_minute * m.items_per_cycle).abs();

                backdrive_production_consumption(
                    ms.clock_speed,
                    rate,
                    base_item_rate,
                    m.overclockable(),
                    &self.user_settings.backdrive_settings.extractor,
                )?
            }
        };
        let mut ms = ms.clone();
        ms.clock_speed = res.clock;
        Some((res.copies, ms))
    }

    /// Backdrive a generator, returning the new number of virtual copies and the new generator settings.
    fn backdrive_generator(
        &self,
        id: ItemIdOrPower,
        rate: f32,
        gs: &GeneratorSettings,
        g: &Generator,
    ) -> Option<(f32, GeneratorSettings)> {
        let res = match id {
            ItemIdOrPower::Power => backdrive_power_producer(
                gs.clock_speed,
                rate,
                &g.power_production,
                &self.user_settings.backdrive_settings.extractor,
            )?,
            ItemIdOrPower::Item(item_id) => {
                // We have 3 distinct cases to cover:
                // 1. The item_id is the selected fuel
                // 2. The item_id is water.
                // 3. The item_id is a fuel burn byproduct
                let fuel_id = gs.fuel.or_else(|| {
                    warn!("Unable to backdrive - no fuel selected");
                    None
                })?;
                let fuel = self.db.get(fuel_id).or_else(|| {
                    warn!("Unable to backdrive - fuel not recognized");
                    None
                })?;
                let fuel = fuel.fuel.as_ref().or_else(|| {
                    warn!("Unable to backdrive - selected fuel item is not a fuel");
                    None
                })?;

                // All generator item rates are actually based on power production, so we have to
                // convert from item rate to power rate and then use power backdriving to get an
                // answer that is corect across historical versions where power production is
                // non-linear. This is easier in the current version of the game where all
                // generators are just linearly clocked, but we maintain backwards compatibility for
                // now.

                let power_rate = if item_id == fuel_id {
                    if fuel.energy == 0.0 {
                        warn!("Unable to backdrive - fuel energy is 0");
                        return None;
                    }
                    // Energy MJ * (Fuel / min) * (min / sec) = Energy MJ / sec = Power MW
                    fuel.energy * rate / 60.0
                } else if let Some(byproduct_rate) =
                    fuel.byproducts.iter().find(|by| by.item == item_id)
                {
                    if byproduct_rate.amount == 0.0 {
                        warn!("Unable to backdrive - byproduct production is 0");
                        return None;
                    }
                    // (Items / min) / (Items / Fuel) = (Fuel / min)
                    let fuel_rate = rate / byproduct_rate.amount;
                    // see above
                    fuel.energy * fuel_rate / 60.0
                } else if item_id == ItemId::water() {
                    if g.used_water == 0.0 {
                        warn!("Unable to backdrive - water consumption is 0");
                        return None;
                    }
                    // (Water / min) / (Water / Power MW) = Power MW
                    rate / g.used_water
                } else {
                    warn!("Unable to backdrive - item {item_id:?} is not the fuel, a byproduct or water");
                    return None;
                };

                backdrive_power_producer(
                    gs.clock_speed,
                    power_rate,
                    &g.power_production,
                    &self.user_settings.backdrive_settings.generator,
                )?
            }
        };
        let mut gs = gs.clone();
        gs.clock_speed = res.clock;
        Some((res.copies, gs))
    }

    /// Backdrives geothermal to a particular power production rate.
    fn backdrive_geothermal(
        &self,
        id: ItemIdOrPower,
        rate: f32,
        gs: &GeothermalSettings,
        g: &Geothermal,
    ) -> Option<f32> {
        if g.power == 0.0 {
            warn!("Unable to backdrive - geothermal has no power production");
            return None;
        }
        if id != ItemIdOrPower::Power {
            warn!("Unable to backdrive - geothermal can only backdrive power");
            return None;
        }
        // Geothermal doesn't allow overclocking so just round up.
        let multiplier = rate / (g.power * gs.purity.speed_multiplier());
        Some(multiplier.ceil())
    }

    /// Backdrivea pump to match a particular resource production rate.
    fn backdrive_pump(
        &self,
        id: ItemIdOrPower,
        rate: f32,
        ps: &PumpSettings,
        p: &Pump,
    ) -> Option<(f32, PumpSettings)> {
        let res = match id {
            ItemIdOrPower::Power => backdrive_power_consumer(
                ps.clock_speed,
                rate,
                &p.power_consumption,
                &self.user_settings.backdrive_settings.extractor,
            )?,
            ItemIdOrPower::Item(item_id) => {
                let resource_id = ps.resource.or_else(|| {
                    warn!("Unable to backdrive - no resource selected");
                    None
                })?;
                if item_id != resource_id {
                    warn!("Unable to backdrive - backdriving resource doesn't match selected resource");
                    return None;
                }

                let base_cycles_per_minute = 60.0 / p.cycle_time;
                let base_item_rate = (base_cycles_per_minute
                    * (ps.pure_pads as f32 * ResourcePurity::Pure.speed_multiplier()
                        + ps.normal_pads as f32 * ResourcePurity::Normal.speed_multiplier()
                        + ps.impure_pads as f32 * ResourcePurity::Impure.speed_multiplier()))
                .abs();

                backdrive_production_consumption(
                    ps.clock_speed,
                    rate,
                    base_item_rate,
                    p.overclockable(),
                    &self.user_settings.backdrive_settings.extractor,
                )?
            }
        };
        let mut ps = ps.clone();
        ps.clock_speed = res.clock;
        Some((res.copies, ps))
    }

    /// Power consumers always just backdrive to a rounded up rate.
    fn backdrive_power_consumer(
        &self,
        id: ItemIdOrPower,
        rate: f32,
        p: &PowerConsumer,
    ) -> Option<f32> {
        if id != ItemIdOrPower::Power {
            warn!("Unable to backdrive power consumer -- requested something other than power");
            return None;
        }

        if p.power == 0.0 {
            warn!("Unable to backdrive - power consumer does not consume any power");
            return None;
        }

        let multiplier = rate / p.power;
        Some(multiplier.ceil())
    }
}

/// Backdrive a balance adjustment.
fn backdrive_balance_adjustment(
    id: ItemIdOrPower,
    rate: f32,
    bas: &BalanceAdjustmentSettings,
    copies: f32,
) -> Option<(f32, BalanceAdjustmentSettings)> {
    let current = match bas.item_or_power {
        Some(id) => id,
        None => {
            warn!("Unable to backdrive balance adjustment -- item not set");
            return None;
        }
    };
    if current != id {
        warn!("Cannot backdrive, requested item does not match current item");
        return None;
    }
    Some((
        copies,
        BalanceAdjustmentSettings {
            rate: rate / copies,
            ..bas.clone()
        },
    ))
}

/// Result of backdriving for power.
struct BackdriveResult {
    /// New number of virtual copies.
    copies: f32,
    /// New clock speed.
    clock: f32,
}

/// Calculate the new clock speed and virtual copies for a power-consuming building, based on
/// requested power usage.
///
/// *   `current_clock`: the current clock speed, used in
///     [`VariableClock`][BackdriveMode::VariableClock] mode.
/// *   `rate`: the requested power consumption rate. Must be positive.
/// *   `power`: power consumption values for this building.
/// *   `settings`: backdrive settings for this building type.
fn backdrive_power_consumer(
    current_clock: f32,
    rate: f32,
    power: &Power,
    settings: &BuildingBackdriveSettings,
) -> Option<BackdriveResult> {
    if power.power == 0.0 {
        warn!("Cannot backdrive power consumption, because the power consumption is 0");
        return None;
    }
    if power.power_exponent == 0.0 {
        // If overclocking isn't allowed, then both variable clock and uniform clock end up donig
        // the same thing; we just find the integer multiplier that gets at least that much
        // consumption.
        let copies = (rate / power.power).ceil();
        return Some(BackdriveResult {
            copies,
            // any_clock^0 = 1, so this is fine.
            clock: 1.0,
        });
    }
    Some(match settings.mode {
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
            BackdriveResult {
                copies: whole_copies + fractional_copies,
                clock: current_clock,
            }
        }
        BackdriveMode::UniformClock => {
            // For uniform clock speed, we first compute an overall multiplier then split it over an
            // integer number of machines based on the clock speed limit. We're trying to solve:
            //
            // rate = copies * power * clock_speed ^ power_exponent
            //
            // such that clock_speed <= uniform_max_clock. First we'll treat clock_speed as a
            // constant equal to uniform_max_clock and solve for copies to get an overall
            // multiplier. Then we'll take the ceiling of the multiplier to get the number of copies
            // and re-solve for the clock_speed.
            //
            // rate / (power * clock_speed ^ power_exponent) = copies
            let overall_multiplier =
                rate / (power.power * settings.uniform_max_clock.powf(power.power_exponent));
            let copies = overall_multiplier.ceil();

            // rate / (power * copies) = clock_speed ^ power_exponent
            //
            // (rate / (power * copies)) ^ (1/power_exponent) = clock_speed
            let rate_per_machine_power = rate / (power.power * copies);
            let clock = rate_per_machine_power.powf(1.0 / power.power_exponent);
            BackdriveResult { copies, clock }
        }
    })
}

/// Calculate the new clock speed and virtual copies for an item-consuming/producing building, based
/// on requested item rate.
///
/// *   `current_clock`: the current clock speed, used in
///     [`VariableClock`][BackdriveMode::VariableClock] mode.
/// *   `rate`: the requested item consumption/production rate. Must be positive.
/// *   `base_rate`: the rate of consumption/production for the building/recipe. Must be positive.
/// *   `overclockable`: Whether the building allows overclocking.
/// *   `settings`: backdrive settings for this building type.
fn backdrive_production_consumption(
    current_clock: f32,
    rate: f32,
    base_rate: f32,
    overclockable: bool,
    settings: &BuildingBackdriveSettings,
) -> Option<BackdriveResult> {
    if base_rate == 0.0 {
        warn!("Cannot backdrive item because its production rate is 0.");
        return None;
    }

    info!("backdrive: rate {rate}, base_rate: {base_rate}, current_clock: {current_clock}");

    let overall_multiplier = rate / base_rate;

    if !overclockable {
        // If overclocking isn't allowed, then both variable clock and uniform clock end up donig
        // the same thing; we just find the integer multiplier that gets at least that much
        // production/consumption.
        return Some(BackdriveResult {
            copies: overall_multiplier.ceil(),
            // any_clock^0 = 1, so this is fine.
            clock: 1.0,
        });
    }

    Some(match settings.mode {
        BackdriveMode::VariableClock => {
            // In variable clock mode, we don't modify the clock speed. We need to solve:
            //
            // rate = copies * base_rate * clock_speed;
            //
            // We already have
            //
            // overall_multiplier = rate / base_rate
            //
            // therefore
            //
            // overall_multiplier = copies * clock_speed
            let copies = overall_multiplier / current_clock;
            BackdriveResult {
                copies,
                clock: current_clock,
            }
        }
        BackdriveMode::UniformClock => {
            // In uniform clock mode, we will set the clock speed as high as possible up to the
            // limit. We do this by solving twice, using different constants. First, we treat the
            // clock speed as constant equal to the max clock speed and solve for the number of
            // integer copies (by rounding up).
            //
            // overall_multiplier = copies * clock_speed;
            let copies = (overall_multiplier / settings.uniform_max_clock).ceil();

            // Then we can solve for the clock speed by treating the integer number of copies as
            // constant and dividing the other way.
            let clock = overall_multiplier / copies;
            BackdriveResult { copies, clock }
        }
    })
}

/// Calculate the new clock speed and virtual copies for a generator, based on requested power
/// production.
///
/// *   `current_clock`: the current clock speed, used in
///     [`VariableClock`][BackdriveMode::VariableClock] mode.
/// *   `rate`: the requested power production rate. Must be positive.
/// *   `power`: power production values for this generator.
/// *   `settings`: backdrive settings for this building type.
fn backdrive_power_producer(
    current_clock: f32,
    rate: f32,
    power: &Power,
    settings: &BuildingBackdriveSettings,
) -> Option<BackdriveResult> {
    if power.power == 0.0 {
        warn!("Cannot backdrive power production, because the power production is 0");
        return None;
    }
    if power.power_exponent == 0.0 {
        // If overclocking isn't allowed, then both variable clock and uniform clock end up donig
        // the same thing; we just find the integer multiplier that gets at least that much
        // production.
        let copies = (rate / power.power).ceil();
        return Some(BackdriveResult { copies, clock: 1.0 });
    }
    Some(match settings.mode {
        BackdriveMode::VariableClock => {
            // For variable clock speed, we keep the current clock constant, so we need to solve
            // this equation for whole and fractional copies:
            //
            // rate = whole_copes * power * clock_speed ^ (1/power_exponent)
            //        + power * (fractional_copies * clock_speed) ^ (1/power_exponent)
            //
            // Factor out:
            //
            // rate = power * (whole_copies * clock_speed ^ (1/power_exponent)
            //                 + (fractional_copies * clock_speed) ^ (1/power_exponent))
            //
            // Distribute the exponent, then factor out more:
            //
            // rate = power * clock_speed ^ (1/power_exponent)
            //        * (whole_copies + fractional_copies ^ (1/power_exponent))
            //
            // Divide:
            //
            // rate / (power * clock_speed ^ (1/power_expoent))
            //     = whole_copies + fractional_copies ^ (1/power_exponent)
            //
            // Since we have fractional_copies < 1, we know that
            // fractional_copies ^ (1/power_exponent) < 1.
            // This means that if we solve for:
            //
            // combined_multiplier = whole_copies + partial_copies
            // where
            //   whole_copies = combined_multiplier.trunc()
            //   partial_copies = combined_multiplier.fract()
            //
            // Since partial_copies is < 1, we can then just do:
            //
            // fractional_copies = partial_copies ^ power_exponent
            //
            // then add the result back into whole_copies to get our final multiplier, accounting
            // for partial clocks.
            let rate_per_power_clock =
                rate / (power.power * current_clock.powf(1.0 / power.power_exponent));
            let whole_copies = rate_per_power_clock.trunc();
            let fractional_copies = rate_per_power_clock.fract().powf(power.power_exponent);
            BackdriveResult {
                copies: whole_copies + fractional_copies,
                clock: current_clock,
            }
        }
        BackdriveMode::UniformClock => {
            // For uniform clock speed, we first compute an overall multiplier then split it over an
            // integer number of machines based on the clock speed limit. We're trying to solve:
            //
            // rate = copies * power * clock_speed ^ (1/power_exponent)
            //
            // such that clock_speed <= uniform_max_clock. First we'll treat clock_speed as a
            // constant equal to uniform_max_clock and solve for copies to get an overall
            // multiplier. Then we'll take the ceiling of the multiplier to get the number of copies
            // and re-solve for the clock_speed.
            //
            // rate / (power * clock_speed ^ (1/power_exponent)) = copies
            let overall_multiplier =
                rate / (power.power * settings.uniform_max_clock.powf(1.0 / power.power_exponent));
            let copies = overall_multiplier.ceil();

            // rate / (power * copies) = clock_speed ^ (1/power_exponent)
            //
            // (rate / (power * copies)) ^ power_exponent = clock_speed
            let rate_per_machine_power = rate / (power.power * copies);
            let clock = rate_per_machine_power.powf(power.power_exponent);
            BackdriveResult { copies, clock }
        }
    })
}
