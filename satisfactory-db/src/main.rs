// Copyright 2021 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use std::collections::{BTreeMap, HashMap, HashSet};

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
        .chain(["Desc_WaterPump_C".to_string(), "Desc_Portal_C".to_string()])
        .collect();

    let generators: HashMap<_, _> = raw
        .generators
        .values()
        .map(|gen| {
            let building = if gen.class_name.starts_with("Build_") {
                gen.class_name.replace("Build_", "Desc_")
            } else {
                assert!(gen.class_name.starts_with("Desc_"));
                gen.class_name.clone()
            };
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
        "Desc_RocketFuel_C",
        "Desc_IonizedFuel_C",
    ];

    /// As of 1.0 Drones can use any fuel.
    const DRONE_FUELS: &[&str] = TRUCK_FUELS;

    let miners: HashMap<_, _> = raw
        .miners
        .values()
        .map(|min| {
            let building = if min.class_name.starts_with("Build_") {
                min.class_name.replace("Build_", "Desc_")
            } else {
                assert!(min.class_name.starts_with("Desc_"));
                min.class_name.clone()
            };
            assert!(raw.buildings.contains_key(building.as_str()));
            (building, min)
        })
        // Fracking Extractor (Resource Well Extractor) is special cased as part of the resource
        // well pressurizer, which is its own unique building type.
        .filter(|(building, _)| building != "Desc_FrackingExtractor_C")
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
        .chain(TRUCK_FUELS.iter().map(|fuel| fuel.to_string()))
        .chain(DRONE_FUELS.iter().map(|fuel| fuel.to_string()))
        // Extra items which we want to include explicitly.
        .chain([
            // Make sure that water is included
            ItemId::water().into(),
            // Ensure nuclear byproducts are included even if they aren't used in any
            // recipes.
            "Desc_NuclearWaste_C".to_owned(),
            "Desc_PlutoniumWaste_C".to_owned(),
            "Desc_AlienPowerFuel_C".to_owned(),
        ])
        .collect();

    let used_buildings: HashSet<_> = manufacturers
        .iter()
        .cloned()
        .chain(generators.keys().cloned())
        .chain(miners.keys().cloned())
        .chain([
            "Desc_FrackingSmasher_C".to_string(),
            "Desc_AlienPowerBuilding_C".to_string(),
        ])
        .collect();

    let mut recipes: BTreeMap<_, _> = machine_recipes
        .iter()
        .map(|recipe| Recipe {
            name: recipe.name.as_str().into(),
            id: recipe.class_name.as_str().into(),
            image: recipe
                .products
                .iter()
                .next()
                .map(|ia| ia.item.as_str().into())
                .or(recipe
                    .ingredients
                    .iter()
                    .next()
                    .map(|ia| ia.item.as_str().into()))
                .unwrap_or_default(),
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
        // Patch in missing recipes.
        .chain([
            // The water extractor is modeled as a regular manufacturer, using a regular recipe
            // that just has water as its only product with no inputs.
            Recipe {
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
            },
            // Map the Main Portal as a manufacturer that only consumes singularity cells with no
            // products.
            Recipe {
                name: "Power Main Portal".into(),
                id: "_Patch_Recipe_MainPortalCells_C".into(),
                image: "singularity-cell".into(),
                time: 30.0,
                ingredients: vec![ItemAmount {
                    item: "Desc_SingularityCell_C".into(),
                    amount: 1.0,
                }],
                products: Vec::new(),
                is_alternate: false,
                produced_in: vec!["Desc_Portal_C".into()],
            },
        ])
        .map(|recipe| (recipe.id, recipe))
        .collect();

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
                    energy: item.energy_value,
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
        .chain([
            // Alien power matrix seems to be missing.
            Item {
                name: "Alien Power Matrix".into(),
                id: "Desc_AlienPowerFuel_C".into(),
                image: "alien-power-matrix".into(),
                description:
                    "This intricate condensed-matter matrix is used to enhance the output of the \
                    Alien Power Augmenter."
                        .into(),
                fuel: None,
                mining_speed: 0.0,
                produced_by: Vec::new(),
                consumed_by: Vec::new(),
                mined_by: Vec::new(),
            },
        ])
        .map(|item| (item.id, item))
        .collect();

    for recipe in recipes.values_mut() {
        let key: &str = recipe.image.as_ref();
        if let Some(item) = items.get(&key.into()) {
            recipe.image = item.image.clone();
        }
    }

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
                    manufacturing_speed: if building.class_name == "Desc_WaterPump_C"
                        || building.class_name == "Desc_Portal_C"
                    {
                        // In 1.0, the water pump has a manufacturingSpeed of 0 for some reason.
                        // We also need to patch the main portal so it consumes singularity cells.
                        1.0
                    } else {
                        building.metadata.manufacturing_speed.unwrap_or(1.0)
                    },
                    // To be patched in later.
                    available_recipes: Vec::new(),
                    power_consumption: Power {
                        power: if building.class_name.as_str() == "Desc_QuantumEncoder_C" {
                            // The quantum encoder has a power usage of 0, but it actually averages
                            // 1000 MW.
                            1000.0
                        } else {
                            building
                                .metadata
                                .power_consumption
                                .expect("Manufacturer missing power_consumption")
                        },
                        power_exponent: if building.class_name.as_str() == "Desc_Portal_C" {
                            // The main portal is not overclockable, so set its power exponent to 0.
                            0.0
                        } else {
                            building
                                .metadata
                                .power_consumption_exponent
                                .expect("Manufacturer missing power_consumption_exponent")
                        },
                    },
                })
            } else if generators.contains_key(building.class_name.as_str()) {
                // Geothermal is a special case.
                if building.class_name == "Desc_GeneratorGeoThermal_C" {
                    BuildingKind::Geothermal(Geothermal {
                        // Patched from wiki because the data says zero. Based on average
                        // power on a normal node. This should work with node purity to get
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
                            // The powerProductionExponents in the source all still say 1.6, but
                            // since U7, generators have scaled linearly.
                            power_exponent: 1.0,
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
        icon_prefix: "v1.0/".to_string(),
        recipes,
        items,
        buildings,
    };

    serde_json::to_writer_pretty(std::io::stdout().lock(), &database)
        .expect("Unable to write database");
}
