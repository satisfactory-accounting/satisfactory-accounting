// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use std::rc::Rc;

use satisfactory_accounting::database::Item;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

use super::NodeDisplay;
use crate::node_display::icon::Icon;
use crate::CtxHelper;

/// How entries in the balance should be sorted.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum BalanceSortMode {
    /// Sort by item, irrespective of whether it's input or output.
    #[default]
    Item,
    /// Sort by whether the item is an input or output (positive or negative balance) then
    /// by item.
    IOItem,
}

impl NodeDisplay {
    /// Build the display for a node's balance.
    pub(super) fn view_balance(&self, ctx: &Context<Self>, vertical: bool) -> Html {
        thread_local! {
            static POWER_LINE: Rc<str> = "power-line".into();
        }

        let balance = ctx.props().node.balance();
        let db = ctx.db();
        let item_balances: Html = match ctx.settings().balance_sort_mode {
            BalanceSortMode::Item => balance
                .balances
                .iter()
                .map(|(&itemid, &rate)| display_item(db.get(itemid), rate))
                .collect(),
            BalanceSortMode::IOItem => balance
                .balances
                .iter()
                .filter(|(_, &rate)| rate > 0.0)
                .chain(balance.balances.iter().filter(|(_, &rate)| rate == 0.0))
                .chain(balance.balances.iter().filter(|(_, &rate)| rate < 0.0))
                // Weird NaN handling? I guess I could probably just use is_nan here?
                .chain(
                    balance
                        .balances
                        .iter()
                        .filter(|(_, &rate)| !(rate < 0.0) && !(rate == 0.0) && !(rate > 0.0)),
                )
                .map(|(&itemid, &rate)| display_item(db.get(itemid), rate))
                .collect(),
        };
        html! {
            <div class={classes!("balance", balance_block_style(vertical))} title="Power">
                <div class={classes!("entry-row", "power-entry", balance_style(balance.power))}>
                    <Icon icon={POWER_LINE.with(Clone::clone)}/>
                    <div class="balance-value">{rounded(balance.power)}</div>
                </div>
                <div class="item-entries">
                { item_balances }
                </div>
            </div>
        }
    }
}

fn display_item(item: Option<&Item>, rate: f32) -> Html {
    match item {
        Some(item) => html! {
            <div class={classes!("entry-row", balance_style(rate))}
                title={Some(item.name.clone())}>
                <Icon icon={item.image.clone()}/>
                <div class="balance-value">{rounded(rate)}</div>
            </div>
        },
        None => html! {
            <div class={classes!("entry-row", balance_style(rate))}
                title="Unknown Item">
                <Icon />
                <div class="balance-value">{rounded(rate)}</div>
            </div>
        },
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
