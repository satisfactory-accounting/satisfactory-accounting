use std::rc::Rc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use self::balance::Balance;
use crate::database::{
    BuildingId, BuildingKind, BuildingKindId, Database, Generator, Geothermal, ItemId,
    Manufacturer, Miner, Pump, RecipeId,
};

mod balance;

/// Accounting node. Each node has a [`Balance`] telling how much of each item it produces
/// or consumes and how much power it generates or uses.
///
/// Nodes are immutable. Modifying them requires creating new nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node(Rc<NodeInner>);

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

impl Node {
    /// Create a new tree node.
    fn new(kind: impl Into<NodeKind>, balance: Balance) -> Node {
        Self(Rc::new(NodeInner {
            kind: kind.into(),
            balance,
            warning: None,
        }))
    }

    /// Create a node that has no balance because of an error.
    fn warn(kind: impl Into<NodeKind>, warning: BuildError) -> Node {
        Self(Rc::new(NodeInner {
            kind: kind.into(),
            balance: Balance::empty(),
            warning: Some(warning),
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

    /// Get the Group if this is a Group, otherwise None.
    pub fn group(&self) -> Option<&Group> {
        self.kind().group()
    }

    /// Get the Building if this is a Building, otherwise None.
    pub fn building(&self) -> Option<&Building> {
        self.kind().building()
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

/// A grouping of other nodes. It's balance is based
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Group {
    /// Name of this group. May be empty.
    pub name: String,
    /// Child nodes of this node. This node's balance is based on the balances of its
    /// children.
    pub children: Vec<Node>,
}

impl Group {
    /// Create a new empty group.
    pub fn empty() -> Self {
        Default::default()
    }

    /// Create a new empty group wrapped in a node.
    pub fn empty_node() -> Node {
        Self::empty().into()
    }

    /// Compute the net balance for this group, using the *cached* values of child nodes.
    /// Caller is responsible for recaching child balances first if necessary.
    fn compute_balance(&self) -> Balance {
        self.children.iter().map(|node| node.balance()).sum()
    }

    /// Get a child of this node by index.
    pub fn get_child(&self, index: usize) -> Option<&Node> {
        self.children.get(index)
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
                (settings, building_kind) => {
                    return Err(BuildError::MismatchedKind {
                        settings_kind: settings.kind_id(),
                        type_kind: building_kind.kind_id(),
                    });
                }
            }
        }
        Ok(Node::new(self, balance))
    }
}

impl Default for Building {
    fn default() -> Self {
        Self {
            building: None,
            settings: BuildingSettings::PowerConsumer,
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
        }
    }
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

            balance.power = -m.power_consumption.get_rate(self.clock_speed);
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

            balance.power = -m.power_consumption.get_rate(self.clock_speed);
            let cycles_per_minute =
                60.0 / m.cycle_time * self.clock_speed * self.purity.speed_multiplier();

            balance
                .balances
                .insert(resource_id, m.items_per_cycle * cycles_per_minute);
        }
        Ok(balance)
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

            balance.power = g.power_production.get_rate(self.clock_speed);
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
}

/// Building which pumps resources from multiple pads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PumpSettings {
    /// Item being pumped. If not set, balance will be zero.
    pub resource: Option<ItemId>,
    /// Clock setting of this building. Ranges from 0.01 to 2.50.
    pub clock_speed: f32,
    /// How pure each resource pad is. If no pads are set, will still consume power but
    /// will not produce any resources.
    pub pads: Vec<ResourcePurity>,
}

impl Default for PumpSettings {
    fn default() -> Self {
        Self {
            resource: None,
            clock_speed: 1.0,
            pads: vec![Default::default()],
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

            balance.power = -p.power_consumption.get_rate(self.clock_speed);
            let base_cycles_per_minute = 60.0 / p.cycle_time * self.clock_speed;
            let mut total_items_per_minute = 0.0;
            for pad in &self.pads {
                total_items_per_minute +=
                    base_cycles_per_minute * pad.speed_multiplier() * p.items_per_cycle;
            }
            balance.balances.insert(resource_id, total_items_per_minute);
        }
        Ok(balance)
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

mod private {
    use super::*;

    pub trait Sealed {}

    impl Sealed for Group {}
    impl Sealed for Building {}
}
