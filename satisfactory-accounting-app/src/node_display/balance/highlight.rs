use log::info;
use satisfactory_accounting::database::ItemIdOrPower;
use yew::html::Scope;
use yew::{hook, html, use_context, Component, Context, ContextProvider, Html, Properties};

use crate::refeqrc::RefEqRc;

type Link = RefEqRc<Scope<HighlightItemManager>>;

#[derive(Clone, PartialEq)]
pub struct HighlightedItemController {
    item: Option<ItemIdOrPower>,
    link: Link,
}

/// Get the highlighted item controller
#[hook]
pub fn use_highlighted_item() -> HighlightedItemController {
    use_context::<HighlightedItemController>()
        .expect("use_highlighte_item can only be used from within a child of HighlightItemManager.")
}

impl HighlightedItemController {
    /// Gets the currently highlighted item.
    pub fn item(&self) -> Option<ItemIdOrPower> {
        self.item
    }

    /// Tell the HighlightManager that the mouse has entered the given item.
    pub fn enter_item(&self, item: ItemIdOrPower) {
        self.link.send_message(Msg::EnterItem(item));
    }

    /// Tell the HighlightManager that the mouse has exited the given item.
    pub fn exit_item(&self, item: ItemIdOrPower) {
        self.link.send_message(Msg::ExitItem(item));
    }
}

pub enum Msg {
    EnterItem(ItemIdOrPower),
    ExitItem(ItemIdOrPower),
}

#[derive(PartialEq, Properties)]
pub struct Props {
    /// Children, which will have access to the world and the world manager's various context
    /// handles.
    pub children: Html,
}

/// Controls what item is currenctly hovered/highlighted.
pub struct HighlightItemManager {
    highlighted_item: Option<ItemIdOrPower>,
    link: Link,
}

impl Component for HighlightItemManager {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        info!("Creating HighlightItemManager");
        Self {
            highlighted_item: None,
            link: RefEqRc::new(ctx.link().clone()),
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            // When entering, only mark a change when the highlighted item is actually different.
            Msg::EnterItem(item) => {
                if self.highlighted_item == Some(item) {
                    false
                } else {
                    self.highlighted_item = Some(item);
                    true
                }
            }
            // When exiting, only exit if the previously hovered item is still the one highlighted.
            Msg::ExitItem(item) => {
                if self.highlighted_item == Some(item) {
                    self.highlighted_item = None;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let controller = HighlightedItemController {
            item: self.highlighted_item,
            link: self.link.clone(),
        };
        html! {
            <ContextProvider<HighlightedItemController> context={controller}>
                {ctx.props().children.clone()}
            </ContextProvider<HighlightedItemController>>
        }
    }
}
