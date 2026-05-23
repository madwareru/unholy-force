use std::collections::HashMap;
use std::marker::PhantomData;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::game_config::effects::EffectMechanicConfig;
use crate::game_config::floor_part_adjacency::FloorPartAdjacencyConfig;
use crate::game_config::floor_parts::FloorPartConfig;
use crate::game_config::floors::FloorConfig;
use crate::game_config::items::ItemConfig;
use crate::game_config::parameters::ParameterConfig;
use crate::game_config::parameters::TagConfig;
use crate::game_config::units::UnitConfig;

pub mod units;
pub mod items;
pub mod floor_parts;
pub mod floor_part_adjacency;
pub mod floors;
pub mod floor_flow_graph;
pub mod parameters;
pub mod effects;

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigId<T: Config> {
    pub uuid: Uuid,
    _phantom: PhantomData<T>,
}

impl <T: Config> PartialEq for ConfigId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl <T: Config> Clone for ConfigId<T> {
    fn clone(&self) -> Self {
        Self {
            uuid: self.uuid,
            _phantom: PhantomData
        }
    }
}
impl <T: Config> Copy for ConfigId<T> {}

impl<T: Config> ConfigId<T> {
    pub const INVALID: ConfigId<T> = ConfigId {
        uuid: Uuid::nil(),
        _phantom: PhantomData
    };

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self {
            uuid,
            _phantom: PhantomData
        }
    }
}

impl<T: Config> Default for ConfigId<T> {
    fn default() -> Self {
        Self::INVALID
    }
}

pub trait Config : Clone {}

pub trait ConfigProvider<T: Config> {
    fn get_config(&self, config_id: ConfigId<T>) -> Option<&T>;
}

pub struct GameConfigProvider {
    units: HashMap<Uuid, UnitConfig>,
    items: HashMap<Uuid, ItemConfig>,
    floor_parts: HashMap<Uuid, FloorPartConfig>,
    floor_parts_adjacency: HashMap<Uuid, FloorPartAdjacencyConfig>,
    floors: HashMap<Uuid, FloorConfig>,
    effect_mechanics: HashMap<Uuid, EffectMechanicConfig>,
    parameters: HashMap<Uuid, ParameterConfig>,
    tags: HashMap<Uuid, TagConfig>,
}

macro_rules! impl_provider {
    ($x:ident <- $y:ident) => {
        impl ConfigProvider<$x> for GameConfigProvider {
            fn get_config(&self, config_id: ConfigId<$x>) -> Option<&$x> {
                self.$y.get(&config_id.uuid)
            }
        }
    };
}

impl_provider!(UnitConfig <- units);
impl_provider!(ItemConfig <- items);
impl_provider!(FloorPartConfig <- floor_parts);
impl_provider!(FloorPartAdjacencyConfig <- floor_parts_adjacency);
impl_provider!(FloorConfig <- floors);
impl_provider!(EffectMechanicConfig <- effect_mechanics);
impl_provider!(ParameterConfig <- parameters);
impl_provider!(TagConfig <- tags);