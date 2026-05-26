use crate::game_config::{Config, ConfigId};
use crate::game_config::items::{ItemConfig, ItemRarity};
use crate::game_config::units::{UnitConfig, UnitDanger};
use crate::graphics::{FloorGraphicsTileGroup, WallGraphicsTileGroup};
use std::io::{Result, Error, ErrorKind, Write, Read};
use uuid::Uuid;
use crate::app::editor_stage::image_widgets::{FloorDataHolder, FloorTilesHolder, WallTilesHolder};
use crate::game_config::effects::EffectMechanicConfig;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FloorCellExtra {
    None,
    SpawnUnitHint(UnitDanger), // 1..6
    SpawnLootHint(ItemRarity), // 7..10
    SpawnUnit(ConfigId<UnitConfig>), // 11
    SpawnLoot(ConfigId<ItemConfig>), // 12
    LadderDownHint,
    LadderUpHint,
    PlayerStartHint,
    TriggerEffect(ConfigId<EffectMechanicConfig>)
}
impl FloorCellExtra {
    pub fn get_id(&self) -> u8 {
        match self {
            FloorCellExtra::None => 0,
            FloorCellExtra::SpawnUnitHint(unit_danger) => 1 + *unit_danger as u8,
            FloorCellExtra::SpawnLootHint(item_rarity) => 7 + *item_rarity as u8,
            FloorCellExtra::SpawnUnit(_) => 11,
            FloorCellExtra::SpawnLoot(_) => 12,
            FloorCellExtra::LadderDownHint => 13,
            FloorCellExtra::LadderUpHint => 14,
            FloorCellExtra::PlayerStartHint => 15,
            FloorCellExtra::TriggerEffect(_) => 0, // Записывается в отдельной битовой карте
        }
    }

    pub fn get_payload(&self) -> Option<Uuid> {
        match self {
            FloorCellExtra::SpawnUnit(ConfigId{ uuid, .. })
            | FloorCellExtra::SpawnLoot(ConfigId{ uuid, .. })
            | FloorCellExtra::TriggerEffect(ConfigId{ uuid, .. }) => Some(*uuid),
            _ => None,
        }
    }
}
impl Default for FloorCellExtra {
    fn default() -> Self { FloorCellExtra::None }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FloorPartConfig {
    pub floor_data: Box<[[FloorGraphicsTileGroup; 5]; 5]>,
    pub wall_data: Box<[[WallGraphicsTileGroup; 5]; 5]>,
    pub extra_data: Box<[[FloorCellExtra; 5]; 5]>,
    pub payload_count: u8,
}

impl FloorTilesHolder<5, 5> for FloorPartConfig {
    fn floor_data(&self) -> &[[FloorGraphicsTileGroup; 5]; 5] {
        &self.floor_data
    }
    fn floor_data_mut(&mut self) -> &mut [[FloorGraphicsTileGroup; 5]; 5] {
        &mut self.floor_data
    }
}

impl WallTilesHolder<5, 5> for FloorPartConfig {
    fn wall_data(&self) -> &[[WallGraphicsTileGroup; 5]; 5] {
        &self.wall_data
    }
    fn wall_data_mut(&mut self) -> &mut [[WallGraphicsTileGroup; 5]; 5] {
        &mut self.wall_data
    }
}

impl FloorDataHolder<5, 5> for FloorPartConfig {}


impl FloorPartConfig {
    const MAX_PAYLOAD_COUNT: u8 = 4;

    pub fn write(&self, writer: &mut impl Write) -> Result<()> {
        let mut payload_count = 0;
        for j in 0..5 {
            for i in 0..5 {
                if self.extra_data[j][i].get_payload().is_some() {
                    payload_count += 1;
                }
            }
        }

        if self.payload_count != payload_count {
            return Err(Error::new(ErrorKind::InvalidInput, "Payload count mismatch"));
        }

        if payload_count > Self::MAX_PAYLOAD_COUNT {
            return Err(Error::new(ErrorKind::InvalidInput, "Payload count exceeds maximum"));
        }

        writer.write(&[self.payload_count])?;

        if self.payload_count > 0 {
            for j in 0..5 {
                for i in 0..5 {
                    match self.extra_data[j][i].get_payload() {
                        Some(uuid) => {
                            let bytes = uuid.into_bytes();
                            writer.write(&bytes)?;
                        }
                        None => {}
                    }
                }
            }
        }

        let mut effect_spawner_mask = 0u32;

        for j in 0..5 {
            for i in 0..5 {
                let extra_id = self.extra_data[j][i].get_id();
                let wall_kind = self.wall_data[j][i] as u8;
                let floor_kind = self.floor_data[j][i] as u8;
                if let FloorCellExtra::TriggerEffect(_) = self.extra_data[j][i] {
                    effect_spawner_mask = effect_spawner_mask | 0b1;
                }
                effect_spawner_mask <<= 1;
                let packed = (extra_id << 4) | (wall_kind << 2) | floor_kind;
                writer.write(&[packed])?;
            }
        }

        for _ in 0..4 {
            let mask_part = (effect_spawner_mask & 0xFF) as u8;
            writer.write(&[mask_part])?;
            effect_spawner_mask >>= 8;
        }

        Ok(())
    }
    pub fn load_from_slice(data: &[u8]) -> Result<Self> {
        if data.is_empty() {
            return Err(Error::new(ErrorKind::InvalidInput, "Empty data"));
        }

        let payload_count;
        let size_expected = if data[0] == 0 {
            payload_count = 0;
            30
        } else if data[0] <= Self::MAX_PAYLOAD_COUNT {
            payload_count = data[0];
            30 + (payload_count as usize) * 16
        } else {
            return Err(Error::new(ErrorKind::InvalidInput, "Invalid payload count"));
        };

        if size_expected < data.len() {
            return Err(Error::new(ErrorKind::InvalidInput, "Invalid data length"));
        }

        let mut payload_data = &data[1..];
        let mut cells_data = &payload_data[(payload_count as usize) * 16..];
        let mut effect_spawner_mask_data = &cells_data[25..];

        let mut effect_spawner_mask = 0u32;
        for i in 0..4 {
            let mut mask_part = [0u8];
            effect_spawner_mask_data.read(&mut mask_part)?;
            effect_spawner_mask = effect_spawner_mask | ((mask_part[0] as u32) << (i * 8));
        }

        let mut floor_data: Box<[[FloorGraphicsTileGroup; 5]; 5]> = Default::default();
        let mut wall_data: Box<[[WallGraphicsTileGroup; 5]; 5]> = Default::default();
        let mut extra_data: Box<[[FloorCellExtra; 5]; 5]> = Default::default();
        for j in 0..5 {
            for i in 0..5 {
                let mut packed = [0u8];
                cells_data.read(&mut packed)?;
                let packed = packed[0];
                let floor = match packed & 0b0000011 {
                    0 => FloorGraphicsTileGroup::Dirt,
                    1 => FloorGraphicsTileGroup::Tile,
                    2 => FloorGraphicsTileGroup::Water,
                    _ => FloorGraphicsTileGroup::Lava
                };
                let wall = match (packed & 0b00001100) >> 2 {
                    0 => WallGraphicsTileGroup::None,
                    1 => WallGraphicsTileGroup::Sandstone,
                    2 => WallGraphicsTileGroup::Rocks,
                    _ => WallGraphicsTileGroup::Bricks
                };
                floor_data[j][i] = floor;
                wall_data[j][i] = wall;
                let extra = match (packed & 0b11110000) >> 4 {
                    0 => {
                        let idx = j * 5 + i;
                        let base_mask = 0b1u32 << 25;
                        let mask = base_mask >> idx;
                        if (effect_spawner_mask & mask) != 0 {
                            let mut uuid = [0u8; 16];
                            payload_data.read(&mut uuid)?;
                            let uuid = Uuid::from_bytes(uuid);
                            FloorCellExtra::TriggerEffect(ConfigId::from_uuid(uuid))
                        } else {
                            FloorCellExtra::None
                        }
                    },
                    x if (1..=6).contains(&x) => FloorCellExtra::SpawnUnitHint(UnitDanger::from_id(x -1)),
                    x if (7..=10).contains(&x) => FloorCellExtra::SpawnLootHint(
                        match x {
                            7 => ItemRarity::Generic,
                            8 => ItemRarity::Rare,
                            9 => ItemRarity::Unique,
                            _ => ItemRarity::Legendary
                        }
                    ),
                    11 => {
                        let mut uuid = [0u8; 16];
                        payload_data.read(&mut uuid)?;
                        let uuid = Uuid::from_bytes(uuid);
                        FloorCellExtra::SpawnUnit(ConfigId::from_uuid(uuid))
                    },
                    12 => {
                        let mut uuid = [0u8; 16];
                        payload_data.read(&mut uuid)?;
                        let uuid = Uuid::from_bytes(uuid);
                        FloorCellExtra::SpawnLoot(ConfigId::from_uuid(uuid))
                    },
                    13 => FloorCellExtra::LadderDownHint,
                    14 => FloorCellExtra::LadderUpHint,
                    15 => FloorCellExtra::PlayerStartHint,
                    _ => return Err(Error::new(ErrorKind::InvalidInput, "Invalid extra data"))
                };
                extra_data[j][i] = extra;
            }
        }

        Ok(Self {
            floor_data,
            wall_data,
            extra_data,
            payload_count,
        })
    }
}

impl Config for FloorPartConfig {}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use crate::game_config::ConfigId;
    use crate::game_config::floor_parts::{FloorCellExtra, FloorPartConfig};
    use crate::game_config::items::ItemRarity;
    use crate::game_config::units::UnitDanger;
    use crate::graphics::{FloorGraphicsTileGroup, WallGraphicsTileGroup};

    #[test]
    fn test_floor_part_config_serialization() {
        fn make_uuid() -> Uuid {
            // каждый раз идентификаторы будут разные, но это для нашего случая нормально,
            // нам нужно только убедиться, что соблюдается порядок и идентификаторы не
            // перепутаются друг между другом
            let unix_epoch = std::time::SystemTime::UNIX_EPOCH;
            let ts = std::time::SystemTime::now()
                .duration_since(unix_epoch)
                .expect("Failed to get timestamp")
                .as_millis()
                .try_into()
                .expect("Failed to get timestamp");
            let rand_bytes = rand::random::<[u8; 10]>();
            let id = uuid::Builder::from_unix_timestamp_millis(ts, &rand_bytes).into_uuid();
            id
        }

        let mut config = FloorPartConfig::default();
        config.payload_count = 3;
        config.floor_data[3][3] = FloorGraphicsTileGroup::Lava;
        config.floor_data[2][2] = FloorGraphicsTileGroup::Water;
        config.floor_data[1][1] = FloorGraphicsTileGroup::Tile;
        config.extra_data[1][1] = FloorCellExtra::LadderDownHint;
        config.extra_data[1][2] = FloorCellExtra::SpawnUnit(ConfigId::from_uuid(make_uuid()));
        config.extra_data[1][3] = FloorCellExtra::LadderUpHint;
        config.extra_data[2][2] = FloorCellExtra::TriggerEffect(ConfigId::from_uuid(make_uuid()));
        config.extra_data[3][1] = FloorCellExtra::SpawnUnitHint(UnitDanger::Challenging);
        config.extra_data[3][2] = FloorCellExtra::SpawnLootHint(ItemRarity::Rare);
        config.extra_data[3][3] = FloorCellExtra::SpawnLoot(ConfigId::from_uuid(make_uuid()));
        for j in 0..5 {
            for i in 0..5 {
                if (1..4).contains(&j) && (1..4).contains(&i) {
                    config.wall_data[j][i] = WallGraphicsTileGroup::None;
                }
            }
        }
        let mut buffer: Vec<u8> = Vec::new();
        config.write(&mut buffer).unwrap();

        let buffer_slice = &buffer;
        let deserialized = FloorPartConfig::load_from_slice(buffer_slice).unwrap();
        assert_eq!(config.payload_count, deserialized.payload_count);
        assert_eq!(&config.floor_data, &deserialized.floor_data);
        assert_eq!(&config.wall_data, &deserialized.wall_data);
        assert_eq!(&config.extra_data, &deserialized.extra_data);
    }
}