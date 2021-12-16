use satisfactory_accounting::accounting::Building;
use satisfactory_accounting::database::{BuildingId, Database};
use yew::prelude::*;

use crate::GetDb;

use super::{icon_missing, slug_to_icon, NodeDisplay};

impl NodeDisplay {
    /// Build display for a building.
    pub(super) fn view_building(&self, ctx: &Context<Self>, building: &Building) -> Html {
        html! {
            <div class="NodeDisplay building">
                {self.drag_handle(ctx)}
                <BuildingTypeDisplay id={building.building} />
                <div class="space" />
                {self.view_balance(ctx)}
                {self.delete_button(ctx)}
            </div>
        }
    }
}

#[derive(PartialEq, Properties)]
struct BuildingTypeDisplayProps {
    /// ID of the selected building, if any.
    id: Option<BuildingId>,
}

/// Displays and allows selection of the Building's Type (BuildingId).
#[derive(Default)]
struct BuildingTypeDisplay {}

impl Component for BuildingTypeDisplay {
    type Message = ();
    type Properties = BuildingTypeDisplayProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Default::default()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let db = ctx.db();
        match ctx.props().id {
            None => html! { <span>{"TODO: input building"}</span> },
            Some(id) => match db.get(id) {
                None => html! {
                    <>
                        {icon_missing()}
                        <span>{"Unknown Building "}{id}</span>
                    </>
                },
                Some(building) => html! {
                    <>
                        <img src={slug_to_icon(&building.image)}
                            alt={building.name.clone()} />
                        <span>{&building.name}</span>
                    </>
                },
            },
        }
    }
}
