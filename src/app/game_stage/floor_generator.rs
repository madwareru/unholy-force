use crate::{
    assets::{AssetDb, AssetKind},
    game_config::ConfigId,
    game_config::floor_part_adjacency::FloorPartAdjacencyConfig,
    game_config::floor_parts::FloorPartConfig,
    game_config::floors::{
        AuthoredFloorSize15x15, AuthoredFloorSize20x20, AuthoredFloorSize25x25,
        AuthoredFloorSize30x30, FloorConfig, FloorSize40x40, FloorSize60x60, FloorSize80x80,
        FloorVariant,
    },
};
use simple_tiled_wfc::grid_generation::{WfcModule};
use std::collections::HashMap;
use bitsetium::{BitEmpty, BitSet};
use simple_tiled_wfc::make_initial_probabilities;
use crate::app::editor_stage::image_widgets::{EditableFloorData, FloorTilesHolderConst, WallTilesHolderConst};
use crate::app::game_stage::grid_math::{traverse_area_inward, traverse_area_outward};
use crate::game_config::floors::{AuthoredFloor, GeneratedFloor};
use crate::graphics::{FloorGraphicsTileGroup, WallGraphicsTileGroup};

pub enum FloorGeneratorResult {
    Size15x15(AuthoredFloorSize15x15),
    Size20x20(AuthoredFloorSize20x20),
    Size25x25(AuthoredFloorSize25x25),
    Size30x30(AuthoredFloorSize30x30),
    Size40x40(FloorSize40x40),
    Size60x60(FloorSize60x60),
    Size80x80(FloorSize80x80),
}
impl EditableFloorData for FloorGeneratorResult {
    fn width(&self) -> usize {
        match self {
            FloorGeneratorResult::Size15x15(_) => 15,
            FloorGeneratorResult::Size20x20(_) => 20,
            FloorGeneratorResult::Size25x25(_) => 25,
            FloorGeneratorResult::Size30x30(_) => 30,
            FloorGeneratorResult::Size40x40(_) => 40,
            FloorGeneratorResult::Size60x60(_) => 60,
            FloorGeneratorResult::Size80x80(_) => 80
        }
    }

    fn height(&self) -> usize {
        match self {
            FloorGeneratorResult::Size15x15(_) => 15,
            FloorGeneratorResult::Size20x20(_) => 20,
            FloorGeneratorResult::Size25x25(_) => 25,
            FloorGeneratorResult::Size30x30(_) => 30,
            FloorGeneratorResult::Size40x40(_) => 40,
            FloorGeneratorResult::Size60x60(_) => 60,
            FloorGeneratorResult::Size80x80(_) => 80
        }
    }

    fn get_floor_data(&self, [x, y]: [usize; 2]) -> &FloorGraphicsTileGroup {
        match self {
            FloorGeneratorResult::Size15x15(data) => &data.floor_data()[y][x],
            FloorGeneratorResult::Size20x20(data) => &data.floor_data()[y][x],
            FloorGeneratorResult::Size25x25(data) => &data.floor_data()[y][x],
            FloorGeneratorResult::Size30x30(data) => &data.floor_data()[y][x],
            FloorGeneratorResult::Size40x40(data) => &data.floor_data()[y][x],
            FloorGeneratorResult::Size60x60(data) => &data.floor_data()[y][x],
            FloorGeneratorResult::Size80x80(data) => &data.floor_data()[y][x]
        }
    }

    fn get_floor_data_mut(&mut self, [x, y]: [usize; 2]) -> &mut FloorGraphicsTileGroup {
        match self {
            FloorGeneratorResult::Size15x15(data) => &mut data.floor_data_mut()[y][x],
            FloorGeneratorResult::Size20x20(data) => &mut data.floor_data_mut()[y][x],
            FloorGeneratorResult::Size25x25(data) => &mut data.floor_data_mut()[y][x],
            FloorGeneratorResult::Size30x30(data) => &mut data.floor_data_mut()[y][x],
            FloorGeneratorResult::Size40x40(data) => &mut data.floor_data_mut()[y][x],
            FloorGeneratorResult::Size60x60(data) => &mut data.floor_data_mut()[y][x],
            FloorGeneratorResult::Size80x80(data) => &mut data.floor_data_mut()[y][x]
        }
    }

    fn get_wall_data(&self, [x, y]: [usize; 2]) -> &WallGraphicsTileGroup {
        match self {
            FloorGeneratorResult::Size15x15(data) => &data.wall_data()[y][x],
            FloorGeneratorResult::Size20x20(data) => &data.wall_data()[y][x],
            FloorGeneratorResult::Size25x25(data) => &data.wall_data()[y][x],
            FloorGeneratorResult::Size30x30(data) => &data.wall_data()[y][x],
            FloorGeneratorResult::Size40x40(data) => &data.wall_data()[y][x],
            FloorGeneratorResult::Size60x60(data) => &data.wall_data()[y][x],
            FloorGeneratorResult::Size80x80(data) => &data.wall_data()[y][x]
        }
    }

    fn get_wall_data_mut(&mut self, [x, y]: [usize; 2]) -> &mut WallGraphicsTileGroup {
        match self {
            FloorGeneratorResult::Size15x15(data) => &mut data.wall_data_mut()[y][x],
            FloorGeneratorResult::Size20x20(data) => &mut data.wall_data_mut()[y][x],
            FloorGeneratorResult::Size25x25(data) => &mut data.wall_data_mut()[y][x],
            FloorGeneratorResult::Size30x30(data) => &mut data.wall_data_mut()[y][x],
            FloorGeneratorResult::Size40x40(data) => &mut data.wall_data_mut()[y][x],
            FloorGeneratorResult::Size60x60(data) => &mut data.wall_data_mut()[y][x],
            FloorGeneratorResult::Size80x80(data) => &mut data.wall_data_mut()[y][x]
        }
    }
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
        FloorVariant::Authored(AuthoredFloor::Size15x15(data)) =>
            Some(FloorGeneratorResult::Size15x15(data)),
        FloorVariant::Authored(AuthoredFloor::Size20x20(data)) =>
            Some(FloorGeneratorResult::Size20x20(data)),
        FloorVariant::Authored(AuthoredFloor::Size25x25(data)) =>
            Some(FloorGeneratorResult::Size25x25(data)),
        FloorVariant::Authored(AuthoredFloor::Size30x30(data)) =>
            Some(FloorGeneratorResult::Size30x30(data)),
        FloorVariant::Generated(floor_var) => {
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
            let mut floor_part_cache = HashMap::new();

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

            let module_count = modules.len();
            let probabilities: CustomBitSet = make_initial_probabilities(module_count);
            let mut probabilities_north_edge = CustomBitSet::empty();
            let mut probabilities_south_edge = CustomBitSet::empty();
            let mut probabilities_west_edge = CustomBitSet::empty();
            let mut probabilities_east_edge = CustomBitSet::empty();

            for module_id in module_ids.iter() {
                let part_config_bytes = asset_db.load_asset(AssetKind::FloorPartConfig, module_id.uuid);
                let part_config = FloorPartConfig::load_from_slice(part_config_bytes)
                    .expect("Failed to parse FloorPartConfig");
                let id = floor_part_cache.len();
                if part_config.wall_data[0].iter().all(|it| !it.eq(&WallGraphicsTileGroup::None)) {
                    probabilities_north_edge.set(id);
                }
                if part_config.wall_data[4].iter().all(|it| !it.eq(&WallGraphicsTileGroup::None)) {
                    probabilities_south_edge.set(id);
                }
                if part_config.wall_data.iter().all(
                    |it| !it[0].eq(&WallGraphicsTileGroup::None),
                ) {
                    probabilities_west_edge.set(id);
                }
                if part_config.wall_data.iter().all(
                    |it| !it[4].eq(&WallGraphicsTileGroup::None),
                ) {
                    probabilities_east_edge.set(id);
                }
                floor_part_cache.insert(*module_id, part_config);
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

            let (width, height, mut result_floor) = match floor_var {
                GeneratedFloor::Size15x15(_) =>
                    (3, 3, FloorGeneratorResult::Size15x15(AuthoredFloorSize15x15::default())),
                GeneratedFloor::Size20x20(_) =>
                    (4, 4, FloorGeneratorResult::Size20x20(AuthoredFloorSize20x20::default())),
                GeneratedFloor::Size25x25(_) =>
                    (5, 5, FloorGeneratorResult::Size25x25(AuthoredFloorSize25x25::default())),
                GeneratedFloor::Size30x30(_) =>
                    (6, 6, FloorGeneratorResult::Size30x30(AuthoredFloorSize30x30::default())),
                GeneratedFloor::Size40x40(_) =>
                    (8, 8, FloorGeneratorResult::Size40x40(FloorSize40x40::default())),
                GeneratedFloor::Size60x60(_) =>
                    (12, 12, FloorGeneratorResult::Size60x60(FloorSize60x60::default())),
                GeneratedFloor::Size80x80(_) =>
                    (16, 16, FloorGeneratorResult::Size80x80(FloorSize80x80::default())),
            };

            let walk_order = match floor_var {
                GeneratedFloor::Size15x15(_) => make_walk_indices(3),
                GeneratedFloor::Size20x20(_) => make_walk_indices(4),
                GeneratedFloor::Size25x25(_) => make_walk_indices(5),
                GeneratedFloor::Size30x30(_) => make_walk_indices(6),
                GeneratedFloor::Size40x40(_) => make_walk_indices(8),
                GeneratedFloor::Size60x60(_) => make_walk_indices(12),
                GeneratedFloor::Size80x80(_) => make_walk_indices(16),
            };

            let results = vec![0; width * height];
            let mut offset = 0;

            for [i, j] in traverse_area_outward(
                match floor_var {
                    GeneratedFloor::Size15x15(_) => 3,
                    GeneratedFloor::Size20x20(_) => 4,
                    GeneratedFloor::Size25x25(_) => 5,
                    GeneratedFloor::Size30x30(_) => 6,
                    GeneratedFloor::Size40x40(_) => 8,
                    GeneratedFloor::Size60x60(_) => 12,
                    GeneratedFloor::Size80x80(_) => 16,
                }
            ) {
                let config_id = module_ids[offset % modules.len()]; // module_ids[results[idx]];
                offset += 1;

                let Some(part_config) = floor_part_cache.get(&config_id) else {
                    continue;
                };

                let j_start = j * 5;
                let i_start = i * 5;

                for jj in 0..5 {
                    for ii in 0..5 {
                        *result_floor.get_floor_data_mut([i_start + ii, j_start + jj]) =
                            part_config.floor_data[jj][ii];
                        *result_floor.get_wall_data_mut([i_start + ii, j_start + jj]) =
                            part_config.wall_data[jj][ii];
                    }
                }
            }
            Some(result_floor)
        }
    }
}


fn make_walk_indices(width: usize) -> Vec<[usize; 2]> {
    fn grow_walk_indices(
        walk_order: &mut Vec<[usize; 2]>,
        [base_offset_x, base_offset_y]: [usize; 2],
        width: usize,
    ) {
        match width {
            0 => {}
            1 => {
                walk_order.push([base_offset_x, base_offset_y]);
            }
            2 => {
                walk_order.extend([
                    [base_offset_x, base_offset_y],
                    [base_offset_x + 1, base_offset_y],
                    [base_offset_x, base_offset_y + 1],
                    [base_offset_x + 1, base_offset_y + 1],
                ]);
            }
            3 => {
                walk_order.extend([
                    // Сначала углы:
                    [base_offset_x, base_offset_y],
                    [base_offset_x + 2, base_offset_y],
                    [base_offset_x, base_offset_y + 2],
                    [base_offset_x + 2, base_offset_y + 2],
                    // Затем рёбра:
                    [base_offset_x + 1, base_offset_y],
                    [base_offset_x + 1, base_offset_y + 2],
                    [base_offset_x, base_offset_y + 1],
                    [base_offset_x + 2, base_offset_y + 1],
                    // Затем центр:
                    [base_offset_x + 1, base_offset_y + 1],
                ]);
            }
            _ => {
                walk_order.extend([
                    // Углы
                    [base_offset_x, base_offset_y],
                    [base_offset_x + width - 1, base_offset_y],
                    [base_offset_x, base_offset_y + width - 1],
                    [base_offset_x + width - 1, base_offset_y + width - 1]
                ]);
                let ww = width - 2;

                // Рёбра заполняются от краёв к центру:
                for i in 0..ww / 2 {
                    walk_order.extend([
                        [base_offset_x + 1 + i, base_offset_y],
                        [base_offset_x + width - 2 - i, base_offset_y]
                    ]);
                    if ww % 2 == 1 {
                        walk_order.push([base_offset_x + ww / 2 + 1, base_offset_y]);
                    }
                    walk_order.extend([
                        [base_offset_x + 1 + i, base_offset_y + width - 1],
                        [base_offset_x + width - 2 - i, base_offset_y + width - 1]
                    ]);
                    if ww % 2 == 1 {
                        walk_order.push([base_offset_x + ww / 2 + 1, base_offset_y + width - 1]);
                    }
                    walk_order.extend([
                        [base_offset_x, base_offset_y + 1 + i],
                        [base_offset_x, base_offset_y + width - 2 - i]
                    ]);
                    if ww % 2 == 1 {
                        walk_order.push([base_offset_x, base_offset_y + ww / 2 + 1]);
                    }
                    walk_order.extend([
                        [base_offset_x + width - 1, base_offset_y + 1 + i],
                        [base_offset_x + width - 1, base_offset_y + width - 2 - i]
                    ]);
                    if ww % 2 == 1 {
                        walk_order.push([base_offset_x + width - 1, base_offset_y + ww / 2 + 1]);
                    }
                }

                // Центр заполняется через рекурсивный вызов:
                grow_walk_indices(
                    walk_order,
                    [base_offset_x + 1 , base_offset_y + 1],
                    ww
                );
            }
        }
    }

    let mut walk_order: Vec<[usize; 2]> = Vec::new();
    grow_walk_indices(&mut walk_order, [0, 0], width);
    assert_eq!(walk_order.len(), width * width);
    walk_order
}