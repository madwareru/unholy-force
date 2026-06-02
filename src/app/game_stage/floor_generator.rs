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
    app::editor_stage::image_widgets::{
        EditableFloorData
    },
    app::game_stage::grid_math::{traverse_area_inward},
    game_config::floors::{AuthoredFloor, GeneratedFloor},
    graphics::{FloorGraphicsTileGroup, WallGraphicsTileGroup}
};
use simple_tiled_wfc::grid_generation::{WfcModule};
use std::collections::{HashMap, HashSet};
use bitsetium::{BitEmpty, BitIntersection, BitSet, BitTest, BitUnset};
use rand::RngExt;
use simple_tiled_wfc::{make_initial_probabilities, BitsIterator};
use crate::app::game_stage::grid_math::get_island_mapping;
use crate::game_config::floor_parts::FloorCellExtra;

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
            FloorGeneratorResult::Size15x15(data) => &data.floor_data[y][x],
            FloorGeneratorResult::Size20x20(data) => &data.floor_data[y][x],
            FloorGeneratorResult::Size25x25(data) => &data.floor_data[y][x],
            FloorGeneratorResult::Size30x30(data) => &data.floor_data[y][x],
            FloorGeneratorResult::Size40x40(data) => &data.floor_data[y][x],
            FloorGeneratorResult::Size60x60(data) => &data.floor_data[y][x],
            FloorGeneratorResult::Size80x80(data) => &data.floor_data[y][x]
        }
    }

    fn get_floor_data_mut(&mut self, [x, y]: [usize; 2]) -> &mut FloorGraphicsTileGroup {
        match self {
            FloorGeneratorResult::Size15x15(data) => &mut data.floor_data[y][x],
            FloorGeneratorResult::Size20x20(data) => &mut data.floor_data[y][x],
            FloorGeneratorResult::Size25x25(data) => &mut data.floor_data[y][x],
            FloorGeneratorResult::Size30x30(data) => &mut data.floor_data[y][x],
            FloorGeneratorResult::Size40x40(data) => &mut data.floor_data[y][x],
            FloorGeneratorResult::Size60x60(data) => &mut data.floor_data[y][x],
            FloorGeneratorResult::Size80x80(data) => &mut data.floor_data[y][x]
        }
    }

    fn get_wall_data(&self, [x, y]: [usize; 2]) -> &WallGraphicsTileGroup {
        match self {
            FloorGeneratorResult::Size15x15(data) => &data.wall_data[y][x],
            FloorGeneratorResult::Size20x20(data) => &data.wall_data[y][x],
            FloorGeneratorResult::Size25x25(data) => &data.wall_data[y][x],
            FloorGeneratorResult::Size30x30(data) => &data.wall_data[y][x],
            FloorGeneratorResult::Size40x40(data) => &data.wall_data[y][x],
            FloorGeneratorResult::Size60x60(data) => &data.wall_data[y][x],
            FloorGeneratorResult::Size80x80(data) => &data.wall_data[y][x]
        }
    }

    fn get_wall_data_mut(&mut self, [x, y]: [usize; 2]) -> &mut WallGraphicsTileGroup {
        match self {
            FloorGeneratorResult::Size15x15(data) => &mut data.wall_data[y][x],
            FloorGeneratorResult::Size20x20(data) => &mut data.wall_data[y][x],
            FloorGeneratorResult::Size25x25(data) => &mut data.wall_data[y][x],
            FloorGeneratorResult::Size30x30(data) => &mut data.wall_data[y][x],
            FloorGeneratorResult::Size40x40(data) => &mut data.wall_data[y][x],
            FloorGeneratorResult::Size60x60(data) => &mut data.wall_data[y][x],
            FloorGeneratorResult::Size80x80(data) => &mut data.wall_data[y][x]
        }
    }

    fn get_cell_extra_data(&self, [x, y]: [usize; 2]) -> &FloorCellExtra {
        match self {
            FloorGeneratorResult::Size15x15(data) => &data.extra_data[y][x],
            FloorGeneratorResult::Size20x20(data) => &data.extra_data[y][x],
            FloorGeneratorResult::Size25x25(data) => &data.extra_data[y][x],
            FloorGeneratorResult::Size30x30(data) => &data.extra_data[y][x],
            FloorGeneratorResult::Size40x40(data) => &data.extra_data[y][x],
            FloorGeneratorResult::Size60x60(data) => &data.extra_data[y][x],
            FloorGeneratorResult::Size80x80(data) => &data.extra_data[y][x]
        }
    }

    fn get_cell_extra_data_mut(&mut self, [x, y]: [usize; 2]) -> &mut FloorCellExtra {
        match self {
            FloorGeneratorResult::Size15x15(data) => &mut data.extra_data[y][x],
            FloorGeneratorResult::Size20x20(data) => &mut data.extra_data[y][x],
            FloorGeneratorResult::Size25x25(data) => &mut data.extra_data[y][x],
            FloorGeneratorResult::Size30x30(data) => &mut data.extra_data[y][x],
            FloorGeneratorResult::Size40x40(data) => &mut data.extra_data[y][x],
            FloorGeneratorResult::Size60x60(data) => &mut data.extra_data[y][x],
            FloorGeneratorResult::Size80x80(data) => &mut data.extra_data[y][x]
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
                let config_bytes = asset_db.load_asset(AssetKind::FloorPartConfig, module_id.uuid);
                let config = FloorPartConfig::load_from_slice(config_bytes)
                    .expect("Failed to parse FloorPartConfig");
                let id = floor_part_cache.len();
                if config.wall_data[0].iter().all(|it| !it.eq(&WallGraphicsTileGroup::None)) {
                    probabilities_north_edge.set(id);
                }
                if config.wall_data[4].iter().all(|it| !it.eq(&WallGraphicsTileGroup::None)) {
                    probabilities_south_edge.set(id);
                }
                if config.wall_data.iter().all(
                    |it| !it[0].eq(&WallGraphicsTileGroup::None),
                ) {
                    probabilities_west_edge.set(id);
                }
                if config.wall_data.iter().all(
                    |it| !it[4].eq(&WallGraphicsTileGroup::None),
                ) {
                    probabilities_east_edge.set(id);
                }
                floor_part_cache.insert(*module_id, config);
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

            let mut indices_to_choose = Vec::new();
            let mut rng = rand::rng();

            let mut results = vec![None; width * height];
            for [i, j] in traverse_area_inward(width) {
                let idx = j * width + i;
                let mut prob = probabilities;

                if j == 0 {
                    prob = prob.intersection(probabilities_north_edge);
                } else if j == height - 1 {
                    prob = prob.intersection(probabilities_south_edge);
                }
                if i == 0 {
                    prob = prob.intersection(probabilities_west_edge);
                } else if i == width - 1 {
                    prob = prob.intersection(probabilities_east_edge);
                }

                let prob_copy = prob;
                for bit in BitsIterator::new(&prob_copy) {
                    let module = modules[bit];
                    if j > 0 && let Some(id) = results[j * width + i - width] {
                        if !module.north_neighbours.test(id) {
                            prob.unset(bit);
                        }
                    }
                    if j < height - 1 && let Some(id) = results[j * width + i + width] {
                        if !module.south_neighbours.test(id) {
                            prob.unset(bit);
                        }
                    }
                    if i > 0 && let Some(id) = results[j * width + i - 1] {
                        if !module.west_neighbours.test(id) {
                            prob.unset(bit);
                        }
                    }
                    if i < width - 1 && let Some(id) = results[j * width + i + 1] {
                        if !module.east_neighbours.test(id) {
                            prob.unset(bit);
                        }
                    }
                }

                indices_to_choose.clear();
                indices_to_choose.extend(BitsIterator::new(&prob));

                if !indices_to_choose.is_empty() {
                    let id =rng.random_range(0..indices_to_choose.len());
                    results[idx] = Some(indices_to_choose[id]);
                }
            }

            for [i, j] in traverse_area_inward(width) {
                let idx = j * width + i;

                let Some(id) = results[idx] else {
                    continue;
                };

                let config_id = module_ids[id];

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

            connect_islands(&mut result_floor);

            Some(result_floor)
        }
    }
}

fn connect_islands(result_floor: &mut FloorGeneratorResult) {
    let (mut num_islands, mut islands_mapping) = get_island_mapping(result_floor);
    while num_islands > 1 {
        struct CrossRoadCorner {
            x: usize,
            y: usize,
            north_way: Option<(u8, usize)>,
            south_way: Option<(u8, usize)>,
            west_way: Option<(u8, usize)>,
            east_way: Option<(u8, usize)>,
        }
        impl CrossRoadCorner {
            pub fn unique_way_count(&self) -> u8 {
                let mut color_buffer = [0; 4];
                let mut num_colors = 0;
                if let Some((_, color)) = self.north_way {
                    if (0..num_colors).all(|i| color_buffer[i] != color) {
                        color_buffer[num_colors] = color;
                        num_colors += 1;
                    }
                }
                if let Some((_, color)) = self.south_way {
                    if (0..num_colors).all(|i| color_buffer[i] != color) {
                        color_buffer[num_colors] = color;
                        num_colors += 1;
                    }
                }
                if let Some((_, color)) = self.west_way {
                    if (0..num_colors).all(|i| color_buffer[i] != color) {
                        color_buffer[num_colors] = color;
                        num_colors += 1;
                    }
                }
                if let Some((_, color)) = self.east_way {
                    if (0..num_colors).all(|i| color_buffer[i] != color) {
                        num_colors += 1;
                    }
                }
                num_colors as u8
            }
            pub fn max_way_length(&self) -> u8 {
                self.north_way.map(|it| it.0).unwrap_or(0)
                    .max(self.south_way.map(|it| it.0).unwrap_or(0))
                    .max(self.west_way.map(|it| it.0).unwrap_or(0))
                    .max(self.east_way.map(|it| it.0).unwrap_or(0))
            }
            pub fn is_worse_than(&self, other: &Self) -> bool {
                self.unique_way_count() < other.unique_way_count() ||
                    self.max_way_length() > other.max_way_length()
            }
        }

        let mut better_corner: Option<CrossRoadCorner> = None;
        let mut known_islands = HashSet::new();

        for j in 0..result_floor.height() {
            for i in 0..result_floor.width() {
                known_islands.clear();
                let mut corner = CrossRoadCorner {
                    x: i,
                    y: j,
                    north_way: None,
                    south_way: None,
                    west_way: None,
                    east_way: None,
                };

                let idx = j * result_floor.width() + i;

                if !islands_mapping[idx].is_none() {
                    continue;
                }

                'search_north: for jj in (0..j).rev() {
                    let idx = jj * result_floor.width() + i;
                    if let Some(color) = islands_mapping[idx] {
                        let length = (j - jj) as u8;
                        if known_islands.insert(color) {
                            corner.north_way = Some((length, color));
                        }
                        break 'search_north;
                    }
                }
                'search_south: for jj in j + 1..result_floor.height() {
                    let idx = jj * result_floor.width() + i;
                    if let Some(color) = islands_mapping[idx] {
                        let length = (jj - j) as u8;
                        if known_islands.insert(color) {
                            corner.south_way = Some((length, color));
                        }
                        break 'search_south;
                    }
                }
                'search_west: for ii in (0..i).rev() {
                    let idx = j * result_floor.width() + ii;
                    if let Some(color) = islands_mapping[idx] {
                        let length = (i - ii) as u8;
                        if known_islands.insert(color) {
                            corner.west_way = Some((length, color));
                        }
                        break 'search_west;
                    }
                }
                'search_east: for ii in i + 1..result_floor.width() {
                    let idx = j * result_floor.width() + ii;
                    if let Some(color) = islands_mapping[idx] {
                        let length = (ii - i) as u8;
                        if known_islands.insert(color) {
                            corner.east_way = Some((length, color));
                        }
                        break 'search_east;
                    }
                }

                if corner.unique_way_count() < 2 {
                    continue;
                }

                if better_corner.is_none() {
                    better_corner = Some(corner);
                    continue;
                }

                if let Some(best) = &better_corner && best.is_worse_than(&corner) {
                    better_corner = Some(corner);
                }
            }
        }

        let Some(better_corner) = better_corner else {
            println!("Curious situation: no better corner found to connect islands.");
            break;
        };

        *result_floor.get_wall_data_mut([better_corner.x, better_corner.y]) =
            WallGraphicsTileGroup::None;
        *result_floor.get_floor_data_mut([better_corner.x, better_corner.y]) =
            FloorGraphicsTileGroup::Tile;

        if let Some((num_tiles, _)) = better_corner.north_way {
            for i in 1..=num_tiles {
                *result_floor.get_wall_data_mut([better_corner.x, better_corner.y - i as usize]) =
                    WallGraphicsTileGroup::None;
                *result_floor.get_floor_data_mut([better_corner.x, better_corner.y - i as usize]) =
                    FloorGraphicsTileGroup::Tile;
            }
        }
        if let Some((num_tiles, _)) = better_corner.south_way {
            for i in 1..=num_tiles {
                *result_floor.get_wall_data_mut([better_corner.x, better_corner.y + i as usize]) =
                    WallGraphicsTileGroup::None;
                *result_floor.get_floor_data_mut([better_corner.x, better_corner.y + i as usize]) =
                    FloorGraphicsTileGroup::Tile;
            }
        }
        if let Some((num_tiles, _)) = better_corner.west_way {
            for i in 1..=num_tiles {
                *result_floor.get_wall_data_mut([better_corner.x - i as usize, better_corner.y]) =
                    WallGraphicsTileGroup::None;
                *result_floor.get_floor_data_mut([better_corner.x - i as usize, better_corner.y]) =
                    FloorGraphicsTileGroup::Tile;
            }
        }
        if let Some((num_tiles, _)) = better_corner.east_way {
            for i in 1..=num_tiles {
                *result_floor.get_wall_data_mut([better_corner.x + i as usize, better_corner.y]) =
                    WallGraphicsTileGroup::None;
                *result_floor.get_floor_data_mut([better_corner.x + i as usize, better_corner.y]) =
                    FloorGraphicsTileGroup::Tile;
            }
        }

        (num_islands, islands_mapping) = get_island_mapping(result_floor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn should_connect_islands() {
        let mut floor = FloorGeneratorResult::Size15x15(AuthoredFloorSize15x15::default());
        for [row, col] in [
            [1, 12],
            [2, 11], [2, 12], [2, 13],
            [3, 11], [3, 12], [3, 13],
            [4, 12],
            [5, 7],
            [6, 5], [6, 6], [6, 7], [6, 8], [6, 9],
            [7, 8],
            [8, 6], [8, 7], [8, 8],
            [9, 6],
            [10, 1], [10, 6],
            [11, 1], [11, 2], [11, 3], [11, 4], [11, 5], [11, 6], [11, 7], [11, 8],
            [12, 1], [12, 2], [12, 3], [12, 4], [12, 5], [12, 6], [12, 7], [12, 8],
            [13, 1], [13, 2], [13, 3], [13, 4], [13, 5], [13, 6], [13, 7], [13, 8],
        ] {
            *floor.get_wall_data_mut([col, row]) = WallGraphicsTileGroup::None;
            *floor.get_floor_data_mut([col, row]) = FloorGraphicsTileGroup::Tile;
        }
        let (num_islands, _) = get_island_mapping(&floor);
        assert_eq!(num_islands, 2);

        connect_islands(&mut floor);
        let (num_islands, _) = get_island_mapping(&floor);
        assert_eq!(num_islands, 1);
    }
}