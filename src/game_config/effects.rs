use serde::{Deserialize, Serialize};
use crate::{
    effect_mechanics::{
        EffectRoot,
        nodes::ValueSource
    },
    game_config::{
        Config,
        ConfigId,
        parameters::{ParameterConfig, TagConfig}
    }
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EffectConfig {
}

impl Default for EffectConfig {
    fn default() -> Self {
        Self{
            // todo: поле со snarl
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum EffectGraphNode {
    EntryPoint(EntryPointEffectGraphNode),
    AddTag(AddTagEffectGraphNode),
    Branch(BranchEffectGraphNode),
    WaitForCondition(WaitForConditionEffectGraphNode),
    WaitForTicks(WaitForTicksEffectGraphNode),
    SpawnSubEffect(SpawnSubEffectGraphNode),
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct EntryPointEffectGraphNode {
    comment: String,
}
impl EntryPointEffectGraphNode {
    pub fn comment(&self) -> &str {
        &self.comment
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct AddTagEffectGraphNode {
    value_source: ValueSource,
    value_parameter_id: ConfigId<ParameterConfig>,
    tag_config_id: ConfigId<TagConfig>,
    comment: String,
}
impl AddTagEffectGraphNode {
    pub fn value_source(&self) -> &ValueSource {
        &self.value_source
    }
    pub fn value_parameter_id(&self) -> ConfigId<ParameterConfig> {
        self.value_parameter_id
    }
    pub fn tag_id(&self) -> ConfigId<TagConfig> {
        self.tag_config_id
    }
    pub fn comment(&self) -> &str {
        &self.comment
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct BranchEffectGraphNode {
    value_source: ValueSource,
    condition_parameter_id: ConfigId<ParameterConfig>,
    comment: String,
}
impl BranchEffectGraphNode {
    pub fn value_source(&self) -> &ValueSource {
        &self.value_source
    }
    pub fn condition_parameter_id(&self) -> ConfigId<ParameterConfig> {
        self.condition_parameter_id
    }
    pub fn comment(&self) -> &str {
        &self.comment
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct WaitForConditionEffectGraphNode {
    value_source: ValueSource,
    condition_parameter_id: ConfigId<ParameterConfig>,
    comment: String,
}
impl WaitForConditionEffectGraphNode {
    pub fn value_source(&self) -> &ValueSource {
        &self.value_source
    }
    pub fn condition_parameter_id(&self) -> ConfigId<ParameterConfig> {
        self.condition_parameter_id
    }
    pub fn comment(&self) -> &str {
        &self.comment
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct WaitForTicksEffectGraphNode {
    value_source: ValueSource,
    tick_count_parameter_id: ConfigId<ParameterConfig>,
    comment: String,
}
impl WaitForTicksEffectGraphNode {
    pub fn value_source(&self) -> &ValueSource {
        &self.value_source
    }
    pub fn tick_count_parameter_id(&self) -> ConfigId<ParameterConfig> {
        self.tick_count_parameter_id
    }
    pub fn comment(&self) -> &str {
        &self.comment
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct SpawnSubEffectGraphNode {
    effect_config_id: ConfigId<EffectConfig>,
    comment: String,
}
impl SpawnSubEffectGraphNode {
    pub fn effect_config_id(&self) -> ConfigId<EffectConfig> {
        self.effect_config_id
    }
    pub fn comment(&self) -> &str {
        &self.comment
    }
}

impl TryInto<EffectRoot> for &EffectConfig {
    type Error = ();

    fn try_into(self) -> Result<EffectRoot, Self::Error> {
        todo!("Реализовать создание корневого узла эффекта")
    }
}

impl Config for EffectConfig {}