use log::warn;
// Copyright 2021, 2022 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use satisfactory_accounting::accounting::{
    BalanceAdjustmentSettings, BuildError, Building, BuildingSettings, GeneratorSettings,
    GeothermalSettings, ManufacturerSettings, MinerSettings, PumpSettings, ResourcePurity,
    StationSettings,
};
use satisfactory_accounting::database::{BuildingId, BuildingKind};
use yew::prelude::*;

use crate::node_display::balance::NodeBalance;
use crate::node_display::clock::ClockSpeed;
use crate::node_display::copies::VirtualCopies;
use crate::node_display::{Msg, NodeDisplay};

use building_type::BuildingTypeDisplay;
use item::ItemDisplay;
use item_or_power::ItemOrPowerDisplay;
use item_rate::ItemRate;
use multi_purity::MultiPurity;
use purity::Purity;
use recipe::RecipeDisplay;

mod building_type;
mod item;
mod item_or_power;
mod item_rate;
mod multi_purity;
mod purity;
mod recipe;

impl NodeDisplay {
    /// Build display for a building.
    pub(super) fn view_building(&self, ctx: &Context<Self>, building: &Building) -> Html {
        let update_copies = ctx.link().callback(|copies| Msg::SetCopyCount { copies });
        let on_change_type = ctx.link().callback(|id| Msg::ChangeType { id });
        let on_backdrive = self.supports_backdrive(building).then(|| {
            ctx.link()
                .callback(|(id, rate)| Msg::Backdrive { id, rate })
        });
        html! {
            <div class="NodeDisplay building">
                {self.drag_handle(ctx)}
                <BuildingTypeDisplay id={building.building} {on_change_type} />
                {self.view_building_settings(ctx, building)}
                if ctx.props().node.warning().is_none() {
                    <NodeBalance node={&ctx.props().node} {on_backdrive} />
                }
                <VirtualCopies copies={building.copies} {update_copies} />
                <div class="section copy-delete">
                    if let Some(warning) = ctx.props().node.warning() {
                        {self.view_warning(warning)}
                    }
                    {self.copy_button(ctx)}
                    {self.delete_button(ctx)}
                </div>
            </div>
        }
    }

    /// Whether a building supports backdriving.
    fn supports_backdrive(&self, building: &Building) -> bool {
        let building_id = match building.building {
            Some(id) => id,
            None => return false,
        };
        let building = match self.db.get(building_id) {
            Some(bldg) => bldg,
            None => return false,
        };
        match building.kind {
            BuildingKind::Manufacturer(_) => true,
            BuildingKind::Miner(_) => true,
            BuildingKind::Generator(_) => true,
            BuildingKind::Pump(_) => true,
            BuildingKind::Geothermal(ref g) => g.power > 0.0,
            BuildingKind::PowerConsumer(ref p) => p.power > 0.0,
            BuildingKind::Station(_) => false,
            BuildingKind::BalanceAdjustment(_) => false,
        }
    }

    fn view_warning(&self, err: BuildError) -> Html {
        // TODO: give better error messages.
        html! {
            <span class="BuildError material-icons error" title={err.to_string()}>
                {"warning"}
            </span>
        }
    }

    /// If a building is selected, display its settings.
    fn view_building_settings(&self, ctx: &Context<Self>, building: &Building) -> Html {
        if let Some(id) = building.building {
            match &building.settings {
                BuildingSettings::Manufacturer(settings) => {
                    self.view_manufacturer_settings(ctx, id, building.copies, settings)
                }
                BuildingSettings::Miner(settings) => {
                    self.view_miner_settings(ctx, id, building.copies, settings)
                }
                BuildingSettings::Generator(settings) => {
                    self.view_generator_settings(ctx, id, building.copies, settings)
                }
                BuildingSettings::Pump(settings) => {
                    self.view_pump_settings(ctx, id, building.copies, settings)
                }
                BuildingSettings::Geothermal(settings) => {
                    self.view_geothermal_settings(ctx, settings)
                }
                BuildingSettings::PowerConsumer => html! {},
                BuildingSettings::Station(settings) => {
                    self.view_station_settings(ctx, id, settings)
                }
                BuildingSettings::BalanceAdjustment(settings) => {
                    self.view_balance_adjustment_settings(ctx, settings)
                }
            }
        } else {
            html! {}
        }
    }

    /// Display the settings for a manufacturer.
    fn view_manufacturer_settings(
        &self,
        ctx: &Context<Self>,
        building: BuildingId,
        copies: f32,
        settings: &ManufacturerSettings,
    ) -> Html {
        let link = ctx.link();
        let on_change_recipe = link.callback(|id| Msg::ChangeRecipe { id });

        html! {
            <>
                <RecipeDisplay building_id={building} recipe_id={settings.recipe}
                    {on_change_recipe} />
                { self.view_clock_controls_if_overclockable(ctx, building, copies, settings.clock_speed) }
            </>
        }
    }

    /// Display the settings for a miner.
    fn view_miner_settings(
        &self,
        ctx: &Context<Self>,
        building: BuildingId,
        copies: f32,
        settings: &MinerSettings,
    ) -> Html {
        let link = ctx.link();
        let on_change_item = link.callback(|id| Msg::ChangeItem { id });
        let on_set_purity = link.callback(|purity| Msg::ChangePurity { purity });
        html! {
            <>
                <ItemDisplay building_id={building} item_id={settings.resource}
                    {on_change_item} />
                { self.view_clock_controls_if_overclockable(ctx, building, copies, settings.clock_speed) }
                <Purity purity={settings.purity} {on_set_purity} />
            </>
        }
    }

    /// Display the settings for a generator.
    fn view_generator_settings(
        &self,
        ctx: &Context<Self>,
        building: BuildingId,
        copies: f32,
        settings: &GeneratorSettings,
    ) -> Html {
        let on_change_item = ctx.link().callback(|id| Msg::ChangeItem { id });
        html! {
            <>
                <ItemDisplay building_id={building} item_id={settings.fuel}
                    {on_change_item} />
                { self.view_clock_controls_if_overclockable(ctx, building, copies, settings.clock_speed) }
            </>
        }
    }

    /// Display the settings for a pump.
    fn view_pump_settings(
        &self,
        ctx: &Context<Self>,
        building: BuildingId,
        copies: f32,
        settings: &PumpSettings,
    ) -> Html {
        let link = ctx.link();
        let on_change_item = link.callback(|id| Msg::ChangeItem { id });
        let on_update_pads =
            link.callback(|(purity, num_pads)| Msg::ChangePumpPurity { purity, num_pads });
        html! {
            <>
                <ItemDisplay building_id={building} item_id={settings.resource}
                    {on_change_item} />
                { self.view_clock_controls_if_overclockable(ctx, building, copies, settings.clock_speed) }
                <div class="section multi-purity-group">
                    <MultiPurity purity={ResourcePurity::Impure}
                        num_pads={settings.impure_pads} on_update_pads={&on_update_pads} />
                    <MultiPurity purity={ResourcePurity::Normal}
                        num_pads={settings.normal_pads} on_update_pads={&on_update_pads} />
                    <MultiPurity purity={ResourcePurity::Pure}
                        num_pads={settings.pure_pads} {on_update_pads} />
                </div>
            </>
        }
    }

    /// Display the settings for a geothermal plant.
    fn view_geothermal_settings(&self, ctx: &Context<Self>, settings: &GeothermalSettings) -> Html {
        let link = ctx.link();
        let on_set_purity = link.callback(|purity| Msg::ChangePurity { purity });
        html! {
            <Purity purity={settings.purity} {on_set_purity} />
        }
    }

    /// Display the settings for a station.
    fn view_station_settings(
        &self,
        ctx: &Context<Self>,
        building: BuildingId,
        settings: &StationSettings,
    ) -> Html {
        let link = ctx.link();
        let on_change_item = link.callback(|id| Msg::ChangeItem { id });
        let update_rate = link.callback(|rate| Msg::ChangeRate { rate });
        html! {
            <>
                <ItemDisplay building_id={building} item_id={settings.fuel}
                    {on_change_item} />
                <ItemRate rate={settings.consumption} {update_rate}
                    title="Fuel Consumption of Fueled Vehicles" />
            </>
        }
    }

    /// Display settings for a balance adjustment.
    fn view_balance_adjustment_settings(
        &self,
        ctx: &Context<Self>,
        settings: &BalanceAdjustmentSettings,
    ) -> Html {
        let link = ctx.link();
        let on_change_item = link.callback(|id| Msg::ChangeItemOrPower { id });
        let update_rate = link.callback(|rate| Msg::ChangeRate { rate });

        html! {
            <>
            </>
        }
    }

    /// If the building can be overclocked, returns the clock controls, otherwise returns None.
    fn view_clock_controls_if_overclockable(
        &self,
        ctx: &Context<Self>,
        building: BuildingId,
        copies: f32,
        clock_speed: f32,
    ) -> Option<Html> {
        match self.db.get(building) {
            Some(building) if !building.overclockable() => None,
            // Treat missing buildings as overclockable by default; only hide the clock control if
            // explicitly not overclockable.
            maybe_building => {
                if maybe_building.is_none() {
                    warn!("Showing clock controls by default for unknown building {building}");
                }
                let on_update_speed = ctx
                    .link()
                    .callback(|clock_speed| Msg::ChangeClockSpeed { clock_speed });
                Some(html! { <ClockSpeed {clock_speed} {copies} {on_update_speed} /> })
            }
        }
    }
}
