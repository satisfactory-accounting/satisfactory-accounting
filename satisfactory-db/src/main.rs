// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use std::collections::{BTreeMap, HashMap, HashSet};

use regex::Regex;
use satisfactory_accounting::database::{
    BuildingKind, BuildingType, Database, Fuel, Generator, Geothermal, Item, ItemAmount, ItemId,
    Manufacturer, Miner, Power, PowerConsumer, Pump, Recipe, Station,
};

mod rawdata;

fn main() {
    let raw = rawdata::RawData::load();

    let machine_recipes: Vec<_> = raw
        .recipes
        .values()
        .filter(|recipe| recipe.in_machine)
        .cloned()
        .collect();

    let manufacturers: HashSet<_> = machine_recipes
        .iter()
        .flat_map(|recipe| &recipe.produced_in)
        .cloned()
        .chain(std::iter::once("Desc_WaterPump_C".to_string()))
        .collect();

    let generators: HashMap<_, _> = raw
        .generators
        .values()
        .map(|gen| {
            assert!(gen.class_name.starts_with("Build_"));
            let building = gen.class_name.replace("Build_", "Desc_");
            assert!(raw.buildings.contains_key(building.as_str()));
            (building, gen)
        })
        .collect();

    let fuels: HashSet<_> = raw
        .generators
        .values()
        .flat_map(|gen| gen.fuel.iter().cloned())
        .collect();

    /// Leaves, Flower Petals, Wood, Mycelia, Fabric,
    /// Alien Carapace, Alien Organs, Color Cartridge,
    /// Biomass, Solid Biofuel, Packaged Liquid Biofuel,
    /// Coal, Compacted Coal, Petroleum Coke, Packaged Oil,
    /// Packaged Heavy Oil Residue, Packaged Fuel, Packaged Turbofuel,
    /// Battery, Uranium Fuel Rod, Plutonium Fuel Rod
    const TRUCK_FUELS: &[&str] = &[
        "Desc_Leaves_C",
        "Desc_FlowerPetals_C",
        "Desc_Wood_C",
        "Desc_Mycelia_C",
        "Desc_Fabric_C",
        "Desc_HogParts_C",
        "Desc_SpitterParts_C",
        "Desc_ColorCartridge_C",
        "Desc_GenericBiomass_C",
        "Desc_Biofuel_C",
        "Desc_PackagedBiofuel_C",
        "Desc_Coal_C",
        "Desc_CompactedCoal_C",
        "Desc_PetroleumCoke_C",
        "Desc_PackagedOil_C",
        "Desc_PackagedOilResidue_C",
        "Desc_Fuel_C",
        "Desc_TurboFuel_C",
        "Desc_Battery_C",
        "Desc_NuclearFuelRod_C",
        "Desc_PlutoniumFuelRod_C",
    ];

    /// Battery
    const DRONE_FUELS: &[&str] = &["Desc_Battery_C"];

    let miners: HashMap<_, _> = raw
        .miners
        .values()
        .map(|min| {
            assert!(min.class_name.starts_with("Build_"));
            let building = min.class_name.replace("Build_", "Desc_");
            assert!(raw.buildings.contains_key(building.as_str()));
            (building, min)
        })
        .collect();

    let used_items: HashSet<_> = machine_recipes
        .iter()
        // Items used in or produced by machine recipes.
        .flat_map(|recipe| recipe.ingredients.iter().chain(recipe.products.iter()))
        .map(|ia| ia.item.clone())
        // Items that can be extracted by miners.
        .chain(raw.resources.keys().cloned())
        // Fuels for generators.
        .chain(fuels.iter().cloned())
        // Special case to make sure water is included.
        .chain(std::iter::once(ItemId::water().into()))
        .chain(TRUCK_FUELS.iter().map(|fuel| fuel.to_string()))
        .chain(DRONE_FUELS.iter().map(|fuel| fuel.to_string()))
        .collect();

    let used_buildings: HashSet<_> = manufacturers
        .iter()
        .cloned()
        .chain(generators.keys().cloned())
        .chain(miners.keys().cloned())
        .chain(std::iter::once("Desc_FrackingSmasher_C".to_string()))
        .collect();

    let bad_icon_names = Regex::new(r"-\(.*\)").unwrap();
    let recipes: BTreeMap<_, _> = machine_recipes
        .iter()
        .map(|recipe| Recipe {
            name: recipe.name.as_str().into(),
            id: recipe.class_name.as_str().into(),
            image: if let Some(nonresidual) = recipe.slug.strip_prefix("residual-") {
                nonresidual.into()
            } else {
                bad_icon_names.replace(recipe.slug.as_str(), "").into()
            },
            time: recipe.time,
            ingredients: recipe
                .ingredients
                .iter()
                .map(|ia| ItemAmount {
                    item: ia.item.as_str().into(),
                    amount: ia.amount,
                })
                .collect(),
            products: recipe
                .products
                .iter()
                .map(|ia| ItemAmount {
                    item: ia.item.as_str().into(),
                    amount: ia.amount,
                })
                .collect(),
            is_alternate: recipe.alternate,
            produced_in: recipe
                .produced_in
                .iter()
                .map(|machine| machine.as_str().into())
                .collect(),
        })
        // Patch a recipe for water using the water extractor.
        .chain(std::iter::once(Recipe {
            name: "Extract Water".into(),
            id: "_Patch_Recipe_ExtractWater_C".into(),
            image: "water".into(),
            time: 0.5,
            ingredients: Vec::new(),
            products: vec![ItemAmount {
                item: ItemId::water(),
                amount: 1.0,
            }],
            is_alternate: false,
            produced_in: vec!["Desc_WaterPump_C".into()],
        }))
        .map(|recipe| (recipe.id, recipe))
        .collect();

    const LIQUID_FUELS: &[&str] = &[
        "Desc_LiquidBiofuel_C",
        "Desc_LiquidFuel_C",
        "Desc_LiquidOil_C",
        "Desc_LiquidTurboFuel_C",
    ];

    let mut items: BTreeMap<_, _> = raw
        .items
        .values()
        .filter(|item| used_items.contains(item.class_name.as_str()))
        .map(|item| Item {
            name: item.name.as_str().into(),
            id: item.class_name.as_str().into(),
            image: item.slug.as_str().into(),
            description: item.description.clone(),
            fuel: if fuels.contains(item.class_name.as_str()) {
                Some(Fuel {
                    // For some reason liquid fuels are inconsistently treated as if they
                    // are in fractional (1/1000th) units instead of whole units by the
                    // source DB. E.g. the recipe for packaging uses whole units (2 liquid
                    // fuel -> 2 packaged fuel), but the oil extractor's extraction rate
                    // and the state energy content are based on 1/1000th unit of fuel, so
                    // we have to multiply the energy content (and see below for where we
                    // divide the production output of the oil extractor).
                    energy: if LIQUID_FUELS.contains(&item.class_name.as_str()) {
                        item.energy_value * 1000.0
                    } else {
                        item.energy_value
                    },
                    // Patch in nuclear byproducts.
                    byproducts: match item.class_name.as_str() {
                        "Desc_NuclearFuelRod_C" => vec![ItemAmount {
                            item: "Desc_NuclearWaste_C".into(),
                            amount: 50.0,
                        }],
                        "Desc_PlutoniumFuelRod_C" => vec![ItemAmount {
                            item: "Desc_PlutoniumWaste_C".into(),
                            amount: 10.0,
                        }],
                        _ => Vec::new(),
                    },
                })
            } else {
                None
            },
            mining_speed: if raw.resources.contains_key(item.class_name.as_str()) {
                raw.resources[item.class_name.as_str()].speed
            } else {
                0.0
            },
            // These will be patched in later.
            produced_by: Vec::new(),
            consumed_by: Vec::new(),
            mined_by: Vec::new(),
        })
        .map(|item| (item.id, item))
        .collect();

    let mut buildings: BTreeMap<_, _> = raw
        .buildings
        .values()
        .filter(|building| {
            used_buildings.contains(building.class_name.as_str())
                || matches!(building.metadata.power_consumption, Some(power) if power > 0.0)
        })
        .map(|building| BuildingType {
            name: building.name.as_str().into(),
            id: building.class_name.as_str().into(),
            image: building.slug.as_str().into(),
            description: building.description.clone(),
            kind: if manufacturers.contains(building.class_name.as_str()) {
                BuildingKind::Manufacturer(Manufacturer {
                    manufacturing_speed: building.metadata.manufacturing_speed.unwrap_or(1.0),
                    // To be patched in later.
                    available_recipes: Vec::new(),
                    power_consumption: Power {
                        power: building
                            .metadata
                            .power_consumption
                            .expect("Manufacturer missing power_consumption"),
                        power_exponent: building
                            .metadata
                            .power_consumption_exponent
                            .expect("Manufacturer missing power_consumption_exponent"),
                    },
                })
            } else if generators.contains_key(building.class_name.as_str()) {
                // Geothermal is a special case.
                if building.class_name == "Desc_GeneratorGeoThermal_C" {
                    BuildingKind::Geothermal(Geothermal {
                        // Patched from wiki because the data says zero. Based on average
                        // power on a normal node. This should work with nod purity to get
                        // the right averages.
                        power: 200.0,
                    })
                } else {
                    let gen = generators[building.class_name.as_str()];
                    BuildingKind::Generator(Generator {
                        allowed_fuel: gen.fuel.iter().map(|fuel| fuel.as_str().into()).collect(),
                        // Patched directly because the waterToPowerRatio in the data
                        // makes no sense to me.
                        used_water: match building.class_name.as_str() {
                            "Desc_GeneratorCoal_C" => 45.0 / 75.0,
                            "Desc_GeneratorNuclear_C" => 300.0 / 2500.0,
                            _ => 0.0,
                        },
                        power_production: Power {
                            power: gen.power_production,
                            power_exponent: gen.power_production_exponent,
                        },
                    })
                }
            } else if miners.contains_key(building.class_name.as_str()) {
                let min = miners[building.class_name.as_str()];
                BuildingKind::Miner(Miner {
                    allowed_resources: min
                        .allowed_resources
                        .iter()
                        .map(|res| res.as_str().into())
                        .collect(),
                    items_per_cycle: if building.class_name.as_str() == "Desc_OilPump_C" {
                        min.items_per_cycle / 1000.0
                    } else {
                        min.items_per_cycle
                    },
                    cycle_time: min.extract_cycle_time,
                    power_consumption: Power {
                        power: building
                            .metadata
                            .power_consumption
                            .expect("Miner missing power consumption"),
                        power_exponent: building
                            .metadata
                            .power_consumption_exponent
                            .expect("Miner missing power consumption exponent"),
                    },
                })
            } else if building.class_name == "Desc_FrackingSmasher_C" {
                BuildingKind::Pump(Pump {
                    allowed_resources: vec![
                        "Desc_LiquidOil_C".into(),
                        "Desc_NitrogenGas_C".into(),
                        ItemId::water(),
                    ],
                    items_per_cycle: 1.0,
                    cycle_time: 1.0,
                    power_consumption: Power {
                        power: building
                            .metadata
                            .power_consumption
                            .expect("Pump missing power consumption"),
                        power_exponent: building
                            .metadata
                            .power_consumption_exponent
                            .expect("Pump missing power consumption exponent"),
                    },
                })
            } else if building.class_name == "Desc_TruckStation_C" {
                BuildingKind::Station(Station {
                    allowed_fuel: TRUCK_FUELS.iter().map(|&fuel| fuel.into()).collect(),
                    power: building
                        .metadata
                        .power_consumption
                        .expect("Power consumer missing power consumption"),
                })
            } else if building.class_name == "Desc_DroneStation_C" {
                BuildingKind::Station(Station {
                    allowed_fuel: DRONE_FUELS.iter().map(|&fuel| fuel.into()).collect(),
                    power: building
                        .metadata
                        .power_consumption
                        .expect("Power consumer missing power consumption"),
                })
            } else {
                BuildingKind::PowerConsumer(PowerConsumer {
                    power: building
                        .metadata
                        .power_consumption
                        .expect("Power consumer missing power consumption"),
                })
            },
        })
        .map(|building| (building.id, building))
        .collect();

    for recipe in recipes.values() {
        for input in &recipe.ingredients {
            items
                .get_mut(&input.item)
                .expect("Missing item used in recipe")
                .consumed_by
                .push(recipe.id);
        }
        for output in &recipe.products {
            items
                .get_mut(&output.item)
                .expect("Missing item produced in recipe")
                .produced_by
                .push(recipe.id);
        }
        for building in &recipe.produced_in {
            match &mut buildings
                .get_mut(building)
                .expect("Missing building used by recipe")
                .kind
            {
                BuildingKind::Manufacturer(m) => m.available_recipes.push(recipe.id),
                kind => panic!(
                    "Recipe {} allows building {} which is a {:?} not Manufacturer",
                    recipe.id,
                    building,
                    kind.kind_id(),
                ),
            }
        }
    }
    for building in buildings.values() {
        match &building.kind {
            BuildingKind::Miner(m) => {
                for item in &m.allowed_resources {
                    items
                        .get_mut(&item)
                        .expect("Missing resource extracted by building")
                        .mined_by
                        .push(building.id);
                }
            }
            BuildingKind::Pump(p) => {
                for item in &p.allowed_resources {
                    items
                        .get_mut(&item)
                        .expect("Missing resource extracted by building")
                        .mined_by
                        .push(building.id);
                }
            }
            _ => {}
        }
    }

    let database = Database {
        icon_prefix: "u5/".to_string(),
        recipes,
        items,
        buildings,
    };

    serde_json::to_writer_pretty(std::io::stdout().lock(), &database)
        .expect("Unable to write database");
}
