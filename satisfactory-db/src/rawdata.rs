use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub(crate) struct Color {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
    pub(crate) a: u8,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub(crate) struct Size {
    pub(crate) width: Option<f32>,
    pub(crate) length: Option<f32>,
    pub(crate) height: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ItemAmount {
    pub(crate) item: String,
    pub(crate) amount: f32,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RawData {
    pub(crate) recipes: HashMap<String, Recipe>,
    pub(crate) items: HashMap<String, Item>,
    pub(crate) schematics: HashMap<String, Schematic>,
    pub(crate) generators: HashMap<String, Generator>,
    pub(crate) resources: HashMap<String, Resource>,
    pub(crate) miners: HashMap<String, Miner>,
    pub(crate) buildings: HashMap<String, Building>,
}

impl RawData {
    pub(crate) fn load() -> Self {
        const RAW_DATA: &str = include_str!("../data.json");
        serde_json::from_str(RAW_DATA).expect("failed to parse default data")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Recipe {
    pub(crate) slug: String,
    pub(crate) name: String,
    pub(crate) class_name: String,
    pub(crate) alternate: bool,
    pub(crate) time: f32,
    pub(crate) manual_time_multiplier: f32,
    pub(crate) ingredients: Vec<ItemAmount>,
    pub(crate) for_building: bool,
    pub(crate) in_machine: bool,
    pub(crate) in_hand: bool,
    pub(crate) in_workshop: bool,
    pub(crate) products: Vec<ItemAmount>,
    pub(crate) produced_in: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Item {
    pub(crate) slug: String,
    pub(crate) class_name: String,
    pub(crate) name: String,
    pub(crate) sink_points: Option<u32>,
    pub(crate) description: String,
    pub(crate) stack_size: u32,
    pub(crate) energy_value: f32,
    pub(crate) radioactive_decay: f32,
    pub(crate) liquid: bool,
    pub(crate) fluid_color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Unlock {
    pub(crate) inventory_slots: u32,
    pub(crate) recipes: Vec<String>,
    pub(crate) scanner_resources: Vec<String>,
    pub(crate) give_items: Vec<ItemAmount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Generator {
    pub(crate) class_name: String,
    pub(crate) fuel: Vec<String>,
    pub(crate) power_production: f32,
    pub(crate) power_production_exponent: f32,
    pub(crate) water_to_power_ratio: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Resource {
    pub(crate) item: String,
    pub(crate) ping_color: Color,
    pub(crate) speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Miner {
    pub(crate) class_name: String,
    pub(crate) allowed_resources: Vec<String>,
    pub(crate) items_per_cycle: f32,
    pub(crate) extract_cycle_time: f32,
    pub(crate) allow_liquids: bool,
    pub(crate) allow_solids: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Building {
    pub(crate) slug: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) categories: Vec<String>,
    pub(crate) build_menu_priority: f32,
    pub(crate) class_name: String,
    pub(crate) metadata: BuildingMetadata,
    pub(crate) size: Size,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
