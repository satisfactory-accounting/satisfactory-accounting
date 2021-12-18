use yew::prelude::*;

use super::{icon_missing, slug_to_icon, NodeDisplay};
use crate::GetDb;

impl NodeDisplay {
    /// Build the display for a node's balance.
    pub(super) fn view_balance(&self, ctx: &Context<Self>) -> Html {
        let balance = ctx.props().node.balance();
        let db = ctx.db();
        html! {
            <div class="balance" title="Power">
                <div class="entry-row">
                    <img class="icon" alt="Power"
                        src={slug_to_icon("power-line")} />
                    <div class={classes!("balance-value", balance_style(balance.power))}>
                        {balance.power}
                    </div>
                </div>
                { for balance.balances.iter().map(|(&itemid, &rate)| match db.get(itemid) {
                    Some(item) => html! {
                        <div class="entry-row" title={item.name.clone()}>
                            <img class="icon" alt={item.name.clone()}
                                src={slug_to_icon(&item.image)} />
                            <div class={classes!("balance-value", balance_style(rate))}>
                                {rate}
                            </div>
                        </div>
                    },
                    None => html! {
                        <div class="entry-row" title="Unknown Item">
                            {icon_missing()}
                            <div class={classes!("balance-value", balance_style(rate))}>
                                {rate}
                            </div>
                        </div>
                    }
                }) }
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
