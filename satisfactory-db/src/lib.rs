use std::collections::{HashMap, HashSet};
use std::fmt;
use std::iter::FusedIterator;
use std::ops::Index;

use internment::Intern;
use once_cell::sync::Lazy;
use rawdata::RawData;
use serde::{Deserialize, Serialize};

mod rawdata;

/// Database of satisfactory ... stuff.
#[derive(Debug, Serialize, Deserialize)]
pub struct Database {
    /// Core recipe storage. We only store machine recipes.
    recipes: HashMap<RecipeId, Recipe>,
    /// Core item storage.
    items: HashMap<ItemId, Item>,
    /// Core buildings storage.
    buildings: HashMap<BuildingId, Building>,
}

static DATABASE: Lazy<Database> = Lazy::new(|| Database::load());

/// Make synthetic extraction recipes for the given items.
fn make_synth_extract_recipes(resource: ItemId, raw_item: &rawdata::Item, impure_time: f32) -> impl Iterator<Item = Recipe> {
}

impl Database {
    fn load() -> Self {
        let raw_data = RawData::load();
        // First get the base recipes.
        let mut recipes: HashMap<_, _> = raw_data
            .recipes
            .into_values()
            .filter(|recipe| recipe.in_machine)
            .map(Recipe::from)
            .map(|recipe| (recipe.id, recipe))
            .collect();

        const MINERS: &[&str] = &["Desc_MinerMk1_C", "Desc_MinerMk2_C", "Desc_MinerMk3_C"];
        for (resource, impure_time, extractors) in [
            ("Desc_Coal_C", 2.0, MINERS),
            ("Desc_OreBauxite_C", 2.0, MINERS),
            ("Desc_OreCopper_C", 2.0, MINERS),
            ("Desc_OreGold_C", 2.0, MINERS),
            ("Desc_OreIron_C", 2.0, MINERS),
            ("Desc_OreUranium_C", 2.0, MINERS),
            ("Desc_RawQuartz_C", 2.0, MINERS),
            ("Desc_Stone_C", 2.0, MINERS),
            ("Desc_Sulfur_C", 2.0, MINERS),
        ] {
            let resource = ItemId::from(resource);
            let raw_item = &raw_data.items[&resource];
            recipes.extend(make_synth_extract_recipes(resource, raw_item).map(|recipe| (recipe.id, recipe)));
        }

        let used_items: HashSet<_> = recipes
            .values()
            .flat_map(|recipe| recipe.ingredients.iter().chain(recipe.products.iter()))
            .map(|item_amount| item_amount.item)
            .collect();

        let used_buildings: HashSet<_> = recipes
            .values()
            .flat_map(|recipe| recipe.produced_in.iter().copied())
            .collect();

        let mut items: HashMap<_, _> = raw_data
            .items
            .into_values()
            .filter(|item| used_items.contains(&item.class_name))
            .map(Item::from)
            .map(|item| (item.id, item))
            .collect();

        let mut buildings: HashMap<_, _> = raw_data
            .buildings
            .into_values()
            .filter(|building| used_buildings.contains(&building.class_name))
            .map(Building::from)
            .map(|building| (building.id, building))
            .collect();

        // Add the back mappings from ingredients and buildings to recipes.
        for recipe in recipes.values() {
            for ingredient in recipe.ingredients.iter() {
                match items.get_mut(&ingredient.item) {
                    Some(item) => item.consumed_by.push(recipe.id),
                    None => panic!(
                        "Recipe {} consumes Item {} which is not defined",
                        recipe.id, ingredient.item,
                    ),
                }
            }
            for product in recipe.products.iter() {
                match items.get_mut(&product.item) {
                    Some(item) => item.produced_by.push(recipe.id),
                    None => panic!(
                        "Recipe {} consumes Item {} which is not defined",
                        recipe.id, product.item,
                    ),
                }
            }
            for producer in recipe.produced_in.iter() {
                match buildings.get_mut(&producer) {
                    Some(building) => building.available_recipes.push(recipe.id),
                    None => panic!(
                        "Recipe {} can be produced in Building {} which is not defined",
                        recipe.id, producer,
                    ),
                }
            }
        }

        Database {
            recipes,
            items,
            buildings,
        }
    }

    /// Get the global database instance.
    pub fn instance() -> &'static Self {
        &DATABASE
    }

    pub fn get<T: Id>(&self, id: T) -> Option<&<T as Id>::Info> {
        id.fetch(self)
    }

    /// Get an iterator over the complete set of recipes.
    pub fn recipes(&self) -> impl Iterator<Item = &Recipe> + ExactSizeIterator + FusedIterator {
        self.recipes.values()
    }

    /// Get an iterator over the complete set of recipes.
    pub fn items(&self) -> impl Iterator<Item = &Item> + ExactSizeIterator + FusedIterator {
        self.items.values()
    }
}

impl<T: Id> Index<T> for Database {
    type Output = <T as Id>::Info;

    fn index(&self, id: T) -> &Self::Output {
        self.get(id).expect("No such item")
    }
}

/// Trait for symbol types.
pub trait Id {
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
        info = Building,
        map = buildings,
    }
}

/// Recipe for crafting an item or items.
#[derive(Debug, Serialize, Deserialize)]
pub struct Recipe {
    /// Name of the recipe. Typically similar to the name of the item(s) produced.
    pub name: String,
    /// ID of this recipe.
    pub id: RecipeId,
    /// ID of the image for this recipe.
    pub image: String,
    /// Time to produce this item at 100% speed.
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

impl From<rawdata::Recipe> for Recipe {
    fn from(recipe: rawdata::Recipe) -> Self {
        Self {
            name: recipe.name,
            id: recipe.class_name,
            image: recipe.slug,
            time: recipe.time,
            ingredients: recipe.ingredients,
            products: recipe.products,
            is_alternate: recipe.alternate,
            produced_in: recipe.produced_in,
        }
    }
}

/// An input or output: a certain number of items produced or consumed.
#[derive(Debug, Serialize, Deserialize)]
pub struct ItemAmount {
    /// Id of the item(s).
    pub item: ItemId,
    /// Number of items produced/consumed. Can only be fractional for fluids.
    pub amount: f32,
}

/// Which kind of item this is (how it is transported).
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ItemTransport {
    /// Item is a solid, moved on conveyor belts.
    Solid,
    /// Item is a liquid, moved through pipes.
    Liquid,
}

impl ItemTransport {
    fn from_is_liquid(is_liquid: bool) -> Self {
        if is_liquid {
            Self::Liquid
        } else {
            Self::Solid
        }
    }
}

/// A solid or liquid item used in crafting.
#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    /// Name of this item.
    pub name: String,
    /// ID of this item.
    pub id: ItemId,
    /// ID of the image for this recipe.
    pub image: String,
    /// Description of this item.
    pub description: String,
    /// Whether this item is a liquid
    pub transport: ItemTransport,
    /// Recipes which produce this item.
    pub produced_by: Vec<RecipeId>,
    /// Recipes which consume this item.
    pub consumed_by: Vec<RecipeId>,
}

impl From<rawdata::Item> for Item {
    /// Build an Item from a RawItem. Does not fill in produced_by or consumed_by.
    fn from(item: rawdata::Item) -> Self {
        Self {
            name: item.name,
            id: item.class_name,
            image: item.slug,
            description: item.description,
            transport: ItemTransport::from_is_liquid(item.liquid),
            produced_by: Vec::new(),
            consumed_by: Vec::new(),
        }
    }
}

/// A building used to produce recipes.
#[derive(Debug, Serialize, Deserialize)]
pub struct Building {
    /// Name of the building.
    pub name: String,
    /// ID of the building.
    pub id: BuildingId,
    /// ID of the image for this building.
    pub image: String,
    /// Description of the building.
    pub description: String,
    /// Amount of power used by this building at 100% production.
    pub power_consumption: f32,
    /// Exponent used to adjust power consumption when scaling down or up.
    pub power_consumption_exponent: f32,
    /// Unclear. Multiplier applied to base manufacturing time on recipes?
    pub manufacturing_speed: f32,
    /// Reverse table of available recipes.
    pub available_recipes: Vec<RecipeId>,
}

impl From<rawdata::Building> for Building {
    fn from(building: rawdata::Building) -> Self {
        Self {
            name: building.name,
            id: building.class_name,
            image: building.slug,
            description: building.description,
            power_consumption: building.metadata.power_consumption.unwrap_or(0.0),
            power_consumption_exponent: building.metadata.power_consumption_exponent.unwrap_or(1.0),
            manufacturing_speed: building.metadata.manufacturing_speed.unwrap_or(1.0),
            available_recipes: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_db() {
        Database::instance();
    }
}
