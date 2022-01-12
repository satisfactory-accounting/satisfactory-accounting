// Copyright 2021, 2022 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use satisfactory_accounting::accounting::{
    BuildError, Building, BuildingSettings, GeneratorSettings, GeothermalSettings,
    ManufacturerSettings, MinerSettings, PumpSettings, ResourcePurity, StationSettings,
};
use satisfactory_accounting::database::BuildingId;
use yew::prelude::*;

use crate::node_display::copies::VirtualCopies;
use crate::node_display::{Msg, NodeDisplay};

use building_type::BuildingTypeDisplay;
use clock::ClockSpeed;
use item::ItemDisplay;
use multi_purity::MultiPurity;
use purity::Purity;
use recipe::RecipeDisplay;
use station_consumption::StationConsumption;

mod building_type;
mod choose_from_list;
mod clock;
mod item;
mod multi_purity;
mod purity;
mod recipe;
mod station_consumption;

impl NodeDisplay {
    /// Build display for a building.
    pub(super) fn view_building(&self, ctx: &Context<Self>, building: &Building) -> Html {
        let update_copies = ctx.link().callback(|copies| Msg::SetCopyCount { copies });
        let change_type = ctx.link().callback(|id| Msg::ChangeType { id });
        html! {
            <div class="NodeDisplay building">
                <div class="section">
                    {self.drag_handle(ctx)}
                    <div class="section spaced">
                        <BuildingTypeDisplay id={building.building} {change_type} />
                        {self.view_building_settings(ctx, building)}
                    </div>
                </div>
                <div class="section">
                    if let Some(warning) = ctx.props().node.warning() {
                        {self.view_warning(warning)}
                    } else {
                        {self.view_balance(ctx, false)}
                    }
                    <VirtualCopies copies={building.copies} {update_copies} />
                    {self.copy_button(ctx)}
                    {self.delete_button(ctx)}
                </div>
            </div>
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
                    self.view_manufacturer_settings(ctx, id, settings)
                }
                BuildingSettings::Miner(settings) => self.view_miner_settings(ctx, id, settings),
                BuildingSettings::Generator(settings) => {
                    self.view_generator_settings(ctx, id, settings)
                }
                BuildingSettings::Pump(settings) => self.view_pump_settings(ctx, id, settings),
                BuildingSettings::Geothermal(settings) => {
                    self.view_geothermal_settings(ctx, settings)
                }
                BuildingSettings::PowerConsumer => html! {},
                BuildingSettings::Station(settings) => {
                    self.view_station_settings(ctx, id, settings)
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
        settings: &ManufacturerSettings,
    ) -> Html {
        let link = ctx.link();
        let change_recipe = link.callback(|id| Msg::ChangeRecipe { id });
        let update_speed = link.callback(|clock_speed| Msg::ChangeClockSpeed { clock_speed });
        html! {
            <>
                <RecipeDisplay building_id={building} recipe_id={settings.recipe}
                    {change_recipe} />
                <ClockSpeed clock_speed={settings.clock_speed} {update_speed} />
            </>
        }
    }

    /// Display the settings for a miner.
    fn view_miner_settings(
        &self,
        ctx: &Context<Self>,
        building: BuildingId,
        settings: &MinerSettings,
    ) -> Html {
        let link = ctx.link();
        let change_item = link.callback(|id| Msg::ChangeItem { id });
        let update_speed = link.callback(|clock_speed| Msg::ChangeClockSpeed { clock_speed });
        let set_purity = link.callback(|purity| Msg::ChangePurity { purity });
        html! {
            <>
                <ItemDisplay building_id={building} item_id={settings.resource}
                    {change_item} />
                <ClockSpeed clock_speed={settings.clock_speed} {update_speed} />
                <Purity purity={settings.purity} {set_purity} />
            </>
        }
    }

    /// Display the settings for a generator.
    fn view_generator_settings(
        &self,
        ctx: &Context<Self>,
        building: BuildingId,
        settings: &GeneratorSettings,
    ) -> Html {
        let link = ctx.link();
        let change_item = link.callback(|id| Msg::ChangeItem { id });
        let update_speed = link.callback(|clock_speed| Msg::ChangeClockSpeed { clock_speed });
        html! {
            <>
                <ItemDisplay building_id={building} item_id={settings.fuel}
                    {change_item} />
                <ClockSpeed clock_speed={settings.clock_speed} {update_speed} />
            </>
        }
    }

    /// Display the settings for a pump.
    fn view_pump_settings(
        &self,
        ctx: &Context<Self>,
        building: BuildingId,
        settings: &PumpSettings,
    ) -> Html {
        let link = ctx.link();
        let change_item = link.callback(|id| Msg::ChangeItem { id });
        let update_speed = link.callback(|clock_speed| Msg::ChangeClockSpeed { clock_speed });
        let update_pads =
            link.callback(|(purity, num_pads)| Msg::ChangePumpPurity { purity, num_pads });
        html! {
            <>
                <ItemDisplay building_id={building} item_id={settings.resource}
                    {change_item} />
                <ClockSpeed clock_speed={settings.clock_speed} {update_speed} />
                <MultiPurity purity={ResourcePurity::Impure}
                    num_pads={settings.impure_pads} update_pads={update_pads.clone()} />
                <MultiPurity purity={ResourcePurity::Normal}
                    num_pads={settings.normal_pads} update_pads={update_pads.clone()} />
                <MultiPurity purity={ResourcePurity::Pure}
                    num_pads={settings.pure_pads} {update_pads} />
            </>
        }
    }

    /// Display the settings for a geothermal plant.
    fn view_geothermal_settings(&self, ctx: &Context<Self>, settings: &GeothermalSettings) -> Html {
        let link = ctx.link();
        let set_purity = link.callback(|purity| Msg::ChangePurity { purity });
        html! {
            <Purity purity={settings.purity} {set_purity} />
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
        let change_item = link.callback(|id| Msg::ChangeItem { id });
        let update_consumption =
            link.callback(|consumption| Msg::ChangeConsumption { consumption });
        html! {
            <>
                <ItemDisplay building_id={building} item_id={settings.fuel}
                    {change_item} />
                <StationConsumption consumption={settings.consumption} {update_consumption} />
            </>
        }
    }
}
