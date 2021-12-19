use std::rc::Rc;

use yew::prelude::*;

use super::NodeDisplay;
use crate::node_display::icon::Icon;
use crate::GetDb;

impl NodeDisplay {
    /// Build the display for a node's balance.
    pub(super) fn view_balance(&self, ctx: &Context<Self>) -> Html {
        thread_local! {
            static POWER_LINE: Rc<str> = "power-line".into();
            static POWER: Rc<str> = "Power".into();
        }

        let balance = ctx.props().node.balance();
        let db = ctx.db();
        html! {
            <div class="balance" title="Power">
                <div class="entry-row">
                    <Icon icon={POWER_LINE.with(Clone::clone)}
                        alt={POWER.with(Clone::clone)} />
                    <div class={classes!("balance-value", balance_style(balance.power))}>
                        {balance.power}
                    </div>
                </div>
                { for balance.balances.iter().map(|(&itemid, &rate)| match db.get(itemid) {
                    Some(item) => html! {
                        <div class="entry-row" title={Some(item.name.clone())}>
                            <Icon icon={item.image.clone()}
                                alt={item.name.clone()} />
                            <div class={classes!("balance-value", balance_style(rate))}>
                                {rate}
                            </div>
                        </div>
                    },
                    None => html! {
                        <div class="entry-row" title="Unknown Item">
                            <Icon />
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
