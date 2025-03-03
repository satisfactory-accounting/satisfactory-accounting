// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;
use std::hash::Hash;
use std::ops::Index;
use std::rc::{Rc, Weak};

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
    /// V1.0 database versions.
    V1_0(V1_0Subversion),
}

macro_rules! db_version_info {
    ($({
        version: DatabaseVersion::$dbv:ident($dbsv:path),
        file: $file:literal,
        name: $name:literal,
        description: $description:literal $(,)?
    }),* $(,)?) => {
        db_version_info!(@real_impl $({
            version_pat: DatabaseVersion::$dbv($dbsv),
            version_expr: DatabaseVersion::$dbv($dbsv),
            file: $file,
            name: $name,
            description: $description,
        },)*);
    };


    (@real_impl $({
        version_pat: $version_pat:pat,
        version_expr: $version_expr:expr,
        file: $file:literal,
        name: $name:literal,
        description: $description:literal $(,)?
    }),* $(,)?) => {
        /// All released database versions in order.
        pub const ALL: &'static [DatabaseVersion] = &[
            $($version_expr,)*
        ];

        /// Load the database at a particuler version.
        pub fn load_database(self) -> Database {
            match self {
                $(
                    $version_pat => {
                        const SERIALIZED_DB: &str = include_str!($file);
                        thread_local! {
                            static SHARED_INNER: RefCell<Weak<DatabaseInner>> = Default::default();
                        }
                        SHARED_INNER.with_borrow_mut(|shared_inner| {
                            match shared_inner.upgrade() {
                                Some(inner) => Database { inner },
                                None => {
                                    let inner: Rc<DatabaseInner> = serde_json::from_str(SERIALIZED_DB)
                                        .expect(concat!("Failed to parse ", $file));
                                    *shared_inner = Rc::downgrade(&inner);
                                    Database { inner }
                                }
                            }
                        })
                    }
                )*
            }
        }

        /// Get the displayable name for this database version.
        pub const fn name(self) -> &'static str {
            match self {
                $($version_pat => $name,)*
            }
        }

        /// Get the description for this version.
        pub const fn description(self) -> &'static str {
            match self {
                $($version_pat => $description,)*
            }
        }
    };
}

impl DatabaseVersion {
    db_version_info! [
        {
            version: DatabaseVersion::U5(U5Subversion::Initial),
            file: "../db-u5-initial.json",
            name: "U5 \u{2013} Initial",
            description: "This is the first version of the database released for U5. Fuel
                generators in this version consume 1000x too much fuel.",
        },
        {
            version: DatabaseVersion::U5(U5Subversion::Final),
            file: "../db-u5-final.json",
            name: "U5 \u{2013} Final",
            description: "This is the final version of the database released for U5.",
        },
        {
            version: DatabaseVersion::U6(U6Subversion::Beta),
            file: "../db-u6-beta.json",
            name: "U6 \u{2013} Beta",
            description: "This is the first version of the Satisfactory Accounting database \
                released after the U6 update.",
        },
        {
            version: DatabaseVersion::U7(U7Subversion::Initial),
            file: "../db-u7-initial.json",
            name: "U7 \u{2013} Initial",
            description: "This is the first version of the database released for U7.",
        },
        {
            version: DatabaseVersion::V1_0(V1_0Subversion::Initial),
            file: "../db-v1.0-initial.json",
            name: "1.0 \u{2013} Initial",
            description: "This is the first version of the Satisfactory Accounting database \
                released for Satisfactory 1.0. In this version, Water Extractors produce 0 water, \
                and the Resource Well Extractor is a separate building from the Resource Well \
                Pressurizer (it's not supposed to be \u{2013} Resource Wells are handled specially \
                as part of the Pressurizer).",
        },
        {
            version: DatabaseVersion::V1_0(V1_0Subversion::Wetter),
            file: "../db-v1.0-wetter.json",
            name: "1.0 \u{2013} Wetter",
            description: "This minor update to the database for 1.0 fixes Water Extractors so they \
                produce water again and fixes the Resource Well Extractor to be correctly handled \
                as part of the Resource Well Pressurizer rather than as its own separate building.",
        },
        {
            version: DatabaseVersion::V1_0(V1_0Subversion::Semiquantum),
            file: "../db-v1.0-semiquantum.json",
            name: "1.0 \u{2013} Semiquantum",
            description: "This update to the databse for Satisfactory 1.0 adds some recipies that \
                were missing related to late-game technologies, though it doesn't add the Alien \
                Power Augmenter.",
        },
        {
            version: DatabaseVersion::V1_0(V1_0Subversion::Powerline),
            file: "../db-v1.0-powerline.json",
            name: "1.0 \u{2013} Powerline",
            description: "This update to the databse for Satisfactory 1.0 fixes power generators \
                so they scale linearly with changes to their clock speed, which has been how the
                game has worked since U7.",
        },
        {
            version: DatabaseVersion::V1_0(V1_0Subversion::Rocket),
            file: "../db-v1.0-rocket.json",
            name: "1.0 \u{2013} Rocket",
            description: "This update to the databse for Satisfactory 1.0 corrects the production \
                rate of the Nitro Rocket Fuel alternate recipe.",
        },
        {
            version: DatabaseVersion::V1_0(V1_0Subversion::Geo),
            file: "../db-v1.0-geo.json",
            name: "1.0 \u{2013} Geo",
            description: "This update to the databse for Satisfactory 1.0 adds the Geothermal \
                Generator and the new Balance Adjustment node",
        },
    ];

    /// Latest version of the database.
    pub const LATEST: DatabaseVersion = Self::ALL[Self::ALL.len() - 1];

    /// Identifies which database versions are considered deprecated.
    ///
    /// This is mainly used in `satisfactory-accounting-app` to hide versions which have
    /// been made obsolete by newer releases of this tool, for example
    /// [`U5Subversion::Initial`] had incorrect rates of fuel consumption. The latest
    /// release for any major update of the game should be kept not-deprecated (for now),
    /// in case there are players still using that version.
    pub fn is_deprecated(self) -> bool {
        self != Self::LATEST
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

/// Minor versions within the U6 database.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum U6Subversion {
    /// Initial release of U6 for Satisfactory Accounting released in 1.1.0.
    Beta,
}

/// Minor versions within the U7 database.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum U7Subversion {
    /// Initial release of U7 released in Satisfactory Accounting released in 1.2.0.
    Initial,
}

/// Minor versions within the 1.0 database.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum V1_0Subversion {
    /// Initial release of Satisfactory 1.0, released in Satisfactory Accounting 1.2.3.
    Initial,
    /// Update that fixes water extractors, released in Satisfactory Accounting 1.2.5.
    Wetter,
    /// Update adding some missing quantim items, released in Satisfactory Accounting 1.2.6.
    Semiquantum,
    /// Update to make power plant overclock scaling linear.
    Powerline,
    /// Update to fix the production rate of nitro rocket fuel, released in Satisfactory Accounting
    /// 1.2.8.
    Rocket,
    /// Update to add Geothermal and Balance Adjustment.
    Geo,
}

impl fmt::Display for DatabaseVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Database of satisfactory ... stuff.
///
/// This is an rc-based shared type with Cow semantics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Database {
    inner: Rc<DatabaseInner>,
}

impl Database {
    /// Construct a new Database with the given values.
    pub fn new(
        icon_prefix: String,
        recipes: BTreeMap<RecipeId, Recipe>,
        items: BTreeMap<ItemId, Item>,
        buildings: BTreeMap<BuildingId, BuildingType>,
    ) -> Self {
        Self {
            inner: Rc::new(DatabaseInner {
                icon_prefix,
                recipes,
                items,
                buildings,
            }),
        }
    }

    /// Gets an iterator over the buildings in the database.
    pub fn buildings(&self) -> BuildingsIter {
        self.inner.buildings.values()
    }

    /// Gets an iterator over the items in the database.
    pub fn items(&self) -> ItemsIter {
        self.inner.items.values()
    }
}

/// Iterator over the list of available buildings.
pub type BuildingsIter<'a> = std::collections::btree_map::Values<'a, BuildingId, BuildingType>;

/// Iterator over the list of available items.
pub type ItemsIter<'a> = std::collections::btree_map::Values<'a, ItemId, Item>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct DatabaseInner {
    /// Prefix used for static paths for icons in this version of the database.
    #[serde(default)]
    icon_prefix: String,
    /// Core recipe storage. We only store machine recipes.
    recipes: BTreeMap<RecipeId, Recipe>,
    /// Core item storage.
    items: BTreeMap<ItemId, Item>,
    /// Core buildings storage.
    buildings: BTreeMap<BuildingId, BuildingType>,
}

impl Database {
    /// Get an item, recipe, or building by id.
    pub fn get<T: Id>(&self, id: T) -> Option<&<T as Id>::Info> {
        id.fetch(self)
    }

    /// Load the default version of the database.
    pub fn load_latest() -> Database {
        DatabaseVersion::LATEST.load_database()
    }

    /// Compare this database to another database, ignoring their icon prefixes.
    pub fn compare_ignore_prefix(&self, other: &Database) -> bool {
        self.inner.recipes == other.inner.recipes
            && self.inner.items == other.inner.items
            && self.inner.buildings == other.inner.buildings
    }

    /// Prefix used for static paths for icons in this version of the database.
    pub fn icon_prefix(&self) -> &str {
        &self.inner.icon_prefix
    }

    /// Set the icon prefix for this database. Clones self if necessary to prevent shared mutation.
    pub fn set_icon_prefix<S>(&mut self, prefix: S)
    where
        S: Into<String>,
    {
        Rc::make_mut(&mut self.inner).icon_prefix = prefix.into();
    }
}

impl PartialEq for Database {
    fn eq(&self, other: &Self) -> bool {
        // Do a ptr-eq first to optimize the common case where two instances are using the same
        // database by reference.
        if Rc::ptr_eq(&self.inner, &other.inner) {
            return true;
        }
        // If different pointers, fall back on structural equality.
        self.inner == other.inner
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
            #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
            pub struct $Self(Intern<str>);

            impl Serialize for $Self {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: serde::Serializer,
                {
                    serializer.serialize_str(self.as_str())
                }
            }

            impl<'de> Deserialize<'de> for $Self {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: serde::Deserializer<'de>,
                {
                    struct Visitor;
                    impl<'de> serde::de::Visitor<'de> for Visitor {
                        type Value = $Self;

                        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                            f.write_str("a string symbol value")
                        }

                        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                        where E: serde::de::Error,
                        {
                            Ok(Self::Value::from(value))
                        }
                    }
                    deserializer.deserialize_str(Visitor)
                }
            }

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
                    Self(Intern::from(&*id))
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
                    database.inner.$map.get(&self)
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

/// Enum used when you need to refer to either an item or the power.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ItemIdOrPower {
    /// Refers to power.
    Power,
    /// Refers to an item.
    Item(ItemId),
}

impl ItemIdOrPower {
    const POWER_FAKE_ITEM_ID: &str = "_Power_";
}

impl Serialize for ItemIdOrPower {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Power => serializer.serialize_str(Self::POWER_FAKE_ITEM_ID),
            Self::Item(item) => item.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for ItemIdOrPower {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = ItemIdOrPower;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a string symbol value")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if value == ItemIdOrPower::POWER_FAKE_ITEM_ID {
                    Ok(ItemIdOrPower::Power)
                } else {
                    Ok(ItemIdOrPower::Item(ItemId::from(value)))
                }
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}

impl From<ItemId> for ItemIdOrPower {
    fn from(value: ItemId) -> Self {
        Self::Item(value)
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
    /// Byproducts produced from consuming this item as fuel. This amount is in items /
    /// fuel consumed.
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

    /// Return true if this type of building can be overclocked.
    pub fn overclockable(&self) -> bool {
        match &self.kind {
            BuildingKind::Manufacturer(manufacturer) => manufacturer.overclockable(),
            BuildingKind::Miner(miner) => miner.overclockable(),
            BuildingKind::Generator(generator) => generator.overclockable(),
            BuildingKind::Pump(pump) => pump.overclockable(),
            BuildingKind::Geothermal(_) => false,
            BuildingKind::PowerConsumer(_) => false,
            BuildingKind::Station(_) => false,
            BuildingKind::BalanceAdjustment(_) => false,
        }
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
    /// Arbitrary balance change.
    BalanceAdjustment(BalanceAdjustment),
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
            Self::BalanceAdjustment(_) => BuildingKindId::BalanceAdjustment,
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
            BuildingKind::BalanceAdjustment(_) => {
                BuildingSettings::BalanceAdjustment(Default::default())
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
    /// Arbitrary balance change.
    BalanceAdjustment,
}

/// Power-usage information for a building.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Power {
    /// Amount of power used by this building at 100% production, in MW. Always non-negative.
    pub power: f32,
    /// Exponent used to adjust power consumption when scaling down or up. Always non-negative. 0
    /// means not overclockable.
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
        if self.power_exponent == 0.0 {
            return self.power;
        }
        self.power * clock_speed.powf(1.0 / self.power_exponent)
    }

    /// Whether this power rate allows overclocking.
    pub fn overclockable(&self) -> bool {
        self.power_exponent != 0.0
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

impl Manufacturer {
    /// Whether this manufacturer allows overclocking.
    #[inline]
    pub fn overclockable(&self) -> bool {
        self.power_consumption.overclockable()
    }
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

impl Miner {
    /// Whether this miner allows overclocking.
    #[inline]
    pub fn overclockable(&self) -> bool {
        self.power_consumption.overclockable()
    }
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

impl Generator {
    /// Whether this generator allows overclocking.
    #[inline]
    pub fn overclockable(&self) -> bool {
        self.power_production.overclockable()
    }
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

impl Pump {
    /// Whether this pump allows overclocking.
    #[inline]
    pub fn overclockable(&self) -> bool {
        self.power_consumption.overclockable()
    }
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

/// Adjusts the balance of an item or power.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceAdjustment {}

mod private {
    pub trait Sealed {}
}
