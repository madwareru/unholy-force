use crate::{
    assets::{AssetDb, AssetKind},
    game_config::ConfigId,
    game_config::floor_part_adjacency::FloorPartAdjacencyConfig,
    game_config::floor_parts::FloorPartConfig,
    game_config::floors::{
        AuthoredFloorSize15x15, AuthoredFloorSize20x20, AuthoredFloorSize25x25,
        AuthoredFloorSize30x30, FloorConfig, FloorSize40x40, FloorSize60x60, FloorSize80x80,
        FloorVariant, GeneratedFloor, GeneratedFloorSize,
    },
};
use simple_tiled_wfc::grid_generation::{WfcContext, WfcContextBuilder, WfcModule};
use std::collections::HashMap;
use std::sync::mpsc::channel;

pub enum FloorGeneratorResult {
    Size15x15(AuthoredFloorSize15x15),
    Size20x20(AuthoredFloorSize20x20),
    Size25x25(AuthoredFloorSize25x25),
    Size30x30(AuthoredFloorSize30x30),
    Size40x40(FloorSize40x40),
    Size60x60(FloorSize60x60),
    Size80x80(FloorSize80x80),
}

pub fn generate(
    asset_db: &AssetDb,
    floor_config: ConfigId<FloorConfig>,
) -> Option<FloorGeneratorResult> {
    if !asset_db.has_asset(AssetKind::FloorConfig, floor_config.uuid) {
        return None;
    }

    let config_text = asset_db.load_json5_asset(AssetKind::FloorConfig, floor_config.uuid);
    let config: FloorConfig = json5::from_str(config_text).expect("Failed to parse FloorConfig");

    match config.floor_variant {
        FloorVariant::Authored15x15(data) => Some(FloorGeneratorResult::Size15x15(data)),
        FloorVariant::Authored20x20(data) => Some(FloorGeneratorResult::Size20x20(data)),
        FloorVariant::Authored25x25(data) => Some(FloorGeneratorResult::Size25x25(data)),
        FloorVariant::Authored30x30(data) => Some(FloorGeneratorResult::Size30x30(data)),
        floor_var => {
            let mut adjacency_configs = Vec::new();
            for fpa in config.available_parts.iter() {
                if !asset_db.has_asset(AssetKind::FloorPartAdjacencyConfig, fpa.uuid) {
                    continue;
                }

                let config_text =
                    asset_db.load_json5_asset(AssetKind::FloorPartAdjacencyConfig, fpa.uuid);

                let config: FloorPartAdjacencyConfig =
                    json5::from_str(config_text).expect("Failed to parse FloorPartAdjacencyConfig");

                if !asset_db.has_asset(AssetKind::FloorPartConfig, config.part.uuid) {
                    continue;
                }

                adjacency_configs.push(config);
            }

            if adjacency_configs.is_empty() {
                return None;
            }

            type CustomBitSet = [u8; 32];
            let mut modules: Vec<WfcModule<CustomBitSet>> = Vec::new();
            let mut uuid_mapping: HashMap<ConfigId<FloorPartConfig>, usize> = HashMap::new();
            let mut module_ids = Vec::new();

            // В данном проходе мы просто собираем ссылки и
            // создаём на каждую уникальную ссылку модуль:
            for config in adjacency_configs.iter() {
                if !uuid_mapping.contains_key(&config.part) {
                    uuid_mapping.insert(config.part, modules.len());
                    modules.push(WfcModule::new());
                    module_ids.push(config.part);
                }

                for part_id in config.north_adjacent_parts.iter() {
                    if !asset_db.has_asset(AssetKind::FloorPartConfig, part_id.uuid) {
                        continue;
                    }

                    if !uuid_mapping.contains_key(part_id) {
                        uuid_mapping.insert(*part_id, modules.len());
                        modules.push(WfcModule::new());
                        module_ids.push(*part_id);
                    }
                }

                for part_id in config.south_adjacent_parts.iter() {
                    if !asset_db.has_asset(AssetKind::FloorPartConfig, part_id.uuid) {
                        continue;
                    }

                    if !uuid_mapping.contains_key(part_id) {
                        uuid_mapping.insert(*part_id, modules.len());
                        modules.push(WfcModule::new());
                        module_ids.push(*part_id);
                    }
                }

                for part_id in config.west_adjacent_parts.iter() {
                    if !asset_db.has_asset(AssetKind::FloorPartConfig, part_id.uuid) {
                        continue;
                    }

                    if !uuid_mapping.contains_key(part_id) {
                        uuid_mapping.insert(*part_id, modules.len());
                        modules.push(WfcModule::new());
                        module_ids.push(*part_id);
                    }
                }

                for part_id in config.east_adjacent_parts.iter() {
                    if !asset_db.has_asset(AssetKind::FloorPartConfig, part_id.uuid) {
                        continue;
                    }

                    if !uuid_mapping.contains_key(part_id) {
                        uuid_mapping.insert(*part_id, modules.len());
                        modules.push(WfcModule::new());
                        module_ids.push(*part_id);
                    }
                }
            }

            for config in adjacency_configs.iter() {
                for part_id in config.north_adjacent_parts.iter() {
                    if !asset_db.has_asset(AssetKind::FloorPartConfig, part_id.uuid) {
                        continue;
                    }

                    if let Some(module) = modules.get_mut(uuid_mapping[&config.part]) {
                        module.add_north_neighbour(uuid_mapping[part_id]);
                    }
                    if let Some(module) = modules.get_mut(uuid_mapping[part_id]) {
                        module.add_south_neighbour(uuid_mapping[&config.part]);
                    }
                }

                for part_id in config.south_adjacent_parts.iter() {
                    if !asset_db.has_asset(AssetKind::FloorPartConfig, part_id.uuid) {
                        continue;
                    }

                    if let Some(module) = modules.get_mut(uuid_mapping[&config.part]) {
                        module.add_south_neighbour(uuid_mapping[part_id]);
                    }
                    if let Some(module) = modules.get_mut(uuid_mapping[part_id]) {
                        module.add_north_neighbour(uuid_mapping[&config.part]);
                    }
                }

                for part_id in config.west_adjacent_parts.iter() {
                    if !asset_db.has_asset(AssetKind::FloorPartConfig, part_id.uuid) {
                        continue;
                    }

                    if let Some(module) = modules.get_mut(uuid_mapping[&config.part]) {
                        module.add_west_neighbour(uuid_mapping[part_id]);
                    }
                    if let Some(module) = modules.get_mut(uuid_mapping[part_id]) {
                        module.add_east_neighbour(uuid_mapping[&config.part]);
                    }
                }

                for part_id in config.east_adjacent_parts.iter() {
                    if !asset_db.has_asset(AssetKind::FloorPartConfig, part_id.uuid) {
                        continue;
                    }

                    if let Some(module) = modules.get_mut(uuid_mapping[&config.part]) {
                        module.add_east_neighbour(uuid_mapping[part_id]);
                    }
                    if let Some(module) = modules.get_mut(uuid_mapping[part_id]) {
                        module.add_west_neighbour(uuid_mapping[&config.part]);
                    }
                }
            }

            macro_rules! do_generation {
                (
                    $gen_floor_name:ident<$size:literal>,
                    $inner_name:ident,
                    $outer_name:ident
                ) => {
                    let width = $gen_floor_name::<$size, $size>::PARTS_WIDTH;
                    let height = $gen_floor_name::<$size, $size>::PARTS_HEIGHT;
                    let mut wfc_context: WfcContext<CustomBitSet> =
                        WfcContextBuilder::new(&modules, width, height).build();

                    let (tx, rc) = channel();
                    wfc_context.collapse(100, tx.clone());
                    let results = rc
                        .recv()
                        .unwrap()
                        .unwrap_or_else(|_| vec![0; width * height]);

                    let mut result_floor = $inner_name::default();

                    for j in 0..height {
                        for i in 0..width {
                            let idx = width * j + i;

                            let config_id = module_ids[results[idx]];

                            let part_config_bytes =
                                asset_db.load_asset(AssetKind::FloorPartConfig, config_id.uuid);

                            let part_config = FloorPartConfig::load_from_slice(part_config_bytes)
                                .expect("Failed to parse FloorPartConfig");

                            let j_start = j * 5;
                            let i_start = i * 5;

                            for jj in 0..5 {
                                for ii in 0..5 {
                                    result_floor.floor_data[j_start + jj][i_start + ii] =
                                        part_config.floor_data[jj][ii];
                                }
                            }
                        }
                    }
                    return Some(FloorGeneratorResult::$outer_name(result_floor));
                };
            }

            match floor_var {
                FloorVariant::Generated15x15(_) => {
                    do_generation!(GeneratedFloor<3>, AuthoredFloorSize15x15, Size15x15);
                }
                FloorVariant::Generated20x20(_) => {
                    do_generation!(GeneratedFloor<4>, AuthoredFloorSize20x20, Size20x20);
                }
                FloorVariant::Generated25x25(_) => {
                    do_generation!(GeneratedFloor<5>, AuthoredFloorSize25x25, Size25x25);
                }
                FloorVariant::Generated30x30(_) => {
                    do_generation!(GeneratedFloor<6>, AuthoredFloorSize30x30, Size30x30);
                }
                FloorVariant::Generated40x40(_) => {
                    do_generation!(GeneratedFloor<8>, FloorSize40x40, Size40x40);
                }
                FloorVariant::Generated60x60(_) => {
                    do_generation!(GeneratedFloor<12>, FloorSize60x60, Size60x60);
                }
                FloorVariant::Generated80x80(_) => {
                    do_generation!(GeneratedFloor<16>, FloorSize80x80, Size80x80);
                }
                _ => unreachable!(),
            }
        }
    }
}
