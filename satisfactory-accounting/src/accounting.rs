// Copyright 2021, 2022 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use std::{fmt, iter::FusedIterator, rc::Rc};

use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub use self::balance::Balance;
use crate::database::{
    BuildingId, BuildingKind, BuildingKindId, Database, Generator, Geothermal, ItemId,
    Manufacturer, Miner, Pump, RecipeId, Station,
};

mod balance;

/// Trait for types which can visit groups when creating copies.
pub trait GroupCopyVisitor {
    fn visit(&self, original: &Group, copy: &mut Group);
}

impl<F> GroupCopyVisitor for F
where
    F: Fn(&Group, &mut Group),
{
    fn visit(&self, original: &Group, copy: &mut Group) {
        self(original, copy)
    }
}

/// Trait for types that can be turned into nodes.
pub trait BuildNode: private::Sealed {
    /// Create a node from this type. Uses the database to compute the balance of the
    /// node.
    fn build_node(self, database: &Database) -> Result<Node, BuildError>;
}

/// Error found when building a [`Node`].
#[derive(Error, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BuildError {
    #[error("Building ID {0} is not in the database.")]
    UnknownBuilding(BuildingId),
    #[error("Recipe ID {0} is not in the database.")]
    UnknownRecipe(RecipeId),
    #[error("Item ID {0} is not in the database.")]
    UnknownItem(ItemId),
    #[error("Item ID {0} is not a fuel.")]
    NotFuel(ItemId),
    #[error("Recipe {recipe} is not compatible with building {building}.")]
    IncompatibleRecipe {
        recipe: RecipeId,
        building: BuildingId,
    },
    #[error("Item {item} is not compatible with building {building}.")]
    IncompatibleItem { item: ItemId, building: BuildingId },
    #[error("Mismatched BuildingKind between Building ({settings_kind:?}) and BuildingType ({type_kind:?}).")]
    MismatchedKind {
        /// BuildingKindId of the settings for the [`Building`].
        settings_kind: BuildingKindId,
        /// BuildingKindId of the [`BuildingType`].
        type_kind: BuildingKindId,
    },
}

impl BuildError {
    /// Builds a node with this error as a waning.
    #[inline]
    pub fn into_warning_node(self, kind: impl Into<NodeKind>) -> Node {
        Node::warn(kind, self)
    }
}

/// Accounting node. Each node has a [`Balance`] telling how much of each item it produces
/// or consumes and how much power it generates or uses.
///
/// Nodes are immutable. Modifying them requires creating new nodes.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Node(Rc<NodeInner>);

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Node, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Recompute children_had_warnings on deserialization.
        let mut node_inner = NodeInner::deserialize(deserializer)?;
        node_inner.children_had_warnings = check_for_child_warnings(&node_inner.kind);
        Ok(Node(Rc::new(node_inner)))
    }
}

/// Checks if any child of this node kind has warnings or any of its descendents have
/// warnings.
fn check_for_child_warnings(kind: &NodeKind) -> bool {
    match kind {
        NodeKind::Group(group) => group
            .children
            .iter()
            .any(|child| child.warning().is_some() || child.children_had_warnings()),
        NodeKind::Building(_) => false,
    }
}

impl Node {
    /// Create a new tree node.
    fn new(kind: impl Into<NodeKind>, balance: Balance) -> Node {
        let kind = kind.into();
        let children_had_warnings = check_for_child_warnings(&kind);
        Self(Rc::new(NodeInner {
            kind,
            balance,
            warning: None,
            children_had_warnings,
        }))
    }

    /// Create a node that has no balance because of an error.
    fn warn(kind: impl Into<NodeKind>, warning: BuildError) -> Node {
        let kind = kind.into();
        let children_had_warnings = check_for_child_warnings(&kind);
        Self(Rc::new(NodeInner {
            kind,
            balance: Balance::empty(),
            warning: Some(warning),
            children_had_warnings,
        }))
    }

    /// Get the kind of this node.
    pub fn kind(&self) -> &NodeKind {
        &self.0.kind
    }

    /// Get the balance of this node.
    pub fn balance(&self) -> &Balance {
        &self.0.balance
    }

    /// Get the warning for this error.
    pub fn warning(&self) -> Option<BuildError> {
        self.0.warning
    }

    /// Returns true if any child of this node (but not the node itself) has a build
    /// warning. Always false for buildings, since buildings cannot have children.
    pub fn children_had_warnings(&self) -> bool {
        self.0.children_had_warnings
    }

    /// Get the Group if this is a Group, otherwise None.
    pub fn group(&self) -> Option<&Group> {
        self.kind().group()
    }

    /// Get the Building if this is a Building, otherwise None.
    pub fn building(&self) -> Option<&Building> {
        self.kind().building()
    }

    /// Create a copy of this node. This is a true copy, with Uuids of Groups changed to
    /// represent newly created, but identical groups.
    pub fn create_copy(&self) -> Self {
        match self.kind() {
            NodeKind::Group(group) => group.create_copy().into(),
            // Buidings have no identity and can be copied verbatim.
            NodeKind::Building(_) => self.clone(),
        }
    }

    /// Create a copy of this node. This is a true copy, with Uuids of Groups changed to
    /// represent newly created, but identical groups. A visitor can be provided to view
    /// the newly created groups, e.g. to copy non-tree data such as metadata.
    pub fn create_copy_with_visitor(&self, visitor: &impl GroupCopyVisitor) -> Self {
        match self.kind() {
            NodeKind::Group(group) => group.create_copy_with_visitor(visitor).into(),
            // Buidings have no identity and can be copied verbatim.
            NodeKind::Building(_) => self.clone(),
        }
    }

    /// Rebuild this node with a new database.
    pub fn rebuild(&self, new_db: &Database) -> Self {
        match self.kind() {
            NodeKind::Group(group) => group.rebuild(new_db),
            NodeKind::Building(building) => building.rebuild(new_db),
        }
    }

    /// Get the children of this node, if any.
    pub fn children(
        &self,
    ) -> impl '_ + Iterator<Item = Node> + ExactSizeIterator + DoubleEndedIterator + FusedIterator
    {
        match self.kind() {
            NodeKind::Group(group) => group.children.iter().cloned(),
            NodeKind::Building(_) => [].iter().cloned(),
        }
    }

    /// Pre-order traversal iterator of this node.
    pub fn iter(&self) -> NodeIter {
        NodeIter {
            to_visit: vec![self.clone()],
        }
    }
}

pub struct NodeIter {
    // Node stack.
    to_visit: Vec<Node>,
}

impl Iterator for NodeIter {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        match self.to_visit.pop() {
            Some(node) => {
                self.to_visit.extend(node.children());
                Some(node)
            }
            None => None,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct NodeInner {
    /// Type of this node.
    kind: NodeKind,

    /// Net balance of this node.
    balance: Balance,

    /// Warnings generated when building this node.
    warning: Option<BuildError>,

    /// Whether this node has any children with warnings.
    #[serde(skip)]
    children_had_warnings: bool,
}

/// Kind of node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeKind {
    Group(Group),
    Building(Building),
}

impl NodeKind {
    /// Get the Group if this is a Group, otherwise None.
    pub fn group(&self) -> Option<&Group> {
        match self {
            Self::Group(group) => Some(group),
            _ => None,
        }
    }

    /// Get the Building if this is a Building, otherwise None.
    pub fn building(&self) -> Option<&Building> {
        match self {
            Self::Building(building) => Some(building),
            _ => None,
        }
    }
}

impl From<Group> for NodeKind {
    fn from(group: Group) -> Self {
        Self::Group(group)
    }
}

impl From<Building> for NodeKind {
    fn from(building: Building) -> Self {
        Self::Building(building)
    }
}

/// Provides the default number of virtual copies for Serde to allow deserializing from
/// before that field was added.
fn default_copies() -> u32 {
    1
}

/// A grouping of other nodes. It's balance is based on its child nodes.
///
/// Note that cloning groups is used to update groups. When creating a new a copy of a
/// group, a new [`Uuid`] needs to be created, and this must be recursive, since two
/// groups with the same `Uuid` in the same tree is not allowed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Group {
    /// Name of this group. May be empty.
    pub name: String,
    /// Child nodes of this node. This node's balance is based on the balances of its
    /// children.
    pub children: Vec<Node>,
    /// Number of virtual copies of this group. This acts as a multiplier on the balance.
    #[serde(default = "default_copies")]
    pub copies: u32,

    /// Uniquely identifies a group, even when the node is shared between trees (e.g. when
    /// saving nodes for undo/redo purposes).
    pub id: Uuid,
}

impl Group {
    /// Create a new empty group.
    pub fn empty() -> Self {
        Group {
            name: Default::default(),
            children: Default::default(),
            copies: 1,
            id: Uuid::new_v4(),
        }
    }

    /// Create a new empty group wrapped in a node.
    pub fn empty_node() -> Node {
        Self::empty().into()
    }

    /// Compute the net balance for this group, using the *cached* values of child nodes.
    /// Caller is responsible for recaching child balances first if necessary.
    fn compute_balance(&self) -> Balance {
        let mut balance = self.children.iter().map(|node| node.balance()).sum();
        balance *= self.copies as f32;
        balance
    }

    /// Get a child of this node by index.
    pub fn get_child(&self, index: usize) -> Option<&Node> {
        self.children.get(index)
    }

    /// Create a true copy of this group, with a newly assigned Uuid. Unlike the result of
    /// `Clone`, the new value doesn't represent the same group, so can be used in the
    /// same tree as the original.
    pub fn create_copy(&self) -> Self {
        Group {
            name: self.name.clone(),
            children: self
                .children
                .iter()
                .map(|child| child.create_copy())
                .collect(),
            copies: self.copies,
            id: Uuid::new_v4(),
        }
    }

    /// Create a true copy of this group, with a newly assigned Uuid. Unlike the result of
    /// `Clone`, the new value doesn't represent the same group, so can be used in the
    /// same tree as the original. A visitor can be used to view the original group and
    /// copy simultaneously. This can be used e.g. to copy out-of-tree related data such
    /// as metadata.
    pub fn create_copy_with_visitor(&self, visitor: &impl GroupCopyVisitor) -> Self {
        let mut copy = Group {
            name: self.name.clone(),
            children: self
                .children
                .iter()
                .map(|child| child.create_copy_with_visitor(visitor))
                .collect(),
            copies: self.copies,
            id: Uuid::new_v4(),
        };
        visitor.visit(self, &mut copy);
        copy
    }

    /// Rebuild this node with a new database.
    fn rebuild(&self, new_db: &Database) -> Node {
        let mut copy = self.clone();
        for child in &mut copy.children {
            *child = child.rebuild(new_db);
        }
        copy.into()
    }
}

impl From<Group> for Node {
    fn from(group: Group) -> Self {
        let balance = group.compute_balance();
        Node::new(group, balance)
    }
}

impl BuildNode for Group {
    fn build_node(self, _database: &Database) -> Result<Node, BuildError> {
        Ok(self.into())
    }
}

/// An instance of a building of a particular type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Building {
    /// Building being used. If not set, balance will be zero.
    pub building: Option<BuildingId>,
    /// Settings for this building. Must match the BuildingKind of the building.
    pub settings: BuildingSettings,
    /// Number of copies of this building.
    #[serde(default = "default_copies")]
    pub copies: u32,
}

impl Building {
    /// Create a new empty building.
    pub fn empty() -> Self {
        Default::default()
    }

    /// Create a new node for an unassigned building.
    pub fn empty_node() -> Node {
        Node::new(Self::empty(), Balance::empty())
    }

    /// Rebuild this node with a new database, converting errors to warnings.
    fn rebuild(&self, new_db: &Database) -> Node {
        match self.clone().build_node(new_db) {
            Ok(node) => node,
            Err(err) => err.into_warning_node(self.clone()),
        }
    }
}

impl BuildNode for Building {
    fn build_node(self, database: &Database) -> Result<Node, BuildError> {
        let mut balance = Balance::empty();
        if let Some(building_id) = self.building {
            let building = database
                .get(building_id)
                .ok_or(BuildError::UnknownBuilding(building_id))?;
            match (&self.settings, &building.kind) {
                (BuildingSettings::Manufacturer(ms), BuildingKind::Manufacturer(m)) => {
                    balance = ms.get_balance(building_id, m, database)?;
                }
                (BuildingSettings::Miner(ms), BuildingKind::Miner(m)) => {
                    balance = ms.get_balance(building_id, m, database)?;
                }
                (BuildingSettings::Generator(gs), BuildingKind::Generator(g)) => {
                    balance = gs.get_balance(building_id, g, database)?;
                }
                (BuildingSettings::Pump(ps), BuildingKind::Pump(p)) => {
                    balance = ps.get_balance(building_id, p, database)?;
                }
                (BuildingSettings::Geothermal(gs), BuildingKind::Geothermal(g)) => {
                    balance = gs.get_balance(g);
                }
                (BuildingSettings::PowerConsumer, BuildingKind::PowerConsumer(p)) => {
                    balance = Balance::power_only(-p.power);
                }
                (BuildingSettings::Station(ss), BuildingKind::Station(s)) => {
                    balance = ss.get_balance(building_id, s, database)?;
                }
                (settings, building_kind) => {
                    return Err(BuildError::MismatchedKind {
                        settings_kind: settings.kind_id(),
                        type_kind: building_kind.kind_id(),
                    });
                }
            }
        }
        balance *= self.copies as f32;
        Ok(Node::new(self, balance))
    }
}

impl Default for Building {
    fn default() -> Self {
        Self {
            building: None,
            settings: BuildingSettings::PowerConsumer,
            copies: 1,
        }
    }
}

/// Settings for a building of a particular kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BuildingSettings {
    Manufacturer(ManufacturerSettings),
    Miner(MinerSettings),
    Generator(GeneratorSettings),
    Pump(PumpSettings),
    Geothermal(GeothermalSettings),
    PowerConsumer,
    Station(StationSettings),
}

impl BuildingSettings {
    /// Get the ID of this buiilding kind.
    pub fn kind_id(&self) -> BuildingKindId {
        match self {
            Self::Manufacturer(_) => BuildingKindId::Manufacturer,
            Self::Miner(_) => BuildingKindId::Miner,
            Self::Generator(_) => BuildingKindId::Generator,
            Self::Pump(_) => BuildingKindId::Pump,
            Self::Geothermal(_) => BuildingKindId::Geothermal,
            Self::PowerConsumer => BuildingKindId::PowerConsumer,
            Self::Station(_) => BuildingKindId::Station,
        }
    }

    /// Get the clock speed of the building.
    pub fn clock_speed(&self) -> f32 {
        match self {
            Self::Manufacturer(m) => m.clock_speed,
            Self::Miner(m) => m.clock_speed,
            Self::Generator(g) => g.clock_speed,
            Self::Pump(p) => p.clock_speed,
            Self::Geothermal(_) => 1.0,
            Self::PowerConsumer => 1.0,
            Self::Station(_) => 1.0,
        }
    }

    /// Set the clock speed of the building if possible.
    pub fn set_clock_speed(&mut self, clock_speed: f32) {
        match self {
            Self::Manufacturer(m) => m.clock_speed = clock_speed,
            Self::Miner(m) => m.clock_speed = clock_speed,
            Self::Generator(g) => g.clock_speed = clock_speed,
            Self::Pump(p) => p.clock_speed = clock_speed,
            Self::Geothermal(_) => {}
            Self::PowerConsumer => {}
            Self::Station(_) => {}
        }
    }

    /// Get replacment settings for changing a building, by copying the settings a much as
    /// possible.
    pub fn build_new_settings(&self, new_kind: &BuildingKind) -> Self {
        match (self, new_kind) {
            (BuildingSettings::Manufacturer(ms), BuildingKind::Manufacturer(m)) => {
                BuildingSettings::Manufacturer(ms.copy_settings(m))
            }
            (BuildingSettings::Miner(ms), BuildingKind::Miner(m)) => {
                BuildingSettings::Miner(ms.copy_settings(m))
            }
            (BuildingSettings::Generator(gs), BuildingKind::Generator(g)) => {
                BuildingSettings::Generator(gs.copy_settings(g))
            }
            (BuildingSettings::Pump(ps), BuildingKind::Pump(p)) => {
                BuildingSettings::Pump(ps.copy_settings(p))
            }
            (BuildingSettings::Geothermal(gs), BuildingKind::Geothermal(_)) => {
                BuildingSettings::Geothermal(gs.clone())
            }
            (BuildingSettings::Station(ss), BuildingKind::Station(s)) => {
                BuildingSettings::Station(ss.copy_settings(s))
            }
            _ => {
                // For mismatched types, just copy the clock speed.
                let mut new_settings = new_kind.get_default_settings();
                new_settings.set_clock_speed(self.clock_speed());
                new_settings
            }
        }
    }
}

macro_rules! settings_from_inner {
    ($($variant:ident ($inner:ident);)+) => {
        $(
            impl From<$inner> for BuildingSettings {
                fn from(inner: $inner) -> Self {
                    Self::$variant(inner)
                }
            }
        )+
    };
}

settings_from_inner! {
    Manufacturer(ManufacturerSettings);
    Miner(MinerSettings);
    Generator(GeneratorSettings);
    Pump(PumpSettings);
    Geothermal(GeothermalSettings);
    Station(StationSettings);
}

/// Building which manufactures items using a recipe that converts input items to output
/// items.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManufacturerSettings {
    /// Recipe being produced. If not set, balance will be zero.
    pub recipe: Option<RecipeId>,
    /// Clock setting of this building. Ranges from 0.01 to 2.50 (unit is fraction, not
    /// percent).
    pub clock_speed: f32,
}

impl Default for ManufacturerSettings {
    fn default() -> Self {
        Self {
            recipe: None,
            clock_speed: 1.0,
        }
    }
}

impl ManufacturerSettings {
    /// Get the balance for this manufacturer.
    fn get_balance(
        &self,
        building_id: BuildingId,
        m: &Manufacturer,
        database: &Database,
    ) -> Result<Balance, BuildError> {
        let mut balance = Balance::empty();
        if let Some(recipe_id) = self.recipe {
            let recipe = database
                .get(recipe_id)
                .ok_or(BuildError::UnknownRecipe(recipe_id))?;

            if !m.available_recipes.contains(&recipe_id) {
                return Err(BuildError::IncompatibleRecipe {
                    recipe: recipe_id,
                    building: building_id,
                });
            }

            balance.power = -m.power_consumption.get_consumption_rate(self.clock_speed);
            let recipe_runs_per_minute =
                60.0 / recipe.time * m.manufacturing_speed * self.clock_speed;

            for input in &recipe.ingredients {
                *balance.balances.entry(input.item).or_default() -=
                    input.amount * recipe_runs_per_minute;
            }
            for output in &recipe.products {
                *balance.balances.entry(output.item).or_default() +=
                    output.amount * recipe_runs_per_minute;
            }
        }
        Ok(balance)
    }

    /// Create a copy of these settings for a different manufacturer.
    fn copy_settings(&self, m: &Manufacturer) -> Self {
        let mut ms = self.clone();
        // leave clock the same and reset the recipe if our current recipe isn't allowed.
        // If the new building allows only one recipe, choose that.
        if let Some(recipe) = ms.recipe {
            if !m.available_recipes.contains(&recipe) {
                ms.recipe = if m.available_recipes.len() == 1 {
                    m.available_recipes.first().copied()
                } else {
                    None
                }
            }
        } else if m.available_recipes.len() == 1 {
            ms.recipe = m.available_recipes.first().copied();
        }
        ms
    }
}

/// Purity of a source resource.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourcePurity {
    Impure,
    Normal,
    Pure,
}

impl Default for ResourcePurity {
    fn default() -> Self {
        ResourcePurity::Normal
    }
}

impl ResourcePurity {
    /// Get the speed multiplier for this resource purity level.
    pub fn speed_multiplier(self) -> f32 {
        match self {
            Self::Impure => 0.5,
            Self::Normal => 1.0,
            Self::Pure => 2.0,
        }
    }

    /// Get the next higher purity level. Saturates on overflow.
    pub fn next(self) -> Self {
        match self {
            Self::Impure => Self::Normal,
            _ => Self::Pure,
        }
    }

    /// Get the next lower purity level. Saturates on overflow.
    pub fn previous(self) -> Self {
        match self {
            Self::Pure => Self::Normal,
            _ => Self::Impure,
        }
    }

    /// Get a string suitable for human display of this purity.
    pub fn name(self) -> &'static str {
        match self {
            Self::Impure => "Impure",
            Self::Normal => "Normal",
            Self::Pure => "Pure",
        }
    }

    /// Get a string suitable for identifying this purity.
    pub fn ident(self) -> &'static str {
        match self {
            Self::Impure => "impure",
            Self::Normal => "normal",
            Self::Pure => "pure",
        }
    }

    /// Gets the purity matching an ident from [`Self::ident`].
    pub fn from_ident(ident: &str) -> Result<Self, ()> {
        match ident {
            "impure" => Ok(Self::Impure),
            "normal" => Ok(Self::Normal),
            "pure" => Ok(Self::Pure),
            _ => Err(()),
        }
    }

    /// Get an iterator over the values of this enum.
    pub fn values(
    ) -> impl Iterator<Item = ResourcePurity> + DoubleEndedIterator + ExactSizeIterator + FusedIterator
    {
        (&[Self::Impure, Self::Normal, Self::Pure]).iter().copied()
    }
}

impl fmt::Display for ResourcePurity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Building which mines a resource.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MinerSettings {
    /// Item being mined. If not set, balance will be zero.
    pub resource: Option<ItemId>,
    /// Clock setting of this building. Ranges from 0.01 to 2.50.
    pub clock_speed: f32,
    /// Purity of the node this miner is built on.
    pub purity: ResourcePurity,
}

impl Default for MinerSettings {
    fn default() -> Self {
        Self {
            resource: None,
            clock_speed: 1.0,
            purity: Default::default(),
        }
    }
}

impl MinerSettings {
    fn get_balance(
        &self,
        building_id: BuildingId,
        m: &Miner,
        database: &Database,
    ) -> Result<Balance, BuildError> {
        let mut balance = Balance::empty();
        if let Some(resource_id) = self.resource {
            database
                .get(resource_id)
                .ok_or(BuildError::UnknownItem(resource_id))?;

            if !m.allowed_resources.contains(&resource_id) {
                return Err(BuildError::IncompatibleItem {
                    item: resource_id,
                    building: building_id,
                });
            }

            balance.power = -m.power_consumption.get_consumption_rate(self.clock_speed);
            let cycles_per_minute =
                60.0 / m.cycle_time * self.clock_speed * self.purity.speed_multiplier();

            balance
                .balances
                .insert(resource_id, m.items_per_cycle * cycles_per_minute);
        }
        Ok(balance)
    }

    /// Create a copy of these settings for a different miner.
    fn copy_settings(&self, m: &Miner) -> Self {
        let mut ms = self.clone();
        // leave clock and purity the same and reset the resource if our current resource
        // isn't allowed.  If the new building allows only one resourece, choose that.
        if let Some(resource) = ms.resource {
            if !m.allowed_resources.contains(&resource) {
                ms.resource = if m.allowed_resources.len() == 1 {
                    m.allowed_resources.first().copied()
                } else {
                    None
                }
            }
        } else if m.allowed_resources.len() == 1 {
            ms.resource = m.allowed_resources.first().copied();
        }
        ms
    }
}

/// Building which produces power by burning items.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneratorSettings {
    /// Item consumed as fuel.
    pub fuel: Option<ItemId>,
    /// Clock setting of this building. Ranges from 0.01 to 2.50.
    pub clock_speed: f32,
}

impl Default for GeneratorSettings {
    fn default() -> Self {
        Self {
            fuel: None,
            clock_speed: 1.0,
        }
    }
}

impl GeneratorSettings {
    fn get_balance(
        &self,
        building_id: BuildingId,
        g: &Generator,
        database: &Database,
    ) -> Result<Balance, BuildError> {
        let mut balance = Balance::empty();
        if let Some(fuel_id) = self.fuel {
            let fuel = database
                .get(fuel_id)
                .ok_or(BuildError::UnknownItem(fuel_id))?;

            let energy = fuel.fuel.as_ref().ok_or(BuildError::NotFuel(fuel_id))?;

            if !g.allowed_fuel.contains(&fuel_id) {
                return Err(BuildError::IncompatibleItem {
                    item: fuel_id,
                    building: building_id,
                });
            }

            balance.power = g.power_production.get_production_rate(self.clock_speed);
            if g.used_water > 0.0 {
                balance
                    .balances
                    .insert(ItemId::water(), -balance.power * g.used_water);
            }
            // Burn time in Seconds MJ / MW = MJ/(MJ/s) = s
            let fuel_burn_time = energy.energy / balance.power;
            *balance.balances.entry(fuel_id).or_default() -= 60.0 / fuel_burn_time;
        }
        Ok(balance)
    }

    /// Create a copy of these settings for a different generator.
    fn copy_settings(&self, g: &Generator) -> Self {
        let mut gs = self.clone();
        // leave clock the same and reset the fuel if our current fuel isn't allowed.
        // If the new building allows only one fuel, choose that.
        if let Some(fuel) = gs.fuel {
            if !g.allowed_fuel.contains(&fuel) {
                gs.fuel = if g.allowed_fuel.len() == 1 {
                    g.allowed_fuel.first().copied()
                } else {
                    None
                }
            }
        } else if g.allowed_fuel.len() == 1 {
            gs.fuel = g.allowed_fuel.first().copied();
        }
        gs
    }
}

/// Building which pumps resources from multiple pads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PumpSettings {
    /// Item being pumped. If not set, balance will be zero.
    pub resource: Option<ItemId>,
    /// Clock setting of this building. Ranges from 0.01 to 2.50.
    pub clock_speed: f32,
    /// Number of pure resource pads. If no pads are set, will still consume power but
    /// will not produce any resources.
    pub pure_pads: u32,
    /// Number of normal resource pads. If no pads are set, will still consume power but
    /// will not produce any resources.
    pub normal_pads: u32,
    /// Number of normal resource pads. If no pads are set, will still consume power but
    /// will not produce any resources.
    pub impure_pads: u32,
}

impl Default for PumpSettings {
    fn default() -> Self {
        Self {
            resource: None,
            clock_speed: 1.0,
            pure_pads: 0,
            normal_pads: 0,
            impure_pads: 0,
        }
    }
}

impl PumpSettings {
    fn get_balance(
        &self,
        building_id: BuildingId,
        p: &Pump,
        database: &Database,
    ) -> Result<Balance, BuildError> {
        let mut balance = Balance::empty();
        if let Some(resource_id) = self.resource {
            database
                .get(resource_id)
                .ok_or(BuildError::UnknownItem(resource_id))?;

            if !p.allowed_resources.contains(&resource_id) {
                return Err(BuildError::IncompatibleItem {
                    item: resource_id,
                    building: building_id,
                });
            }

            balance.power = -p.power_consumption.get_consumption_rate(self.clock_speed);
            let base_cycles_per_minute = 60.0 / p.cycle_time * self.clock_speed;
            let total_items_per_minute = base_cycles_per_minute
                * p.items_per_cycle
                * (self.pure_pads as f32 * ResourcePurity::Pure.speed_multiplier()
                    + self.normal_pads as f32 * ResourcePurity::Normal.speed_multiplier()
                    + self.impure_pads as f32 * ResourcePurity::Impure.speed_multiplier());
            balance.balances.insert(resource_id, total_items_per_minute);
        }
        Ok(balance)
    }

    /// Create a copy of these settings for a different pump.
    fn copy_settings(&self, p: &Pump) -> Self {
        let mut ps = self.clone();
        // leave clock the same and reset the fuel if our current fuel isn't allowed.
        // If the new building allows only one fuel, choose that.
        if let Some(resource) = ps.resource {
            if !p.allowed_resources.contains(&resource) {
                ps.resource = if p.allowed_resources.len() == 1 {
                    p.allowed_resources.first().copied()
                } else {
                    None
                }
            }
        } else if p.allowed_resources.len() == 1 {
            ps.resource = p.allowed_resources.first().copied();
        }
        ps
    }
}

/// Building which produces power directly from a geothermal resource pad.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeothermalSettings {
    /// Purity of the pad, affects generated power.
    pub purity: ResourcePurity,
}

impl Default for GeothermalSettings {
    fn default() -> Self {
        Self {
            purity: Default::default(),
        }
    }
}

impl GeothermalSettings {
    fn get_balance(&self, g: &Geothermal) -> Balance {
        Balance::power_only(self.purity.speed_multiplier() * g.power)
    }
}

/// Building which pumps resources from multiple pads.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct StationSettings {
    /// Fuel type being used.
    pub fuel: Option<ItemId>,
    /// Configured fuel consumption rate.
    pub consumption: f32,
}

impl StationSettings {
    fn get_balance(
        &self,
        building_id: BuildingId,
        s: &Station,
        database: &Database,
    ) -> Result<Balance, BuildError> {
        let mut balance = Balance::empty();
        if let Some(fuel_id) = self.fuel {
            database
                .get(fuel_id)
                .ok_or(BuildError::UnknownItem(fuel_id))?;

            if !s.allowed_fuel.contains(&fuel_id) {
                return Err(BuildError::IncompatibleItem {
                    item: fuel_id,
                    building: building_id,
                });
            }

            balance.power = -s.power;
            balance.balances.insert(fuel_id, -self.consumption);
        }
        Ok(balance)
    }

    /// Create a copy of these settings for a different pump.
    fn copy_settings(&self, s: &Station) -> Self {
        let mut ss = self.clone();
        // leave clock the same and reset the fuel if our current fuel isn't allowed.
        // If the new building allows only one fuel, choose that.
        if let Some(fuel) = ss.fuel {
            if !s.allowed_fuel.contains(&fuel) {
                ss.fuel = if s.allowed_fuel.len() == 1 {
                    s.allowed_fuel.first().copied()
                } else {
                    None
                }
            }
        } else if s.allowed_fuel.len() == 1 {
            ss.fuel = s.allowed_fuel.first().copied();
        }
        ss
    }
}

mod private {
    use super::*;

    pub trait Sealed {}

    impl Sealed for Group {}
    impl Sealed for Building {}
}
