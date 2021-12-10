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
pub struct Database {
    /// Core recipe storage. We only store machine recipes.
    recipes: HashMap<RecipeId, Recipe>,
    /// Core item storage.
    items: HashMap<ItemId, Item>,
}

static DATABASE: Lazy<Database> = Lazy::new(|| Database::new(RawData::load()));

impl Database {
    fn new(raw_data: RawData) -> Self {
        let recipes: HashMap<_, _> = raw_data
            .recipes
            .into_values()
            .filter(|recipe| recipe.in_machine)
            .map(|recipe| Recipe {
                name: recipe.name,
                id: recipe.class_name,
                image: recipe.slug,
                time: recipe.time,
                ingredients: recipe
                    .ingredients
                    .into_iter()
                    .map(|ingredient| ItemAmount {
                        item: ingredient.item,
                        amount: ingredient.amount,
                    })
                    .collect(),
                products: recipe
                    .products
                    .into_iter()
                    .map(|product| ItemAmount {
                        item: product.item,
                        amount: product.amount,
                    })
                    .collect(),
            })
            .map(|recipe| (recipe.id, recipe))
            .collect();

        let used_items: HashSet<_> = recipes
            .values()
            .flat_map(|recipe| recipe.ingredients.iter().chain(recipe.products.iter()))
            .map(|item_amount| item_amount.item)
            .collect();

        let mut items: HashMap<_, _> = raw_data
            .items
            .into_values()
            .map(|item| Item {
                name: item.name,
                id: item.class_name,
                image: item.slug,
                description: item.description,
                transport: ItemTransport::from_is_liquid(item.liquid),
                produced_by: Vec::new(),
                consumed_by: Vec::new(),
            })
            .filter(|item| used_items.contains(&item.id))
            .map(|item| (item.id, item))
            .collect();

        // Add the back mappings from ingredients to recipes.
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
        }

        Database { recipes, items }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_db() {
        Database::instance();
    }
}
