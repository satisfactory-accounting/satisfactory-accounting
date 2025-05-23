use log::info;
// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use satisfactory_accounting::accounting::Node;
use satisfactory_accounting::database::{BuildingKindId, Item, ItemId, ItemIdOrPower};
use serde::{Deserialize, Serialize};
use yew::prelude::*;

use crate::inputs::clickedit::{
    AdjustDir, AdjustModifier, AdjustScale, ClickEdit, ValueAdjustment,
};
use crate::node_display::icon::Icon;
use crate::user_settings::number_format::{
    BalanceDisplaySettings, NumberFormatSettings, NumberStylingMode, UserConfiguredFormat,
};
use crate::user_settings::use_user_settings;
use crate::world::use_db;

/// How entries in the balance should be sorted.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum BalanceSortMode {
    /// Sort by item, irrespective of whether it's input or output.
    Item,
    /// Sort by whether the item is an input or output (positive or negative balance) then
    /// by item.
    #[default]
    IOItem,
}

/// Controls how the balance is displayed.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub enum BalanceShape {
    /// Display the items in-line in a row.
    #[default]
    Horizontal,
    /// Display the balance as a separate block with items stacked.
    Vertical,
}

impl BalanceShape {
    /// Get the class name for this balance shape.
    fn to_class_name(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

#[derive(Debug, PartialEq, Properties)]
pub struct Props {
    /// Node to display the balance of.
    pub node: Node,
    /// Whether to use the vertical or horizontal display format.
    #[prop_or_default]
    pub shape: BalanceShape,
    /// Callback to use for backdriving (setting the clock speed based on item count).
    #[prop_or_default]
    pub on_backdrive: Option<Callback<(ItemIdOrPower, f32)>>,
}

#[function_component]
pub fn NodeBalance(
    &Props {
        ref node,
        shape,
        ref on_backdrive,
    }: &Props,
) -> Html {
    let balance = node.balance();
    let db = use_db();
    let user_settings = use_user_settings();
    let balance_settings = &user_settings.number_display.balance;
    let on_backdrive = on_backdrive.as_ref();
    let adjustment_class = node.building().and_then(|b| {
        if b.settings.kind_id() == BuildingKindId::BalanceAdjustment {
            Some("balance-adjustment")
        } else {
            None
        }
    });

    let item_balances: Html = match user_settings.balance_sort_mode {
        BalanceSortMode::Item => {
            let combined_balances = balance.balances.iter().map(|(&itemid, &rate)| {
                display_item(itemid, db.get(itemid), rate, balance_settings, on_backdrive)
            });
            html! {
                <div class="item-entries combined">
                    {for combined_balances}
                </div>
            }
        }
        BalanceSortMode::IOItem => {
            let display_rate = |rate| {
                display_rate(
                    rate,
                    &balance_settings.item_format_settings,
                    balance_settings,
                )
            };
            let positive_balances = balance
                .balances
                .iter()
                .filter(|(_, &rate)| display_rate(rate) > 0.0)
                .map(|(&itemid, &rate)| {
                    display_item(itemid, db.get(itemid), rate, balance_settings, on_backdrive)
                });
            let negative_balances = balance
                .balances
                .iter()
                .filter(|(_, &rate)| display_rate(rate) < 0.0)
                .map(|(&itemid, &rate)| {
                    display_item(itemid, db.get(itemid), rate, balance_settings, on_backdrive)
                });

            let neutral_balances = balance
                .balances
                .iter()
                // Weird NaN handling? I guess I could probably just use is_nan here?
                .filter(|(_, &rate)| {
                    let rate = display_rate(rate);
                    rate == 0.0 || !(rate < 0.0 || rate > 0.0)
                })
                .map(|(&itemid, &rate)| {
                    display_item(itemid, db.get(itemid), rate, balance_settings, on_backdrive)
                });

            html! {
                <>
                <div class="item-entries positive">
                    {for positive_balances}
                </div>
                <div class="item-entries neutral">
                    {for neutral_balances}
                </div>
                <div class="item-entries negative">
                    {for negative_balances}
                </div>
                </>
            }
        }
    };
    html! {
        <div class={classes!("NodeBalance", shape.to_class_name(), adjustment_class)}>
            {item_row(ItemIdOrPower::Power, "Power".into(), Some("power-line".into()), balance.power, balance_settings, on_backdrive)}
            { item_balances }
        </div>
    }
}

fn display_item(
    id: ItemId,
    item: Option<&Item>,
    rate: f32,
    balance_settings: &BalanceDisplaySettings,
    on_backdrive: Option<&Callback<(ItemIdOrPower, f32)>>,
) -> Html {
    match item {
        Some(item) => item_row(
            id.into(),
            item.name.clone().into(),
            Some(item.image.clone().into()),
            rate,
            balance_settings,
            on_backdrive,
        ),
        None => item_row(
            id.into(),
            "Unknown Item".into(),
            None,
            rate,
            balance_settings,
            on_backdrive,
        ),
    }
}

fn item_row(
    id: ItemIdOrPower,
    title: AttrValue,
    icon: Option<AttrValue>,
    rate: f32,
    display_settings: &BalanceDisplaySettings,
    on_backdrive: Option<&Callback<(ItemIdOrPower, f32)>>,
) -> Html {
    let (power_class, rounding) = match id {
        ItemIdOrPower::Power => (Some("power-entry"), &display_settings.power_format_settings),
        _ => (None, &display_settings.item_format_settings),
    };
    let class = classes!(
        "entry-row",
        balance_style(rate, rounding, display_settings),
        power_class
    );

    let rounded_value: AttrValue = rate.format(rounding).to_string().into();

    match on_backdrive {
        None => html! {
            <div {class} {title}>
                <Icon {icon}/>
                <div class="balance-value">{rounded_value}</div>
            </div>
        },
        Some(on_backdrive) => {
            fn adjust(adjustment: ValueAdjustment, current: AttrValue) -> AttrValue {
                let current = match current.parse::<f32>() {
                    Ok(current) => current,
                    Err(_) => return current,
                };
                let dir = match adjustment.dir {
                    // Multiply by signum so that the direction adjusts the absolute magnitude,
                    // regardless of whether the balance is positive or negative.
                    AdjustDir::Up => 1.0 * current.signum(),
                    AdjustDir::Down => -1.0 * current.signum(),
                };
                let dist = match (adjustment.scale, adjustment.modifier) {
                    // Fine adjustment by increments of 1 building.
                    (AdjustScale::Fine, AdjustModifier::None) => 1.0,
                    // Coarse adjustment by increments of 5 buildings.
                    (AdjustScale::Coarse, AdjustModifier::None) => 5.0,
                    // Small scale adjustments are by 1% and 10% of clock respectively.
                    (AdjustScale::Fine, AdjustModifier::Smaller) => 0.01,
                    (AdjustScale::Coarse, AdjustModifier::Smaller) => 0.1,
                };
                (current + dir * dist).to_string().into()
            }

            let on_backdrive = on_backdrive.clone();
            let on_commit = Callback::from(move |edit_text: AttrValue| {
                if let Ok(value) = edit_text.parse::<f32>() {
                    on_backdrive.emit((id, value));
                }
            });
            let prefix = html!(<Icon {icon} />);
            html! {
                <ClickEdit {class} {prefix} {title} value={rate.to_string()} {rounded_value}
                    {on_commit} adjust={adjust as fn(_,_)->_} />
            }
        }
    }
}

fn balance_style(
    balance: f32,
    rounding: &NumberFormatSettings,
    settings: &BalanceDisplaySettings,
) -> Classes {
    let rate_for_color = display_rate(balance, rounding, settings);
    let rate_for_hide = match settings.hide_style.mode {
        NumberStylingMode::DisplayedValue => balance.round_by_format(rounding),
        NumberStylingMode::ExactValue => balance,
    };
    info!("balance: {balance}, color: {rate_for_color}, hide: {rate_for_hide}");
    let rate_color_mode = if rate_for_color < 0.0 {
        info!("negative: {balance} {rate_for_color}");
        "negative"
    } else if rate_for_color > 0.0 {
        "positive"
    } else {
        "neutral"
    };
    // Handle NaN the same as for color mode.
    let hide_mode = if !(rate_for_hide < 0.0) && !(rate_for_hide > 0.0) {
        Some("hideable-neutral")
    } else {
        None
    };
    classes!(rate_color_mode, hide_mode)
}

/// Get the rate that will be used for display.
fn display_rate(
    balance: f32,
    rounding: &NumberFormatSettings,
    settings: &BalanceDisplaySettings,
) -> f32 {
    match settings.highlight_style.mode {
        NumberStylingMode::DisplayedValue => balance.round_by_format(rounding),
        NumberStylingMode::ExactValue => balance,
    }
}
