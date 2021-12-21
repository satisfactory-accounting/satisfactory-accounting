use satisfactory_accounting::accounting::{
    Building, BuildingSettings, GeneratorSettings, ManufacturerSettings, MinerSettings,
};
use satisfactory_accounting::database::BuildingId;
use yew::prelude::*;

use crate::node_display::{Msg, NodeDisplay};

use building_type::BuildingTypeDisplay;
use clock::ClockSpeed;
use item::ItemDisplay;
use recipe::RecipeDisplay;

mod building_type;
mod choose_from_list;
mod clock;
mod item;
mod recipe;

impl NodeDisplay {
    /// Build display for a building.
    pub(super) fn view_building(&self, ctx: &Context<Self>, building: &Building) -> Html {
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
                    {self.view_balance(ctx, false)}
                    {self.copy_button(ctx)}
                    {self.delete_button(ctx)}
                </div>
            </div>
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
                _ => html! {},
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

    /// Display the settings for a manufacturer.
    fn view_miner_settings(
        &self,
        ctx: &Context<Self>,
        building: BuildingId,
        settings: &MinerSettings,
    ) -> Html {
        let link = ctx.link();
        let change_item = link.callback(|id| Msg::ChangeItem { id });
        let update_speed = link.callback(|clock_speed| Msg::ChangeClockSpeed { clock_speed });
        html! {
            <>
                <ItemDisplay building_id={building} item_id={settings.resource}
                    {change_item} />
                <ClockSpeed clock_speed={settings.clock_speed} {update_speed} />
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
}
