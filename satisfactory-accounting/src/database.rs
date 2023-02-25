// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;
use std::hash::Hash;
use std::ops::Index;
use std::rc::Rc;

use internment::Intern;
use serde::{Deserialize, Serialize};

use crate::accounting::{
    BuildingSettings, GeneratorSettings, ManufacturerSettings, MinerSettings, PumpSettings,
    StationSettings,
};

/// Enum which identifies versions of the database.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "major", content = "minor")]
pub enum DatabaseVersion {
    /// U5 database versions.
    U5(U5Subversion),
    /// U6 database versions.
    U6(U6Subversion),
    /// U7 database versions.
    U7(U7Subversion),
}

impl DatabaseVersion {
    /// All released database versions in order.
    pub const ALL: &[DatabaseVersion] = &[
        DatabaseVersion::U5(U5Subversion::Initial),
        DatabaseVersion::U5(U5Subversion::Final),
        DatabaseVersion::U6(U6Subversion::Beta),
        DatabaseVersion::U7(U7Subversion::Final),
    ];

    /// Latest version of the database.
    pub const LATEST: DatabaseVersion = Self::ALL[Self::ALL.len() - 1];

    /// Identifies which database versions are considered deprecated.
    pub fn is_deprecated(self) -> bool {
        match self {
            DatabaseVersion::U7(U7Subversion::Final) => false,
            _ => true,
        }
    }

    /// Load this version of the database from the included database string.
    pub fn load_database(self) -> Database {
        match self {
            DatabaseVersion::U5(U5Subversion::Initial) => {
                const SERIALIZED_DB: &str = include_str!("../db-u5-initial.json");
                serde_json::from_str(SERIALIZED_DB).expect("Failed to parse db-u5-initial.json")
            }
            DatabaseVersion::U5(U5Subversion::Final) => {
                const SERIALIZED_DB: &str = include_str!("../db-u5-final.json");
                serde_json::from_str(SERIALIZED_DB).expect("Failed to parse db-u5-final.json")
            }
            DatabaseVersion::U6(U6Subversion::Beta) => {
                const SERIALIZED_DB: &str = include_str!("../db-u6-beta.json");
                serde_json::from_str(SERIALIZED_DB).expect("Failed to parse db-u6-beta.json")
            }
            DatabaseVersion::U7(U7Subversion::Final) => {
                const SERIALIZED_DB: &str = include_str!("../db-u7-final.json");
                serde_json::from_str(SERIALIZED_DB).expect("Failed to parse db-u7-final.json")
            }
        }
    }

    /// Get the displayable name for this database version.
    pub fn name(self) -> &'static str {
        match self {
            DatabaseVersion::U5(U5Subversion::Initial) => "U5 \u{2013} Initial",
            DatabaseVersion::U5(U5Subversion::Final) => "U5 \u{2013} Final",
            DatabaseVersion::U6(U6Subversion::Beta) => "U6 \u{2013} Beta",
            DatabaseVersion::U7(U7Subversion::Final) => "U7 \u{2013} Final",
        }
    }

    /// Get the description for this version.
    pub fn description(self) -> &'static str {
        match self {
            DatabaseVersion::U5(U5Subversion::Initial) => {
                "This is the first version of the database released for U5. Fuel \
                generators in this version consume 1000x too much fuel."
            }
            DatabaseVersion::U5(U5Subversion::Final) => {
                "This is the final version of the database released for U5."
            }
            DatabaseVersion::U6(U6Subversion::Beta) => {
                "This is the first version of the Satisfactory Accounting database \
                released after the U6 update. Please report missing/incorrect recipes!"
            }
            DatabaseVersion::U7(U7Subversion::Final) => {
                "This is the final version of the database released for U7."
            }
        }
    }
}

/// Minor versions within the U5 database.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum U5Subversion {
    /// Initial database released in Satisfactory Accounting 1.0.0.
    Initial,
    /// Final variant of U5 released in Satisfactory Accounting released in 1.0.1.
    Final,
}

/// Minor versions with in the U6 database.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum U6Subversion {
    /// Initial release of U6 for Satisfactory Accounting released in 1.1.0.
    Beta,
}

/// Minor versions with in the U7 database.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum U7Subversion {
    /// Final release of U7 released in Satisfactory Accounting released in 1.2.0.
    Final,
}

impl fmt::Display for DatabaseVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Database of satisfactory ... stuff.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Database {
    /// Which version of the database this is, if it corresponds to a particular version.
    #[serde(default)]
    pub icon_prefix: String,
    /// Core recipe storage. We only store machine recipes.
    pub recipes: BTreeMap<RecipeId, Recipe>,
    /// Core item storage.
    pub items: BTreeMap<ItemId, Item>,
    /// Core buildings storage.
    pub buildings: BTreeMap<BuildingId, BuildingType>,
}

impl Database {
    /// Get an item, recipe, or building by id.
    pub fn get<T: Id>(&self, id: T) -> Option<&<T as Id>::Info> {
        id.fetch(self)
    }

    /// Load the default version of the database.
    pub fn load_default() -> Database {
        DatabaseVersion::U7(U7Subversion::Final).load_database()
    }

    /// Compare this database to another database, ignoring their icon prefixes.
    pub fn compare_ignore_prefix(&self, other: &Database) -> bool {
        self.recipes == other.recipes
            && self.items == other.items
            && self.buildings == other.buildings
    }
}

impl<T: Id> Index<T> for Database {
    type Output = <T as Id>::Info;

    fn index(&self, id: T) -> &Self::Output {
        self.get(id).expect("No such item")
    }
}

/// Trait for symbol types.
pub trait Id:
    fmt::Display + fmt::Debug + Eq + PartialEq + Copy + Clone + Hash + private::Sealed
{
    type Info;

    /// Fetch the item of the correct type with this id from the database.
    fn fetch(self, database: &Database) -> Option<&Self::Info>;
}

macro_rules! typed_symbol {
    ($($(#[$m:meta])*
     $Self:ident {
        info = $info:ident,
        map = $map:ident,
     })+) => {
        $(
            $(#[$m])*
            #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
            #[serde(from = "String", into = "String")]
            pub struct $Self(Intern<String>);

            impl $Self {
                fn as_str(&self) -> &str {
                    &*self.0
                }
            }

            impl Ord for $Self {
                fn cmp(&self, other: &Self) -> Ordering {
                    self.as_str().cmp(other.as_str())
                }
            }

            impl PartialOrd for $Self {
                fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                    Some(self.cmp(other))
                }
            }

            impl From<String> for $Self {
                fn from(id: String) -> Self {
                    Self(Intern::new(id))
                }
            }

            impl From<&str> for $Self {
                fn from(id: &str) -> Self {
                    Self(Intern::from(id))
                }
            }

            impl From<$Self> for String {
                fn from(id: $Self) -> Self {
                    id.as_str().to_owned()
                }
            }

            impl fmt::Display for $Self {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str(self.as_str())
                }
            }

            impl Id for $Self {
                type Info = $info;

                fn fetch(self, database: &Database) -> Option<&Self::Info> {
                    database.$map.get(&self)
                }
            }

            impl private::Sealed for $Self {}
        )+
    };
}

typed_symbol! {
    /// Id of a recipe.
    RecipeId {
        info = Recipe,
        map = recipes,
    }

    /// Id of an item.
    ItemId {
        info = Item,
        map = items,
    }

    BuildingId {
        info = BuildingType,
        map = buildings,
    }
}

/// Recipe for crafting an item or items.
///
/// Recipies sort only by ID.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recipe {
    /// Name of the recipe. Typically similar to the name of the item(s) produced.
    pub name: Rc<str>,
    /// ID of this recipe.
    pub id: RecipeId,
    /// ID of the image for this recipe.
    pub image: Rc<str>,
    /// Time to produce this item at 100% speed, in seconds.
    pub time: f32,
    /// Number and types of ingredients needed for this recipe.
    pub ingredients: Vec<ItemAmount>,
    /// Number and types of products produced by this recipe.
    pub products: Vec<ItemAmount>,
    /// True if this is an alternate recipe.
    pub is_alternate: bool,
    /// Buildings which can produce this recipe.
    pub produced_in: Vec<BuildingId>,
}

/// An input or output: a certain number of items produced or consumed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemAmount {
    /// Id of the item(s).
    pub item: ItemId,
    /// Number of items produced/consumed. Can only be fractional for fluids.
    pub amount: f32,
}

/// A solid or liquid item used in crafting.
///
/// Items sort only by ID.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    /// Name of this item.
    pub name: Rc<str>,
    /// ID of this item.
    pub id: ItemId,
    /// ID of the image for this recipe.
    pub image: Rc<str>,
    /// Description of this item.
    pub description: String,
    /// Fuel settings of this item.
    pub fuel: Option<Fuel>,
    /// Recipes which produce this item.
    pub produced_by: Vec<RecipeId>,
    /// Recipes which consume this item.
    pub consumed_by: Vec<RecipeId>,
    /// Buildings which can mine this item.
    pub mined_by: Vec<BuildingId>,
    /// Speed that this resource is mined at.
    pub mining_speed: f32,
}

/// Settings for an item used as fuel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fuel {
    /// Amount of energy that this item is worth in MJ.
    pub energy: f32,
    /// Byproducts produced from consuming this item as fuel.
    pub byproducts: Vec<ItemAmount>,
}

impl ItemId {
    /// Get the ItemId for water.
    pub fn water() -> Self {
        "Desc_Water_C".into()
    }
}

/// A building used to produce or use items.
///
/// Buildings sort only by ID.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildingType {
    /// Name of the building.
    pub name: Rc<str>,
    /// ID of the building.
    pub id: BuildingId,
    /// ID of the image for this building.
    pub image: Rc<str>,
    /// Description of the building.
    pub description: String,
    /// Kind of the building.
    pub kind: BuildingKind,
}

impl BuildingType {
    /// Gets the settings
    pub fn get_default_settings(&self) -> BuildingSettings {
        self.kind.get_default_settings()
    }
}

/// Which kind of building this is (affects how resources are produced/consumed).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BuildingKind {
    /// Manufacturing buildings consume power to produce outputs according to recipes.
    Manufacturer(Manufacturer),
    /// Miners consume power to produce resources from resource pads.
    Miner(Miner),
    /// Generators produce power by consuming items.
    Generator(Generator),
    /// Pump settings. Pumps are like miners but for resource wells.
    Pump(Pump),
    /// Geothermal generator settings. Geothermal generators sit on resource pad and
    /// produce power but can't be overclocked.
    Geothermal(Geothermal),
    /// General power consumer with no production.
    PowerConsumer(PowerConsumer),
    /// A station which refuels vehicles.
    Station(Station),
}

impl BuildingKind {
    /// Get the ID of this buiilding kind.
    pub fn kind_id(&self) -> BuildingKindId {
        match self {
            Self::Manufacturer(_) => BuildingKindId::Manufacturer,
            Self::Miner(_) => BuildingKindId::Miner,
            Self::Generator(_) => BuildingKindId::Generator,
            Self::Pump(_) => BuildingKindId::Pump,
            Self::Geothermal(_) => BuildingKindId::Geothermal,
            Self::PowerConsumer(_) => BuildingKindId::PowerConsumer,
            Self::Station(_) => BuildingKindId::Station,
        }
    }

    /// Gets the settings for a new building of this kind.
    pub fn get_default_settings(&self) -> BuildingSettings {
        match self {
            BuildingKind::Manufacturer(m) => {
                let mut settings = ManufacturerSettings::default();
                if m.available_recipes.len() == 1 {
                    settings.recipe = m.available_recipes.first().copied();
                }
                BuildingSettings::Manufacturer(settings)
            }
            BuildingKind::Miner(m) => {
                let mut settings = MinerSettings::default();
                if m.allowed_resources.len() == 1 {
                    settings.resource = m.allowed_resources.first().copied();
                }
                BuildingSettings::Miner(settings)
            }
            BuildingKind::Generator(g) => {
                let mut settings = GeneratorSettings::default();
                if g.allowed_fuel.len() == 1 {
                    settings.fuel = g.allowed_fuel.first().copied();
                }
                BuildingSettings::Generator(settings)
            }
            BuildingKind::Pump(p) => {
                let mut settings = PumpSettings::default();
                if p.allowed_resources.len() == 1 {
                    settings.resource = p.allowed_resources.first().copied();
                }
                BuildingSettings::Pump(settings)
            }
            BuildingKind::Geothermal(_) => BuildingSettings::Geothermal(Default::default()),
            BuildingKind::PowerConsumer(_) => BuildingSettings::PowerConsumer,
            BuildingKind::Station(s) => {
                let mut settings = StationSettings::default();
                if s.allowed_fuel.len() == 1 {
                    settings.fuel = s.allowed_fuel.first().copied();
                }
                BuildingSettings::Station(settings)
            }
        }
    }
}

/// Name of a BuildingKind. Used to identify both [`BuildingKind`] and [`BuildingSettings`], essentially the same as BuildingKind but with no data.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub enum BuildingKindId {
    /// Manufacturing buildings consume power to produce outputs according to recipes.
    Manufacturer,
    /// Miners consume power to produce resources from resource pads.
    Miner,
    /// Generators produce power by consuming items.
    Generator,
    /// Pump settings. Pumps are like miners but for resource wells.
    Pump,
    /// Geothermal generator settings. Geothermal generators sit on resource pad and
    /// produce power but can't be overclocked.
    Geothermal,
    /// General power consumer with no production.
    PowerConsumer,
    /// A station which refuels vehicles.
    Station,
}

/// Power-usage information for a building.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Power {
    /// Amount of power used by this building at 100% production, in MW.
    pub power: f32,
    /// Exponent used to adjust power consumption when scaling down or up.
    pub power_exponent: f32,
}

impl Power {
    /// Get the rate of power consumption for these power settings at the given clock
    /// speed.
    pub fn get_consumption_rate(&self, clock_speed: f32) -> f32 {
        self.power * clock_speed.powf(self.power_exponent)
    }

    /// Get the rate of power production for these power settings at the given clock
    /// speed.
    pub fn get_production_rate(&self, clock_speed: f32) -> f32 {
        self.power * clock_speed.powf(1.0 / self.power_exponent)
    }
}

/// Manufacturing settings of a building.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manufacturer {
    /// Multiplier applied to base manufacturing time on recipes.
    pub manufacturing_speed: f32,
    /// Reverse table of available recipes.
    pub available_recipes: Vec<RecipeId>,
    /// Power usage of manufacturing.
    pub power_consumption: Power,
}

/// Miner settings of a building.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Miner {
    /// Items which can be mined.
    pub allowed_resources: Vec<ItemId>,
    /// Number of items to extract per cycle.
    pub items_per_cycle: f32,
    /// Amount of time for each extract cycle.
    pub cycle_time: f32,
    /// Power usage of manufacturing.
    pub power_consumption: Power,
}

/// Generator settings of a building.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Generator {
    /// Recipes this generator type can use.
    pub allowed_fuel: Vec<ItemId>,
    /// Amount of water used per MW of production.
    pub used_water: f32,
    /// Power production of this generator.
    pub power_production: Power,
}

/// Pump settings of a building.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pump {
    /// Recipes this generator type can use.
    pub allowed_resources: Vec<ItemId>,
    /// Number of items to extract per pad, percycle.
    pub items_per_cycle: f32,
    /// Amount of time for each extract cycle.
    pub cycle_time: f32,
    /// Power usage of manufacturing.
    pub power_consumption: Power,
}

/// Geothermal generator settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Geothermal {
    /// Power production. No exponent because overclocking is not possible.
    pub power: f32,
}

/// A general power-consumer which doesn't produce or consume items, just power.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PowerConsumer {
    /// Amount of power consumed.
    pub power: f32,
}

/// A vehicle station which can refuel vehicles which stop at it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Station {
    /// Amount of power consumed.
    pub power: f32,
    /// Allowed fuels for vehicles at this station.
    pub allowed_fuel: Vec<ItemId>,
}

mod private {
    pub trait Sealed {}
}
