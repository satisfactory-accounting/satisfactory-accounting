use satisfactory_accounting::accounting::{Building, BuildingSettings, ManufacturerSettings};
use satisfactory_accounting::database::{BuildingId, Database};
use yew::prelude::*;

use super::NodeDisplay;
use crate::node_display::Msg;
use crate::GetDb;

use building_type::BuildingTypeDisplay;
use recipe::RecipeDisplay;

mod building_type;
mod choose_from_list;
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
                    {self.view_balance(ctx)}
                    {self.delete_button(ctx)}
                    {self.copy_button(ctx)}
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
        let change_recipe = ctx.link().callback(|id| Msg::ChangeRecipe { id });
        html! {
            <RecipeDisplay building_id={building} recipe_id={settings.recipe}
                {change_recipe} />
        }
    }
}
