use satisfactory_accounting::accounting::{Building, BuildingSettings, ManufacturerSettings};
use satisfactory_accounting::database::{BuildingId, Database};
use yew::prelude::*;

use super::{icon_missing, slug_to_icon, NodeDisplay};
use crate::node_display::NodeMsg;
use crate::GetDb;

use choose_from_list::{Choice, ChooseFromList};

mod choose_from_list;

impl NodeDisplay {
    /// Build display for a building.
    pub(super) fn view_building(&self, ctx: &Context<Self>, building: &Building) -> Html {
        let change_type = ctx.link().callback(|id| NodeMsg::ChangeType { id });
        html! {
            <div class="NodeDisplay building">
                <div class="section">
                    {self.drag_handle(ctx)}
                    <BuildingTypeDisplay id={building.building} {change_type} />
                </div>
                {self.view_building_settings(ctx, building)}
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
        let db = ctx.db();
        if let Some(id) = building.building {
            match &building.settings {
                BuildingSettings::Manufacturer(settings) => {
                    self.view_manufacturer_settings(ctx, &db, id, settings)
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
        db: &Database,
        building: BuildingId,
        settings: &ManufacturerSettings,
    ) -> Html {
        html! {}
    }
}

#[derive(PartialEq, Properties)]
struct BuildingTypeDisplayProps {
    /// ID of the selected building, if any.
    id: Option<BuildingId>,
    /// Callback to change the type of this building.
    change_type: Callback<BuildingId>,
}

/// Messages for [`BuildingTypeDisplay`]
enum BuildingTypeMsg {
    /// Switches in or out of editing.
    ToggleEdit {
        /// The new editing state.
        editing: bool,
    },
    /// Select a new building ID.
    Select {
        /// The new ID.
        id: BuildingId,
    },
}

/// Displays and allows selection of the Building's Type (BuildingId).
#[derive(Default)]
struct BuildingTypeDisplay {
    /// Whether a building is currently being entered.
    editing: bool,
}

impl Component for BuildingTypeDisplay {
    type Message = BuildingTypeMsg;
    type Properties = BuildingTypeDisplayProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Default::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            BuildingTypeMsg::ToggleEdit { editing } => {
                self.editing = editing;
                true
            }
            BuildingTypeMsg::Select { id } => {
                ctx.props().change_type.emit(id);
                self.editing = false;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let db = ctx.db();
        let link = ctx.link();
        if self.editing {
            let choices: Vec<_> = db
                .buildings
                .values()
                .map(|building| Choice {
                    id: building.id,
                    name: building.name.clone(),
                    image: slug_to_icon(&building.image),
                })
                .collect();

            let selected = link.callback(|id| BuildingTypeMsg::Select { id });
            let cancelled = link.callback(|()| BuildingTypeMsg::ToggleEdit { editing: false });
            html! {
                <span class="name">
                    <ChooseFromList<BuildingId> {choices} {selected} {cancelled} />
                </span>
            }
        } else {
            let edit = link.callback(|_| BuildingTypeMsg::ToggleEdit { editing: true });
            match ctx.props().id {
                None => html! {
                    <span class="name" onclick={edit}>{"select building"}</span>
                },
                Some(id) => match db.get(id) {
                    None => html! {
                        <span class="name" onclick={edit}>
                            {icon_missing()}
                            <span>{"Unknown Building "}{id}</span>
                        </span>
                    },
                    Some(building) => html! {
                        <span class="name" onclick={edit}>
                            <img class="icon"
                                src={slug_to_icon(&building.image)}
                                alt={building.name.clone()} />
                            <span>{&building.name}</span>
                        </span>
                    },
                },
            }
        }
    }
}
