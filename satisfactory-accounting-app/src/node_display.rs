// Copyright 2021, 2022 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use log::warn;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use satisfactory_accounting::accounting::{
    BuildNode, Building, BuildingSettings, GeneratorSettings, GeothermalSettings, Group,
    ManufacturerSettings, MinerSettings, Node, NodeKind, PumpSettings, ResourcePurity,
    StationSettings,
};
use satisfactory_accounting::database::{
    BuildingId, BuildingKind, BuildingKindId, BuildingType, ItemId, RecipeId,
};

use crate::CtxHelper;

mod balance;
mod building;
mod copies;
mod drag;
mod graph_manipulation;
mod group;
mod icon;

/// Mapping of node medatata by node id.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeMetadata(Rc<HashMap<Uuid, NodeMeta>>);

impl NodeMetadata {
    /// Get the metadata for a particular node by id.
    pub fn meta(&self, uuid: Uuid) -> NodeMeta {
        self.0.get(&uuid).cloned().unwrap_or_default()
    }

    /// Build a version of the metadata with the given value updated.
    pub fn set_meta(&mut self, uuid: Uuid, meta: NodeMeta) {
        Rc::make_mut(&mut self.0).insert(uuid, meta);
    }

    /// Build a version of the metadata with the given values updated.
    pub fn batch_update(&mut self, update: impl IntoIterator<Item = (Uuid, NodeMeta)>) {
        Rc::make_mut(&mut self.0).extend(update);
    }

    /// Prune metadata for anything that isn't referenced from the given node.
    pub fn prune(&mut self, root: &Node) {
        let used_uuids: HashSet<_> = root
            .iter()
            .filter_map(|node| match node.kind() {
                NodeKind::Group(g) => Some(g.id),
                NodeKind::Building(_) => None,
            })
            .collect();
        Rc::make_mut(&mut self.0).retain(|k, _| used_uuids.contains(k));
    }
}

/// Metadata about a node which isn't stored in the tree and isn't available for
/// undo/redo.
#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeMeta {
    /// Whether the node should be shown collapsed or expanded.
    collapsed: bool,
}

#[derive(Debug, PartialEq, Properties)]
pub struct Props {
    /// The node to display.
    pub node: Node,
    /// Path to this node in the tree.
    pub path: Vec<usize>,
    /// Callback to tell the parent to delete this node.
    #[prop_or_default]
    pub delete: Option<Callback<usize>>,
    /// Callback to tell the parent to copy this node.
    #[prop_or_default]
    pub copy: Option<Callback<usize>>,
    /// Callback to tell the parent to replace this node.
    pub replace: Callback<(usize, Node)>,
    /// Callback to tell the parent to move a node.
    pub move_node: Callback<(Vec<usize>, Vec<usize>)>,
    /// Callback to set the metadata of a node.
    pub set_metadata: Callback<(Uuid, NodeMeta)>,
    /// Callback to set the metadata of many nodes at once.
    pub batch_set_metadata: Callback<HashMap<Uuid, NodeMeta>>,
}

/// Messages which can be sent to a Node.
pub enum Msg {
    // Shared messages:
    /// Set the number of virtual copies of this building or group.
    SetCopyCount { copies: u32 },

    // Messages for groups:
    /// Replace the child at the given index with the specified node.
    ReplaceChild { idx: usize, replacement: Node },
    /// Delete the child at the specified index.
    DeleteChild { idx: usize },
    /// Copy the child at the specified index.
    CopyChild { idx: usize },
    /// Add the given node as a child at the end of the list.
    AddChild { child: Node },
    /// Rename this node.
    Rename { name: String },
    /// When another node starts being dragged over this one.
    DragEnter { insert_pos: usize },
    /// When another node is dragged over this one.
    DragOver { insert_pos: usize },
    /// When another dragging node leaves this one.
    DragLeave,
    /// Move a node between positions.
    MoveNode {
        src_path: Vec<usize>,
        dest_path: Vec<usize>,
    },

    // Messages for buildings:
    /// Change the building type of this node.
    ChangeType { id: BuildingId },
    /// Change the recipe for the building, if a manufacturer.
    ChangeRecipe { id: RecipeId },
    /// Change the item for the building, if a Generator, Miner, or Pump.
    ChangeItem { id: ItemId },
    /// Change the clock speed for the building.
    ChangeClockSpeed { clock_speed: f32 },
    /// Change the resource purity for the node the building is on.
    ChangePurity { purity: ResourcePurity },
    /// Change the number of nodes of a particular purity for a pump.
    ChangePumpPurity {
        /// Purity kind to modify.
        purity: ResourcePurity,
        /// New number of pads of that type.
        num_pads: u32,
    },
    /// Change the consumption of a Station.
    ChangeConsumption { consumption: f32 },
}

/// Display for a single AccountingGraph node.
#[derive(Default)]
pub struct NodeDisplay {
    /// Element where children are attached.
    children: NodeRef,
    /// When a drag is in progress and over our children area, this is the proposed insert
    /// position.
    insert_pos: Option<usize>,
    /// Number of virtual insert markers requested. Used to prevent flicker, since
    /// dragenter happens for a new element before dragleave for the prior element.
    insert_count: usize,
}

impl Component for NodeDisplay {
    type Message = Msg;
    type Properties = Props;

    fn create(_: &Context<Self>) -> Self {
        Default::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let our_idx = ctx.props().path.last().copied().unwrap_or_default();
        let db = ctx.db();
        match msg {
            Msg::SetCopyCount { copies } => {
                match ctx.props().node.kind() {
                    NodeKind::Group(group) => {
                        let mut new_group = group.clone();
                        new_group.copies = copies;
                        ctx.props().replace.emit((our_idx, new_group.into()));
                    }
                    NodeKind::Building(building) => {
                        let mut new_bldg = building.clone();
                        new_bldg.copies = copies;
                        match new_bldg.build_node(&db) {
                            Ok(new_node) => ctx.props().replace.emit((our_idx, new_node)),
                            Err(e) => warn!("Unable to build node: {}", e),
                        }
                    }
                }
                false
            }
            Msg::ReplaceChild { idx, replacement } => {
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
            Msg::DeleteChild { idx } => {
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
            Msg::CopyChild { idx } => {
                if let NodeKind::Group(group) = ctx.props().node.kind() {
                    if idx < group.children.len() {
                        let mut new_group = group.clone();
                        let new_meta = RefCell::new(HashMap::new());
                        let copied = new_group.children[idx].create_copy_with_visitor(
                            &|old: &Group, new: &mut Group| {
                                let meta = ctx.meta(old.id);
                                new_meta.borrow_mut().insert(new.id, meta);
                            },
                        );
                        new_group.children.insert(idx + 1, copied);
                        ctx.props().batch_set_metadata.emit(new_meta.into_inner());
                        ctx.props().replace.emit((our_idx, new_group.into()));
                    } else {
                        warn!(
                            "Cannot copy child index {}; out of range for this group",
                            idx
                        );
                    }
                } else {
                    warn!("Cannot copy child of a non-group");
                }
                false
            }
            Msg::AddChild { child } => {
                if let NodeKind::Group(group) = ctx.props().node.kind() {
                    let mut new_group = group.clone();
                    new_group.children.push(child);
                    ctx.props().replace.emit((our_idx, new_group.into()));
                } else {
                    warn!("Cannot add child to a non-group");
                }
                false
            }
            Msg::Rename { name } => {
                if let NodeKind::Group(group) = ctx.props().node.kind() {
                    let name = name.trim().to_owned();
                    if name != group.name {
                        let mut new_group = group.clone();
                        new_group.name = name;
                        ctx.props().replace.emit((our_idx, new_group.into()));
                    }
                } else {
                    warn!("Cannot rename a non-group");
                }
                false
            }
            Msg::DragEnter { insert_pos } => {
                self.insert_count = self
                    .insert_count
                    .checked_add(1)
                    .expect("overflowed insert count");
                if self.insert_pos != Some(insert_pos) {
                    self.insert_pos = Some(insert_pos);
                    true
                } else {
                    false
                }
            }
            Msg::DragOver { insert_pos } => {
                if self.insert_pos != Some(insert_pos) {
                    self.insert_pos = Some(insert_pos);
                    true
                } else {
                    false
                }
            }
            Msg::DragLeave => {
                self.insert_count = self.insert_count.saturating_sub(1);
                if self.insert_count == 0 {
                    self.insert_pos = None;
                    true
                } else {
                    false
                }
            }
            Msg::MoveNode {
                src_path,
                dest_path,
            } => {
                let path = &ctx.props().path[..];
                let prefix_len = path.len();
                debug_assert!(
                    prefix_len < dest_path.len(),
                    "Got asked to move a node for a parent."
                );
                if prefix_len < src_path.len()
                    && path == &src_path[..prefix_len]
                    && path == &dest_path[..prefix_len]
                {
                    // This node is the common ancestor of the source and destination
                    // paths.
                    if let NodeKind::Group(group) = ctx.props().node.kind() {
                        if let Some(new_group) = graph_manipulation::move_child(
                            group,
                            &src_path[prefix_len..],
                            &dest_path[prefix_len..],
                        ) {
                            ctx.props().replace.emit((our_idx, new_group.into()));
                        }
                    } else {
                        warn!("Attempting to move nodes in a non-group.");
                    }
                } else {
                    // No common ancestor yet, ask parent to do the move.
                    ctx.props().move_node.emit((src_path, dest_path));
                }
                if self.insert_pos.is_some() {
                    self.insert_pos = None;
                    true
                } else {
                    false
                }
            }
            Msg::ChangeType { id } => {
                if let NodeKind::Building(building) = ctx.props().node.kind() {
                    if building.building != Some(id) {
                        let mut new_bldg = building.clone();
                        new_bldg.building = Some(id);
                        match db.get(id) {
                            Some(building) => {
                                new_bldg.settings =
                                    new_bldg.settings.build_new_settings(&building.kind);
                            }
                            None => warn!("New building ID is unknown."),
                        }
                        match new_bldg.build_node(&db) {
                            Ok(new_node) => ctx.props().replace.emit((our_idx, new_node)),
                            Err(e) => warn!("Unable to build node: {}", e),
                        }
                    }
                } else {
                    warn!("Cannot change building type id of a non-building");
                }
                false
            }
            Msg::ChangeRecipe { id } => {
                let building = match ctx.props().node.kind() {
                    NodeKind::Building(building) => building,
                    _ => {
                        warn!("Cannot change recipe id of a non-building");
                        return false;
                    }
                };
                if let Some(building_id) = building.building {
                    match db.get(building_id) {
                        Some(BuildingType {
                            kind: BuildingKind::Manufacturer(m),
                            ..
                        }) => {
                            if !m.available_recipes.contains(&id) {
                                warn!(
                                    "Recipe {} is not available for building {}",
                                    id, building_id
                                );
                                return false;
                            }
                        }
                        Some(_) => {
                            warn!("Cannot change recipe id, building is not a manufacturer");
                            return false;
                        }
                        None => {
                            warn!("Cannot change recipe id, unknown building");
                            return false;
                        }
                    }
                } else {
                    warn!("Cannot change recipe id, building not set");
                    return false;
                };
                let settings = ManufacturerSettings {
                    recipe: Some(id),
                    ..match &building.settings {
                        BuildingSettings::Manufacturer(ms) => ms.clone(),
                        settings => {
                            warn!("Had to change building settings kind, did not match building kind in db");
                            ManufacturerSettings {
                                clock_speed: settings.clock_speed(),
                                ..Default::default()
                            }
                        }
                    }
                }.into();
                let new_bldg = Building {
                    settings,
                    ..building.clone()
                };
                match new_bldg.build_node(&db) {
                    Ok(new_node) => ctx.props().replace.emit((our_idx, new_node)),
                    Err(e) => warn!("Unable to build node: {}", e),
                }
                false
            }
            Msg::ChangeItem { id } => {
                let building = match ctx.props().node.kind() {
                    NodeKind::Building(building) => building,
                    _ => {
                        warn!("Cannot change item id of a non-building");
                        return false;
                    }
                };
                let kind_id = if let Some(building_id) = building.building {
                    match db.get(building_id) {
                        Some(BuildingType {
                            kind: BuildingKind::Miner(m),
                            ..
                        }) => {
                            if !m.allowed_resources.contains(&id) {
                                warn!(
                                    "Resource {} is not available for building {}",
                                    id, building_id
                                );
                                return false;
                            }
                            BuildingKindId::Miner
                        }
                        Some(BuildingType {
                            kind: BuildingKind::Generator(g),
                            ..
                        }) => {
                            if !g.allowed_fuel.contains(&id) {
                                warn!("Fuel {} is not available for building {}", id, building_id);
                                return false;
                            }
                            BuildingKindId::Generator
                        }
                        Some(BuildingType {
                            kind: BuildingKind::Pump(p),
                            ..
                        }) => {
                            if !p.allowed_resources.contains(&id) {
                                warn!(
                                    "Resource {} is not available for building {}",
                                    id, building_id
                                );
                                return false;
                            }
                            BuildingKindId::Pump
                        }
                        Some(BuildingType {
                            kind: BuildingKind::Station(s),
                            ..
                        }) => {
                            if !s.allowed_fuel.contains(&id) {
                                warn!("Fuel {} is not available for building {}", id, building_id);
                                return false;
                            }
                            BuildingKindId::Station
                        }
                        Some(_) => {
                            warn!("Cannot change item id, building is not a miner, generator, pump, or station");
                            return false;
                        }
                        None => {
                            warn!("Cannot change recipe id, unknown building");
                            return false;
                        }
                    }
                } else {
                    warn!("Cannot change recipe id, building not set");
                    return false;
                };
                let settings = match (kind_id, &building.settings) {
                    (BuildingKindId::Miner, BuildingSettings::Miner(ms)) => MinerSettings {
                        resource: Some(id),
                        ..ms.clone()
                    }
                    .into(),
                    (BuildingKindId::Miner, settings) => {
                        warn!("Had to change building settings kind, did not match building kind in db");
                        MinerSettings {
                            resource: Some(id),
                            clock_speed: settings.clock_speed(),
                            ..Default::default()
                        }
                        .into()
                    }
                    (BuildingKindId::Generator, BuildingSettings::Generator(gs)) => {
                        GeneratorSettings {
                            fuel: Some(id),
                            ..gs.clone()
                        }
                        .into()
                    }
                    (BuildingKindId::Generator, settings) => {
                        warn!("Had to change building settings kind, did not match building kind in db");
                        GeneratorSettings {
                            fuel: Some(id),
                            clock_speed: settings.clock_speed(),
                            ..Default::default()
                        }
                        .into()
                    }
                    (BuildingKindId::Pump, BuildingSettings::Pump(ms)) => PumpSettings {
                        resource: Some(id),
                        ..ms.clone()
                    }
                    .into(),
                    (BuildingKindId::Pump, settings) => {
                        warn!("Had to change building settings kind, did not match building kind in db");
                        PumpSettings {
                            resource: Some(id),
                            clock_speed: settings.clock_speed(),
                            ..Default::default()
                        }
                        .into()
                    }
                    (BuildingKindId::Station, BuildingSettings::Station(ss)) => StationSettings {
                        fuel: Some(id),
                        ..ss.clone()
                    }
                    .into(),
                    (BuildingKindId::Station, _) => {
                        warn!("Had to change building settings kind, did not match building kind in db");
                        StationSettings {
                            fuel: Some(id),
                            ..Default::default()
                        }
                        .into()
                    }
                    // We know the other BuidingKindId values are impossible because we
                    // only return these three from the previous match.
                    _ => unreachable!(),
                };
                let new_bldg = Building {
                    settings,
                    ..building.clone()
                };
                match new_bldg.build_node(&db) {
                    Ok(new_node) => ctx.props().replace.emit((our_idx, new_node)),
                    Err(e) => warn!("Unable to build node: {}", e),
                }

                false
            }
            Msg::ChangeClockSpeed { clock_speed } => {
                if let NodeKind::Building(building) = ctx.props().node.kind() {
                    if building.settings.clock_speed() != clock_speed {
                        let mut new_bldg = building.clone();
                        new_bldg.settings.set_clock_speed(clock_speed);
                        match new_bldg.build_node(&db) {
                            Ok(new_node) => ctx.props().replace.emit((our_idx, new_node)),
                            Err(e) => warn!("Unable to build node: {}", e),
                        }
                    }
                } else {
                    warn!("Cannot change clock speed of a non-building");
                }
                false
            }
            Msg::ChangePurity { purity } => {
                let building = match ctx.props().node.kind() {
                    NodeKind::Building(building) => building,
                    _ => {
                        warn!("Cannot change purity of a non-building");
                        return false;
                    }
                };
                if building.building.is_none() {
                    warn!("Cannot change purity, building not set");
                    return false;
                };
                let settings = match &building.settings {
                    BuildingSettings::Miner(ms) => MinerSettings {
                        purity,
                        ..ms.clone()
                    }
                    .into(),
                    BuildingSettings::Geothermal(gs) => GeothermalSettings {
                        purity,
                        ..gs.clone()
                    }
                    .into(),
                    _ => {
                        warn!(
                            "Building kind {:?} does not support purity",
                            building.settings.kind_id()
                        );
                        return false;
                    }
                };
                let new_bldg = Building {
                    settings,
                    ..building.clone()
                };
                match new_bldg.build_node(&db) {
                    Ok(new_node) => ctx.props().replace.emit((our_idx, new_node)),
                    Err(e) => warn!("Unable to build node: {}", e),
                }

                false
            }
            Msg::ChangePumpPurity { purity, num_pads } => {
                let building = match ctx.props().node.kind() {
                    NodeKind::Building(building) => building,
                    _ => {
                        warn!("Cannot change purity of a non-building");
                        return false;
                    }
                };
                if building.building.is_none() {
                    warn!("Cannot change pump purity, building not set");
                    return false;
                };
                let settings = match &building.settings {
                    BuildingSettings::Pump(ps) => {
                        let mut ps = ps.clone();
                        match purity {
                            ResourcePurity::Impure => ps.impure_pads = num_pads,
                            ResourcePurity::Normal => ps.normal_pads = num_pads,
                            ResourcePurity::Pure => ps.pure_pads = num_pads,
                        }
                        ps.into()
                    }
                    _ => {
                        warn!(
                            "Building kind {:?} does not support multi-purity",
                            building.settings.kind_id()
                        );
                        return false;
                    }
                };
                let new_bldg = Building {
                    settings,
                    ..building.clone()
                };
                match new_bldg.build_node(&db) {
                    Ok(new_node) => ctx.props().replace.emit((our_idx, new_node)),
                    Err(e) => warn!("Unable to build node: {}", e),
                }

                false
            }
            Msg::ChangeConsumption { consumption } => {
                let building = match ctx.props().node.kind() {
                    NodeKind::Building(building) => building,
                    _ => {
                        warn!("Cannot change station consumption of a non-building");
                        return false;
                    }
                };
                if building.building.is_none() {
                    warn!("Cannot change station consumption, building not set");
                    return false;
                };
                let settings = match &building.settings {
                    BuildingSettings::Station(ss) => StationSettings {
                        consumption,
                        ..ss.clone()
                    }
                    .into(),
                    _ => {
                        warn!(
                            "Building kind {:?} does not support directly setting consumption",
                            building.settings.kind_id()
                        );
                        return false;
                    }
                };
                let new_bldg = Building {
                    settings,
                    ..building.clone()
                };
                match new_bldg.build_node(&db) {
                    Ok(new_node) => ctx.props().replace.emit((our_idx, new_node)),
                    Err(e) => warn!("Unable to build node: {}", e),
                }

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match ctx.props().node.kind() {
            NodeKind::Group(group) => self.view_group(ctx, group),
            NodeKind::Building(building) => self.view_building(ctx, building),
        }
    }
}

/// CSS class that identifies children which identifies the `div` which marks where an
/// element will be dropped. Used to avoid having the insert point count towards the
/// index being chosen for insertion when searching children to figure out what index the
/// drop is at. Also used to style the insert point.
const DRAG_INSERT_POINT: &str = "drag-insert-point";

impl NodeDisplay {
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
                    <button {onclick} class="delete" title="Delete">
                        <span class="material-icons">{"delete"}</span>
                    </button>
                }
            }
            None => html! {},
        }
    }

    /// Creates the copy button, if the parent allows this node to be copied.
    fn copy_button(&self, ctx: &Context<Self>) -> Html {
        match ctx.props().copy.clone() {
            Some(copy_from_parent) => {
                let idx = ctx
                    .props()
                    .path
                    .last()
                    .copied()
                    .expect("Parent provided a copy callback, but this is the root node.");
                let onclick = Callback::from(move |_| copy_from_parent.emit(idx));
                html! {
                    <button {onclick} class="copy" title="Copy">
                        <span class="material-icons">{"content_copy"}</span>
                    </button>
                }
            }
            None => html! {},
        }
    }
}

fn get_value_from_input_event(e: InputEvent) -> String {
    let event: Event = e.dyn_into().unwrap();
    let event_target = event.target().unwrap();
    let target: HtmlInputElement = event_target.dyn_into().unwrap();
    target.value()
}
