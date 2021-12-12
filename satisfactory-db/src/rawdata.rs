use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{BuildingId, ItemAmount, ItemId, RecipeId};

#[derive(Serialize, Deserialize)]
pub(crate) struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Size {
    width: Option<f32>,
    length: Option<f32>,
    height: Option<f32>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct RawData {
    pub(crate) recipes: HashMap<RecipeId, Recipe>,
    pub(crate) items: HashMap<ItemId, Item>,
    pub(crate) schematics: HashMap<String, Schematic>,
    pub(crate) generators: HashMap<String, Generator>,
    pub(crate) resources: HashMap<ItemId, Resource>,
    pub(crate) miners: HashMap<String, Miner>,
    pub(crate) buildings: HashMap<BuildingId, Building>,
}

impl RawData {
    pub(crate) fn load() -> Self {
        const RAW_DATA: &str = include_str!("../data.json");
        let mut data: RawData = serde_json::from_str(RAW_DATA).expect("failed to parse default data");

        // Patch extra fields.
        data.buildings
            .get_mut(&"Desc_MinerMk1_C".into())
            .unwrap()
            .metadata
            .manufacturing_speed = Some(1.0);
        data.buildings
            .get_mut(&"Desc_MinerMk2_C".into())
            .unwrap()
            .metadata
            .manufacturing_speed = Some(1.0);
        data.buildings
            .get_mut(&"Desc_MinerMk3_C".into())
            .unwrap()
            .metadata
            .manufacturing_speed = Some(1.0);

        data
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Recipe {
    pub(crate) slug: String,
    pub(crate) name: String,
    pub(crate) class_name: RecipeId,
    pub(crate) alternate: bool,
    pub(crate) time: f32,
    pub(crate) manual_time_multiplier: f32,
    pub(crate) ingredients: Vec<ItemAmount>,
    pub(crate) for_building: bool,
    pub(crate) in_machine: bool,
    pub(crate) in_hand: bool,
    pub(crate) in_workshop: bool,
    pub(crate) products: Vec<ItemAmount>,
    pub(crate) produced_in: Vec<BuildingId>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Item {
    pub(crate) slug: String,
    pub(crate) class_name: ItemId,
    pub(crate) name: String,
    pub(crate) sink_points: Option<u32>,
    pub(crate) description: String,
    pub(crate) stack_size: u32,
    pub(crate) energy_value: f32,
    pub(crate) radioactive_decay: f32,
    pub(crate) liquid: bool,
    pub(crate) fluid_color: Color,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Schematic {
    pub(crate) class_name: String,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) tier: u32,
    pub(crate) cost: Vec<ItemAmount>,
    pub(crate) unlock: Unlock,
    pub(crate) required_schematics: Vec<String>,
    pub(crate) r#type: String,
    pub(crate) time: f32,
    pub(crate) alternate: bool,
    pub(crate) mam: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Unlock {
    pub(crate) inventory_slots: u32,
    pub(crate) recipes: Vec<RecipeId>,
    pub(crate) scanner_resources: Vec<String>,
    pub(crate) give_items: Vec<ItemAmount>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Generator {
    pub(crate) class_name: String,
    pub(crate) fuel: Vec<ItemId>,
    pub(crate) power_production: f32,
    pub(crate) power_production_exponent: f32,
    pub(crate) water_to_power_ratio: f32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Resource {
    pub(crate) item: ItemId,
    pub(crate) ping_color: Color,
    pub(crate) speed: f32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Miner {
    pub(crate) class_name: String,
    pub(crate) allowed_resources: Vec<ItemId>,
    pub(crate) items_per_cycle: f32,
    pub(crate) extract_cycle_time: f32,
    pub(crate) allow_liquids: bool,
    pub(crate) allow_solids: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Building {
    pub(crate) slug: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) categories: Vec<String>,
    pub(crate) build_menu_priority: f32,
    pub(crate) class_name: BuildingId,
    pub(crate) metadata: BuildingMetadata,
    pub(crate) size: Size,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BuildingMetadata {
    pub(crate) power_consumption: Option<f32>,
    pub(crate) power_consumption_exponent: Option<f32>,
    pub(crate) manufacturing_speed: Option<f32>,
    pub(crate) max_length: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_data() {
        RawData::load();
    }
}
