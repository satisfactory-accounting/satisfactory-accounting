use std::rc::Rc;

use log::warn;
use yew::prelude::*;

use satisfactory_accounting::accounting::{Building, Group, Node, NodeKind};
use satisfactory_accounting::database::Database;

#[derive(Debug, PartialEq, Properties)]
pub struct NodeDisplayProperties {
    /// The node to display.
    pub node: Node,
    /// Path to this node in the tree.
    pub path: Vec<usize>,
    /// Callback to tell the parent to delete this node.
    #[prop_or_default]
    pub delete: Option<Callback<usize>>,
    /// Callback to tell the parent to replace this node.
    pub replace: Callback<(usize, Node)>,
}

/// Messages which can be sent to a Node.
pub enum NodeMsg {
    ReplaceChild { idx: usize, replacement: Node },
    DeleteChild { idx: usize },
    AddChild { child: Node },
}

/// Display for a single AccountingGraph node.
pub struct NodeDisplay {}

impl Component for NodeDisplay {
    type Message = NodeMsg;
    type Properties = NodeDisplayProperties;

    fn create(_: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let our_idx = ctx.props().path.last().copied().unwrap_or_default();
        match msg {
            NodeMsg::ReplaceChild { idx, replacement } => {
                if let NodeKind::Group(group) = ctx.props().node.kind() {
                    if idx < group.children.len() {
                        let mut new_group = group.clone();
                        new_group.children[idx] = replacement;
                        ctx.props().replace.emit((our_idx, new_group.into()));
                    } else {
                        warn!(
                            "Cannot replace child index {}; out of range for this group",
                            idx
                        );
                    }
                } else {
                    warn!("Cannot replace child of a non-group");
                }
                false
            }
            NodeMsg::DeleteChild { idx } => {
                if let NodeKind::Group(group) = ctx.props().node.kind() {
                    if idx < group.children.len() {
                        let mut new_group = group.clone();
                        new_group.children.remove(idx);
                        ctx.props().replace.emit((our_idx, new_group.into()));
                    } else {
                        warn!(
                            "Cannot delete child index {}; out of range for this group",
                            idx
                        );
                    }
                } else {
                    warn!("Cannot delete child of a non-group");
                }
                false
            }
            NodeMsg::AddChild { child } => {
                if let NodeKind::Group(group) = ctx.props().node.kind() {
                    let mut new_group = group.clone();
                    new_group.children.push(child);
                    ctx.props().replace.emit((our_idx, new_group.into()));
                } else {
                    warn!("Cannot add child to a non-group");
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match ctx.props().node.kind() {
            NodeKind::Group(group) => self.view_group(ctx, group),
            NodeKind::Building(building) => {
                html! {}
            }
        }
    }
}

impl NodeDisplay {
    /// Build the display for a Group.
    fn view_group(&self, ctx: &Context<Self>, group: &Group) -> Html {
        let link = ctx.link();
        let replace =
            link.callback(|(idx, replacement)| NodeMsg::ReplaceChild { idx, replacement });
        let delete = link.callback(|idx| NodeMsg::DeleteChild { idx });
        let add_group = link.callback(|_| NodeMsg::AddChild {
            child: Group::empty_node(),
        });
        let add_building = link.callback(|_| NodeMsg::AddChild {
            child: Building::empty_node(),
        });
        html! {
            <div class="NodeDisplay group">
                <div class="header">
                    <GroupName name={group.name.clone()} />
                    {self.delete_button(ctx)}
                </div>
                <div class="body">
                    <div class="children-display">
                        { for group.children.iter().cloned().enumerate().map(|(i, node)| {
                            let mut path = ctx.props().path.clone();
                            path.push(i);
                            html! {
                                <NodeDisplay {node} {path}
                                    replace={replace.clone()}
                                    delete={delete.clone()} />
                            }
                        }) }
                    </div>
                    {self.view_balance(ctx)}
                </div>
                <div class="footer">
                    <button class="create create-group" onclick={add_group}>
                        <span class="material-icons">{"create_new_folder"}</span>
                    </button>
                    <button class="create create-building" onclick={add_building}>
                        <span class="material-icons">{"add"}</span>
                    </button>
                </div>
            </div>
        }
    }

    /// Build the display for a node's balance.
    fn view_balance(&self, ctx: &Context<Self>) -> Html {
        let balance = ctx.props().node.balance();
        let (db, _) = ctx
            .link()
            .context::<Rc<Database>>(Callback::noop())
            .expect("context to be set");
        html! {
            <div class="balance">
                <div class="entry-row">
                    <img class="icon" alt="power" src={slug_to_icon("power-line")} />
                    <div class={classes!("balance-value", balance_style(balance.power))}>
                        {balance.power}
                    </div>
                    { for balance.balances.iter().map(|(&itemid, &rate)| {
                        let (icon, name) = match db.get(itemid) {
                            Some(item) => (slug_to_icon(&item.image), item.name.to_owned()),
                            None => (slug_to_icon("expanded-power-infrastructure"), "unknown".to_owned()),
                        };
                        html! {
                            <div class="entry-row">
                                <img class="icon" alt={name} src={icon} />
                                <div class={classes!("balance-value", balance_style(rate))}>
                                    {rate}
                                </div>
                            </div>
                        }
                    }) }
                </div>
            </div>
        }
    }

    /// Creates the delete button, if the parent allows this node to be deleted.
    fn delete_button(&self, ctx: &Context<Self>) -> Html {
        match ctx.props().delete.clone() {
            Some(delete_from_parent) => {
                let idx = ctx
                    .props()
                    .path
                    .last()
                    .copied()
                    .expect("Parent provided a delete callback, but this is the root node.");
                let onclick = Callback::from(move |_| delete_from_parent.emit(idx));
                html! {
                    <button {onclick} class="delete">
                        <span class="material-icons">{"delete"}</span>
                    </button>
                }
            }
            None => html! {},
        }
    }
}

/// Get the icon path for a given slug name.
fn slug_to_icon(slug: impl AsRef<str>) -> String {
    let mut icon = slug.as_ref().to_owned();
    icon.insert_str(0, "/images/items/");
    icon.push_str("_64.png");
    icon
}

fn balance_style(balance: f32) -> &'static str {
    if balance < 0.0 {
        "negative"
    } else if balance > 0.0 {
        "positive"
    } else {
        "neutral"
    }
}

#[derive(PartialEq, Properties)]
struct GroupNameProps {
    name: Option<String>,
}

struct GroupName {}

impl Component for GroupName {
    type Message = ();
    type Properties = GroupNameProps;

    fn create(_: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let name = &ctx.props().name;
        html! {
            <div class="GroupName">
                if let Some(name) = name {
                    <span class="name">{name}</span>
                } else {
                    <span class="name notset">{"unnamed"}</span>
                }
                <div class="space" />
                <button class="edit">
                    <span class="material-icons">{"edit"}</span>
                </button>
            </div>
        }
    }
}
