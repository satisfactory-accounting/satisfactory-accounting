use std::rc::Rc;

use satisfactory_accounting::database::Database;
use yew::prelude::*;

use super::{slug_to_icon, NodeDisplay};

impl NodeDisplay {
    /// Build the display for a node's balance.
    pub fn view_balance(&self, ctx: &Context<Self>) -> Html {
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
