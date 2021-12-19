use satisfactory_accounting::database::BuildingId;
use yew::prelude::*;

use crate::GetDb;
use crate::node_display::icon::Icon;
use crate::node_display::building::choose_from_list::{Choice, ChooseFromList};

#[derive(PartialEq, Properties)]
pub struct Props {
    /// ID of the selected building, if any.
    pub id: Option<BuildingId>,
    /// Callback to change the type of this building.
    pub change_type: Callback<BuildingId>,
}

/// Messages for [`BuildingTypeDisplay`]
pub enum Msg {
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
pub struct BuildingTypeDisplay {
    /// Whether a building is currently being entered.
    editing: bool,
}

impl Component for BuildingTypeDisplay {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Default::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleEdit { editing } => {
                self.editing = editing;
                true
            }
            Msg::Select { id } => {
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
                    image: Some(building.image.clone()),
                })
                .collect();

            let selected = link.callback(|id| Msg::Select { id });
            let cancelled = link.callback(|()| Msg::ToggleEdit { editing: false });
            html! {
                <span class="name">
                    <ChooseFromList<BuildingId> {choices} {selected} {cancelled} />
                </span>
            }
        } else {
            let edit = link.callback(|_| Msg::ToggleEdit { editing: true });
            match ctx.props().id {
                None => html! {
                    <span class="name" onclick={edit}>{"select building"}</span>
                },
                Some(id) => match db.get(id) {
                    None => html! {
                        <span class="name" onclick={edit}>
                            <Icon />
                            <span>{"Unknown Building "}{id}</span>
                        </span>
                    },
                    Some(building) => html! {
                        <span class="name" onclick={edit}>
                            <Icon icon={building.image.clone()}
                                alt={building.name.clone()} />
                            <span>{&building.name}</span>
                        </span>
                    },
                },
            }
        }
    }
}
