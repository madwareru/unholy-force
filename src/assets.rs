use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use uuid::Uuid;
use crate::game_config::{ConfigId, GameConfig};

type SharedAssetDb = Arc<Mutex<AssetDb>>;

lazy_static!(
    pub static ref ASSET_DATABASE: SharedAssetDb = Arc::new(Mutex::new(AssetDb::load()));
);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum AssetKind {
    UnitConfig,
    ItemConfig,
    FloorPartConfig,
    FloorPartAdjacencyConfig,
    FloorConfig,
    FloorFlowGraphConfig,
    ParameterConfig,
    TagConfig,
    EffectMechanicConfig,
    GameGonfig
}
impl AssetKind {
    pub fn editor_label(&self) -> &'static str {
        match self {
            AssetKind::UnitConfig => "Персонажи",
            AssetKind::ItemConfig => "Предметы",
            AssetKind::FloorPartConfig => "Части этажей",
            AssetKind::FloorPartAdjacencyConfig => "Связи частей",
            AssetKind::FloorConfig => "Этажи",
            AssetKind::FloorFlowGraphConfig => "Граф этажей",
            AssetKind::ParameterConfig => "Черты",
            AssetKind::TagConfig => "Лычки",
            AssetKind::EffectMechanicConfig => "Механики эффектов",
            AssetKind::GameGonfig => "Игры"
        }
    }
}

pub struct AssetDb {
    assets: HashMap<AssetKind, HashMap<Uuid, (String, Vec<u8>)>>,
    deleted_assets: HashSet<(AssetKind, Uuid)>,
    changed_assets: HashSet<(AssetKind, Uuid)>,
}

impl AssetDb {
    fn load() -> Self {
        let mut unit_assets = HashMap::new();
        let mut item_assets = HashMap::new();
        let mut floor_part_assets = HashMap::new();
        let mut floor_part_adjacency_assets = HashMap::new();
        let mut floor_assets = HashMap::new();
        let mut floor_flow_graph_assets = HashMap::new();
        let mut parameter_assets = HashMap::new();
        let mut tag_assets = HashMap::new();

        for (kind, map, ext) in [
            (AssetKind::UnitConfig, &mut unit_assets, ".json5"),
            (AssetKind::ItemConfig, &mut item_assets, ".json5"),
            (AssetKind::FloorPartConfig, &mut floor_part_assets, ".part"),
            (AssetKind::FloorPartAdjacencyConfig, &mut floor_part_adjacency_assets, ".json5"),
            (AssetKind::FloorConfig, &mut floor_assets, ".json5"),
            (AssetKind::FloorFlowGraphConfig, &mut floor_flow_graph_assets, ".json5"),
            (AssetKind::ParameterConfig, &mut parameter_assets, ".json5"),
            (AssetKind::TagConfig, &mut tag_assets, ".json5"),
        ] {
            let asset_dir = get_or_create_asset_dir(kind);
            if let Ok(dir) = std::fs::read_dir(asset_dir) {
                for entry in dir {
                    let Ok(entry) = entry else {
                        continue;
                    };
                    let Some(file_name) = entry
                        .file_name()
                        .to_str()
                        .map(|it| it.to_string()) else {
                        continue;
                    };
                    if !file_name.ends_with(ext) {
                        continue;
                    }

                    let mut uuid_split = file_name.split(ext);
                    let Some(uuid_string) = uuid_split.next() else {
                        continue;
                    };
                    let Ok(id) = Uuid::parse_str(uuid_string) else {
                        panic!("Failed to parse uuid: {}", file_name);
                    };

                    let bytes = load_asset_bytes(kind, id);
                    let name = get_asset_name(kind, id);
                    map.insert(id, (name, bytes));
                }
            }
        }

        let mut assets = HashMap::new();
        assets.insert(AssetKind::UnitConfig, unit_assets);
        assets.insert(AssetKind::ItemConfig, item_assets);
        assets.insert(AssetKind::FloorPartConfig, floor_part_assets);
        assets.insert(AssetKind::FloorPartAdjacencyConfig, floor_part_adjacency_assets);
        assets.insert(AssetKind::FloorConfig, floor_assets);
        assets.insert(AssetKind::FloorFlowGraphConfig, floor_flow_graph_assets);
        assets.insert(AssetKind::ParameterConfig, parameter_assets);
        assets.insert(AssetKind::TagConfig, tag_assets);

        Self {
            assets,
            deleted_assets: Default::default(),
            changed_assets: Default::default()
        }
    }

    pub fn list_all_assets(&self, kind: AssetKind) -> impl Iterator<Item = (Uuid, &str)> {
        self.assets[&kind].iter().map(|(id, (name, _))| (*id, name.as_str()))
    }

    pub fn create_asset(&mut self, kind: AssetKind, name: &str, data: &[u8]) -> Uuid {
        let uuid = create_new_asset(kind, name, data);
        if let Some(assets) = self.assets.get_mut(&kind) {
            assets.insert(uuid, (String::from(name), data.to_vec()));
        }
        uuid
    }

    pub fn create_json5_asset(&mut self, kind: AssetKind, name: &str, text: &str) -> Uuid {
        let uuid = create_new_asset(kind, name, text.as_bytes());
        if let Some(assets) = self.assets.get_mut(&kind) {
            assets.insert(uuid, (String::from(name), text.as_bytes().to_vec()));
        }
        uuid
    }

    pub fn load_json5_asset(&self, kind: AssetKind, uuid: Uuid) -> &str {
        str::from_utf8(&self.assets[&kind][&uuid].1)
            .expect("Failed to load json5 asset")
    }

    pub fn load_asset(&self, kind: AssetKind, uuid: Uuid) -> &[u8] {
        &self.assets[&kind][&uuid].1
    }

    pub fn update_asset(&mut self, kind: AssetKind, uuid: Uuid, data: &[u8]) {
        if let Some(assets) = self.assets.get_mut(&kind) {
            if let Some(asset) = assets.get_mut(&uuid) {
                asset.1 = data.to_vec();
                self.changed_assets.insert((kind, uuid));
            }
        }
    }

    pub fn update_json5_asset(&mut self, kind: AssetKind, uuid: Uuid, text: &str) {
        if let Some(assets) = self.assets.get_mut(&kind) {
            if let Some(asset) = assets.get_mut(&uuid) {
                asset.1 = text.as_bytes().to_vec();
                self.changed_assets.insert((kind, uuid));
            }
        }
    }

    pub fn rename_asset(&mut self, kind: AssetKind, uuid: Uuid, name: &str) {
        if let Some(assets) = self.assets.get_mut(&kind) {
            if let Some(asset) = assets.get_mut(&uuid) {
                asset.0 = String::from(name);
            }
        }

    }

    pub fn delete_asset(&mut self, kind: AssetKind, uuid: Uuid) {
        if self.assets.get(&kind).and_then(|x| x.get(&uuid)).is_none() {
            return;
        }
        self.deleted_assets.insert((kind, uuid));
        let Some(assets) = self.assets.get_mut(&kind) else {
            return;
        };
        assets.remove(&uuid);
    }

    pub fn flush_assets_to_disk(&mut self) {
        for (kind, id) in self.deleted_assets.drain() {
            remove_asset(kind, id);
        }
        for (kind, uuid) in self.changed_assets.drain() {
            if let Some((name, bytes)) = self.assets[&kind].get(&uuid) {
                write_asset_bytes(kind, uuid, bytes);
                rename_asset(kind, uuid, name);
            }
        }
    }
}

fn executable_dir() -> PathBuf {
    std::env::current_exe()
        .expect("Failed to get exe path")
        .parent()
        .map(std::path::Path::to_path_buf)
        .expect("executable path has no parent directory")
}

fn asset_dir(kind: AssetKind) -> PathBuf {
    executable_dir().join("assets").join(
        match kind {
            AssetKind::UnitConfig => "units",
            AssetKind::ItemConfig => "items",
            AssetKind::FloorPartConfig => "floor_parts",
            AssetKind::FloorPartAdjacencyConfig => "floor_part_adjacency",
            AssetKind::FloorConfig => "floors",
            AssetKind::FloorFlowGraphConfig => "floor_flow_graph",
            AssetKind::ParameterConfig => "parameters",
            AssetKind::TagConfig => "tags",
            AssetKind::EffectMechanicConfig => "effect_mechanics",
            AssetKind::GameGonfig => "games"
        }
    )
}

fn project_dir() -> PathBuf {
    executable_dir().join("project")
}

fn get_or_create_asset_dir(kind: AssetKind) -> PathBuf {
    let asset_dir = asset_dir(kind);
    if !asset_dir.exists() {
        std::fs::create_dir_all(&asset_dir)
            .unwrap_or_else(|_| panic!("Failed to create asset dir: {:?}", &asset_dir));
    }
    asset_dir
}

fn get_or_create_project_dir() -> PathBuf {
    let project_dir = project_dir();
    if !project_dir.exists() {
        std::fs::create_dir_all(&project_dir)
            .unwrap_or_else(|_| panic!("Failed to create project dir: {:?}", &project_dir));
        update_project_game_config_id(ConfigId::INVALID);
    }
    project_dir
}

fn update_project_game_config_id(game_config_id: ConfigId<GameConfig>) {
    let game_setting_path = get_or_create_project_dir()
        .join("game.settings");
    let mut buffer: Vec<u8> = Vec::new();
    buffer.write(game_config_id.uuid.as_bytes())
        .expect("Failed to write game config id into a buffer");
    std::fs::write(game_setting_path, buffer)
        .expect("Failed to write game config id into a file");
}

fn asset_name_file_name(kind: AssetKind, id: Uuid) -> PathBuf {
    asset_dir(kind).join(format!("{}.name", id))
}

fn asset_file_name(kind: AssetKind, id: Uuid) -> PathBuf {
    asset_dir(kind)
        .join(match kind {
            AssetKind::FloorPartConfig => format!("{id}.part"),
            _ => format!("{id}.json5")
        })
}

fn get_asset_name(kind: AssetKind, id: Uuid) -> String {
    let file_name = asset_name_file_name(kind, id);
    std::fs::read_to_string(file_name)
        .expect("Failed to read asset name")
}

fn rename_asset(kind: AssetKind, id: Uuid, new_name: &str) {
    let file_name = asset_name_file_name(kind, id);
    std::fs::write(file_name, new_name).expect("Failed to rename asset");
}

fn load_asset_bytes (kind: AssetKind, id: Uuid) -> Vec<u8> {
    let file_name = asset_file_name(kind, id);
    std::fs::read(file_name).expect("Failed to read asset file")
}

fn write_asset_bytes (kind: AssetKind, id: Uuid, data: &[u8]) {
    let file_name = asset_file_name(kind, id);
    std::fs::write(file_name, data).expect("Failed to write asset file");
}

fn remove_asset(kind: AssetKind, id: Uuid) {
    std::fs::remove_file(asset_name_file_name(kind, id))
        .expect("Failed to remove asset name file");
    std::fs::remove_file(asset_file_name(kind, id))
        .expect("Failed to remove asset file");
}

fn create_new_asset(kind: AssetKind, name: &str, data: &[u8]) -> Uuid {
    let unix_epoch = std::time::SystemTime::UNIX_EPOCH;
    let ts = std::time::SystemTime::now()
        .duration_since(unix_epoch)
        .expect("Failed to get timestamp")
        .as_millis()
        .try_into()
        .expect("Failed to get timestamp");
    let rand_bytes = rand::random::<[u8; 10]>();
    let id = uuid::Builder::from_unix_timestamp_millis(ts, &rand_bytes).into_uuid();
    let file_name = asset_file_name(kind, id);
    std::fs::write(file_name, data).expect("Failed to write asset file");
    rename_asset(kind, id, name);
    id
}