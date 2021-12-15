use std::ops::Deref;
use std::rc::Rc;

use yew::prelude::*;

use satisfactory_accounting::accounting::{Node, Group, NodeKind};
use satisfactory_accounting::database::Database;

/// Path to a node in the tree.
#[derive(Clone, PartialEq, Debug)]
pub struct NodePath(Rc<Vec<usize>>);

#[derive(Debug, PartialEq, Properties)]
pub struct NodeDisplayProperties {
    /// The node to display.
    pub node: Node,
    /// Path to this node in the tree.
    pub path: Vec<usize>,
}

/// Display for a single AccountingGraph node.
pub struct NodeDisplay {}

impl Component for NodeDisplay {
    type Message = ();
    type Properties = NodeDisplayProperties;

    fn create(_: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="NodeDisplay">
            {match ctx.props().node.kind() {
                NodeKind::Group(group) => self.view_group(ctx, group),
                NodeKind::Building(building) => {
                    html! {}
                }
            }}
            </div>
        }
    }
}

impl NodeDisplay {
    /// Build the display for a Group.
    fn view_group(&self, ctx: &Context<Self>, group: &Group) -> Html {
        html! {
            <div class="Group">
                <div class="header">
                    if let Some(name) = &group.name {
                        <span class="name">{name}</span>
                    } else {
                        <span class="name notset">{"unnamed"}</span>
                    }
                    <div class="space" />
                    <button class="edit">{"edit"}</button>
                </div>
                <div class="body">
                    <div class="children-display">
                        { for group.children.iter().cloned().enumerate().map(|(i, node)| {
                            let mut path = ctx.props().path.clone();
                            path.push(i);
                            html! {<NodeDisplay {node} {path} />}
                        }) }
                    </div>
                    <div class="space" />
                    <div>{self.view_balance(ctx)}</div>
                </div>
                <div class="footer">
                    <button class="create group">{"G"}</button>
                    <button class="create building">{"+"}</button>
                </div>
            </div>
        }
    }

    /// Build the display for a node's balance.
    fn view_balance(&self, ctx: &Context<Self>) -> Html {
        let balance = ctx.props().node.balance();
        let (db, _) = ctx
            .link()
            .context::<Database>(Callback::noop())
            .expect("context to be set");
        html! {
            <div class="Balance">
                <div class="entry-row">
                    <img class="icon" alt="power" src={slug_to_icon("power-line")} />
                    <div class={classes!("balance", balance_style(balance.power))}>
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
                                <div class={classes!("balance", balance_style(rate))}>
                                    {rate}
                                </div>
                            </div>
                        }
                    }) }
                </div>
            </div>
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
