use std::rc::Rc;

use yew::prelude::*;

use super::NodeDisplay;
use crate::node_display::icon::Icon;
use crate::CtxHelper;

impl NodeDisplay {
    /// Build the display for a node's balance.
    pub(super) fn view_balance(&self, ctx: &Context<Self>, vertical: bool) -> Html {
        thread_local! {
            static POWER_LINE: Rc<str> = "power-line".into();
            static POWER: Rc<str> = "Power".into();
        }

        let balance = ctx.props().node.balance();
        let db = ctx.db();
        html! {
            <div class={classes!("balance", balance_block_style(vertical))} title="Power">
                <div class={classes!("entry-row", "power-entry", balance_style(balance.power))}>
                    <Icon icon={POWER_LINE.with(Clone::clone)}
                        alt={POWER.with(Clone::clone)} />
                    <div class="balance-value">{rounded(balance.power)}</div>
                </div>
                { for balance.balances.iter().map(|(&itemid, &rate)| match db.get(itemid) {
                    Some(item) => html! {
                        <div class={classes!("entry-row", balance_style(rate))}
                            title={Some(item.name.clone())}>
                            <Icon icon={item.image.clone()}
                                alt={item.name.clone()} />
                            <div class="balance-value">{rounded(rate)}</div>
                        </div>
                    },
                    None => html! {
                        <div class={classes!("entry-row", balance_style(rate))}
                            title="Unknown Item">
                            <Icon />
                            <div class="balance-value">{rounded(rate)}</div>
                        </div>
                    }
                }) }
            </div>
        }
    }
}

fn rounded(val: f32) -> f32 {
    (val * 100.0).round() / 100.0
}

fn balance_block_style(vertical: bool) -> &'static str {
    if vertical {
        "vertical"
    } else {
        "horizontal"
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
