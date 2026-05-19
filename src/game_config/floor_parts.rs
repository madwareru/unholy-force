use crate::game_config::{Config, ConfigId};
use crate::game_config::items::{ItemConfig, ItemRarity};
use crate::game_config::units::{UnitConfig, UnitDanger};
use crate::graphics::{FloorGraphicsTileGroup, WallGraphicsTileGroup};
use std::io::{Result, Error, ErrorKind, Write, Read};
use uuid::Uuid;

#[derive(Copy, Clone, Debug)]
pub enum FloorCellExtra {
    None,
    SpawnUnitHint(UnitDanger), // 1..6
    SpawnLootHint(ItemRarity), // 7..10
    SpawnUnit(ConfigId<UnitConfig>), // 11
    SpawnLoot(ConfigId<ItemConfig>), // 12
    LadderDownHint,
    LadderUpHint,
    PlayerStartHint,
    TriggerEffect(ConfigId<ItemConfig>)
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

#[derive(Debug, Default)]
pub struct FloorPartConfig {
    pub floor_data: [[FloorGraphicsTileGroup; 5]; 5],
    pub wall_data: [[WallGraphicsTileGroup; 5]; 5],
    pub extra_data: [[FloorCellExtra; 5]; 5],
    payload_count: u8,
}

impl FloorPartConfig {
    pub fn write(&self, writer: &mut impl Write) -> Result<()> {
        if self.payload_count > 4 {
            return Err(Error::new(ErrorKind::InvalidInput, "Payload count exceeds maximum"));
        }

        writer.write(&[self.payload_count])?;

        if self.payload_count > 0 {
            for j in 0..5 {
                for i in 0..5 {
                    match self.extra_data[j][i].get_payload() {
                        Some(uuid) => {
                            for b in uuid.as_bytes() {
                                writer.write(&[*b])?;
                            }
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
    pub fn load_from_slice(&mut self, data: &[u8]) -> Result<Self> {
        if data.is_empty() {
            return Err(Error::new(ErrorKind::InvalidInput, "Empty data"));
        }

        let payload_count;
        let size_expected = if data[0] == 0 {
            payload_count = 0;
            30
        } else if data[0] <= 4 {
            payload_count = data[0];
            30 + (payload_count as usize) * 16
        } else {
            return Err(Error::new(ErrorKind::InvalidInput, "Invalid payload count"));
        };

        if size_expected < data.len() {
            return Err(Error::new(ErrorKind::InvalidInput, "Invalid data length"));
        }

        let payload_data = &data[1..];
        let cells_data = &payload_data[(payload_count as usize) * 16..];
        let mut effect_spawner_mask_data = &cells_data[25..];

        let mut effect_spawner_mask = 0u32;
        for i in 0..4 {
            let mut mask_part = [0u8];
            effect_spawner_mask_data.read(&mut mask_part)?;
            effect_spawner_mask = effect_spawner_mask | ((mask_part[0] as u32) << (i * 8));
        }

        let floor_data: [[FloorGraphicsTileGroup; 5]; 5] = Default::default();
        let wall_data: [[WallGraphicsTileGroup; 5]; 5] = Default::default();
        let extra_data: [[FloorCellExtra; 5]; 5] = Default::default();

        todo!()
    }
}

impl Config for FloorPartConfig {}