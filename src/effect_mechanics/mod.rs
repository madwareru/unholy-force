use std::collections::{HashMap};
use std::hash::{DefaultHasher, Hash, Hasher};
use bumpalo::{Bump, collections::Vec};
use egui_snarl::NodeId;
use rand::RngExt;
use tracing::{error, warn};
use uuid::Uuid;
use crate::app::game_stage::{EntityId, GameWorld};
use crate::game_config::{ConfigId, ConfigProvider, ConfigProviderImpl};
use crate::game_config::effects::EffectConfig;
use crate::game_config::parameters::{CompiledExpressionParameterNode, ExpressionParameterNode, ParameterConfig, ParameterOperator, ParameterType, TagConfig};

#[derive(Default)]
pub struct EffectEvaluatorRegistry {
    bump: Bump,
    provider: EffectRootProvider
}

#[derive(Default)]
pub struct EffectRootProvider {
    effects: HashMap<ConfigId<EffectConfig>, EffectRoot>,
}
impl EffectRootProvider {
    fn get_or_create_effect_root(
        &mut self,
        game_config_provider: &ConfigProvider,
        effect_config_id: ConfigId<EffectConfig>
    ) -> Option<&EffectRoot> {
        if !self.effects.contains_key(&effect_config_id) {
            match game_config_provider.get_config(effect_config_id) {
                None => {
                    error!(
                        target: "Эффект граф",
                        "Не найден эффект с идентификатором {:?}",
                        effect_config_id
                    );
                }
                Some(effect_config) => {
                    match effect_config.create_root() {
                        Some(effect_evaluator) => {
                            self.effects.insert(
                                effect_config_id,
                                effect_evaluator
                            );
                        }
                        _ => {
                            error!(
                                target: "Эффект граф",
                                "Не удалось создать корневой узел для эффекта с идентификатором {:?}",
                                effect_config_id
                            );
                        }
                    }
                }
            }
        }
        self.effects.get(&effect_config_id)
    }
}

impl EffectEvaluatorRegistry {
    pub fn create_effect(
        &mut self,
        game_config_provider: &ConfigProvider,
        game_world: &mut GameWorld,
        effect_config_id: ConfigId<EffectConfig>,
        effect_context: EffectContext
    ) {
        let Some(root_node) = self.provider.get_or_create_effect_root(
            game_config_provider,
            effect_config_id
        ) else {
            return;
        };

        let bump = &self.bump;
        let mut queue = DelayedEffectQueue(Vec::new_in(bump));

        let effect_env = EffectEnv::new();
        let effect_id = game_world.spawn((effect_config_id, effect_env, effect_context));
        root_node.setup(game_world, effect_id, &mut queue);

        let mut offset = 0;
        while offset < queue.0.len() {
            let Some(DelayedEffect { effect_config_id, effect_context }) = queue.0.get(offset).copied() else {
                continue;
            };
            offset += 1;

            let Some(root_node) = self.provider.get_or_create_effect_root(
                game_config_provider,
                effect_config_id
            ) else {
                continue;
            };

            let effect_env = EffectEnv::new();
            let effect_id = game_world.spawn((effect_config_id, effect_env, effect_context));
            root_node.setup(game_world, effect_id, &mut queue);
        }
    }

    pub fn tick(
        &mut self,
        game_config_provider: &ConfigProvider,
        effect_id: EntityId,
        game_world: &mut GameWorld
    ) {
        let effect_config_id = {
            match game_world.get::<&ConfigId<EffectConfig>>(effect_id) {
                Ok(mechanic_setting) => *mechanic_setting,
                _ => panic!("No effect found for {:?}", effect_id)
            }
        };

        let Some(root_node) = self.provider.get_or_create_effect_root(
            game_config_provider,
            effect_config_id
        ) else {
            return;
        };

        let bump = &self.bump;
        let mut queue = DelayedEffectQueue(Vec::new_in(bump));

        if let EffectFlow::Complete = root_node.tick(game_world, effect_id, &mut queue) {
            root_node.on_destroy(game_world, effect_id, &mut queue);
            game_world.despawn(effect_id).expect("Failed to despawn effect");
        }

        let mut offset = 0;
        while offset < queue.0.len() {
            let Some(DelayedEffect { effect_config_id, effect_context }) = queue.0.get(offset).copied() else {
                continue;
            };
            offset += 1;

            let Some(root_node) = self.provider.get_or_create_effect_root(
                game_config_provider,
                effect_config_id
            ) else {
                continue;
            };

            let effect_env = EffectEnv::new();
            let effect_id = game_world.spawn((effect_config_id, effect_env, effect_context));
            root_node.setup(game_world, effect_id, &mut queue);
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct DelayedEffect {
    effect_config_id: ConfigId<EffectConfig>,
    effect_context: EffectContext
}

pub struct DelayedEffectQueue<'a>(Vec<'a, DelayedEffect>);
impl<'a> DelayedEffectQueue<'a> {
    pub fn push(&mut self, effect_config_id: ConfigId<EffectConfig>, effect_context: EffectContext) {
        self.0.push(DelayedEffect { effect_config_id, effect_context } );
    }
}

#[derive(Copy, Clone, Debug)]
pub enum EffectContext {
    WorldOnly,
    CasterOnly { id: EntityId },
    TargetOnly { id: EntityId },
    CasterAndTarget { caster_id: EntityId, target_id: EntityId }
}

pub enum EffectFlow {
    Continue,
    Complete
}

#[derive(Clone)]
pub struct EffectEnv {
    buckets: smallvec::SmallVec<[(u128, f32); 16]>,
}
impl EffectEnv {
    pub fn new() -> Self {
        Self { buckets: smallvec::SmallVec::new() }
    }

    pub fn get<N: EffectNode, H: Hash>(&self, node: &N, salt_hash: H) -> f32 {
        let id = get_node_hash(node, salt_hash);
        self.buckets.iter().find(|it| it.0 == id).map(|it| it.1).unwrap_or(0.0)
    }

    pub fn set<N: EffectNode, H: Hash>(&mut self, node: &N, salt_hash: H, value: f32) {
        let id = get_node_hash(node, salt_hash);
        if self.buckets.iter_mut().find(|it| it.0 == id).map(|it| it.1 = value).is_none() {
            self.buckets.push((id, value));
        }
    }
}

pub struct EntityValueHolder {
    tag_buckets: smallvec::SmallVec<[(Uuid, f32); 16]>,
    parameter_buckets: smallvec::SmallVec<[(Uuid, f32); 16]>,
}

impl EntityValueHolder {
    pub fn new(parameter_buckets: smallvec::SmallVec<[(Uuid, f32); 16]>) -> Self {
        Self {
            tag_buckets: smallvec::SmallVec::new(),
            parameter_buckets
        }
    }

    pub fn get_parameter_value(&self, parameter_config_id: ConfigId<ParameterConfig>) -> f32 {
        self.parameter_buckets
            .iter()
            .find(|it| it.0 == parameter_config_id.uuid)
            .map_or(0f32, |it| it.1)
    }

    pub fn count_tag(&self, tag_config_id: ConfigId<TagConfig>) -> f32 {
        self.tag_buckets
            .iter()
            .find(|it| it.0 == tag_config_id.uuid)
            .map_or(0f32, |it| it.1)
    }

    pub fn increment_tag(&mut self, tag_config_id: ConfigId<TagConfig>) {
        if self.tag_buckets
            .iter_mut()
            .find(|it| it.0 == tag_config_id.uuid).map(|it| it.1 += 1.0)
            .is_none() {
            self.tag_buckets.push((tag_config_id.uuid, 1.0));
        }
    }

    pub fn decrement_tag(&mut self, tag_config_id: ConfigId<TagConfig>) {
        if self.count_tag(tag_config_id) > 1f32 {
            self.tag_buckets
                .iter_mut()
                .find(|it| it.0 == tag_config_id.uuid)
                .map(|it| it.1 -= 1.0);
            return;
        }
        self.tag_buckets.retain(|it| it.0 != tag_config_id.uuid);
    }
}

pub fn get_entity_parameter_value(
    game_config_provider: &ConfigProvider,
    game_world: &GameWorld,
    entity: EntityId,
    parameter_config_id: ConfigId<ParameterConfig>,
) -> f32 {
    let Ok(holder) = game_world.get::<&EntityValueHolder>(entity) else {
        return 0f32;
    };
    if let Some(parameter_config) = game_config_provider.get_config(parameter_config_id) {
        match parameter_config.parameter_type {
            ParameterType::Constant => {
                return holder.get_parameter_value(parameter_config_id);
            },
            ParameterType::Expression(_) => {
                fn eval_expression(
                    game_config_provider: &ConfigProvider,
                    game_world: &GameWorld,
                    entity: EntityId,
                    parameter_config_id: ConfigId<ParameterConfig>,
                    expr_param_node: &ExpressionParameterNode
                ) -> f32 {
                    match expr_param_node {
                        ExpressionParameterNode::ParameterValue(param_config_id) => {
                            if *param_config_id == parameter_config_id {
                                error!(
                                    target: "Эффект граф",
                                    "Обнаружено рекурсивное выражение в черте с идентификатором {}",
                                    parameter_config_id.uuid
                                );
                                return 0f32;
                            }
                            get_entity_parameter_value(
                                game_config_provider,
                                game_world,
                                entity,
                                *param_config_id
                            )
                        }
                        ExpressionParameterNode::TagCount(tag_config_id) => {
                            get_entity_tag_count(
                                game_world,
                                entity,
                                *tag_config_id
                            )
                        }
                        ExpressionParameterNode::Constant(value) => {
                            *value
                        }
                        ExpressionParameterNode::Operator(op, operands) => {
                            match op {
                                ParameterOperator::Plus => {
                                    if operands.is_empty() {
                                        error!(
                                            target: "Эффект граф",
                                            "Обнаружен оператор + с пустым списком операндов в черте с идентификатором {}",
                                            parameter_config_id.uuid
                                        );
                                        return 0f32;
                                    }
                                    let acc = &operands[0];
                                    let mut acc = eval_expression(
                                        game_config_provider,
                                        game_world,
                                        entity,
                                        parameter_config_id,
                                        acc
                                    );
                                    let rest = &operands[1..];
                                    for next in rest.iter() {
                                        acc += eval_expression(
                                            game_config_provider,
                                            game_world,
                                            entity,
                                            parameter_config_id,
                                            next
                                        );
                                    }
                                    acc
                                }
                                ParameterOperator::Minus => {
                                    if operands.is_empty() {
                                        error!(
                                            target: "Эффект граф",
                                            "Обнаружен оператор - с пустым списком операндов в черте с идентификатором {}",
                                            parameter_config_id.uuid
                                        );
                                        return 0f32;
                                    }
                                    let acc = &operands[0];
                                    let mut acc = eval_expression(
                                        game_config_provider,
                                        game_world,
                                        entity,
                                        parameter_config_id,
                                        acc
                                    );
                                    let rest = &operands[1..];
                                    if rest.is_empty() {
                                        return -acc;
                                    }
                                    for next in rest.iter() {
                                        acc -= eval_expression(
                                            game_config_provider,
                                            game_world,
                                            entity,
                                            parameter_config_id,
                                            next
                                        );
                                    }
                                    acc
                                }
                                ParameterOperator::Mul => {
                                    if operands.len() < 2 {
                                        error!(
                                            target: "Эффект граф",
                                            "Обнаружен оператор * со списком операндов арности меньше 2 в черте с идентификатором {}",
                                            parameter_config_id.uuid
                                        );
                                        return 0f32;
                                    }
                                    let mut acc = eval_expression(
                                        game_config_provider,
                                        game_world,
                                        entity,
                                        parameter_config_id,
                                        &operands[0]
                                    );
                                    acc *= eval_expression(
                                        game_config_provider,
                                        game_world,
                                        entity,
                                        parameter_config_id,
                                        &operands[1]
                                    );
                                    let rest = &operands[2..];
                                    for next in rest.iter() {
                                        acc *= eval_expression(
                                            game_config_provider,
                                            game_world,
                                            entity,
                                            parameter_config_id,
                                            next
                                        );
                                    }
                                    acc
                                }
                                ParameterOperator::Div => {
                                    if operands.len() < 2 {
                                        error!(
                                            target: "Эффект граф",
                                            "Обнаружен оператор / со списком операндов арности меньше 2 в черте с идентификатором {}",
                                            parameter_config_id.uuid
                                        );
                                        return 0f32;
                                    }
                                    let mut acc = eval_expression(
                                        game_config_provider,
                                        game_world,
                                        entity,
                                        parameter_config_id,
                                        &operands[0]
                                    );
                                    acc /= eval_expression(
                                        game_config_provider,
                                        game_world,
                                        entity,
                                        parameter_config_id,
                                        &operands[1]
                                    );
                                    let rest = &operands[2..];
                                    for next in rest.iter() {
                                        acc /= eval_expression(
                                            game_config_provider,
                                            game_world,
                                            entity,
                                            parameter_config_id,
                                            next
                                        );
                                    }
                                    acc
                                }
                                ParameterOperator::Clamp => {
                                    if operands.len() != 3 {
                                        error!(
                                            target: "Эффект граф",
                                            "Обнаружен оператор clamp со списком операндов арности != 3 в черте с идентификатором {}",
                                            parameter_config_id.uuid
                                        );
                                        return 0f32;
                                    }
                                    let v = eval_expression(
                                        game_config_provider,
                                        game_world,
                                        entity,
                                        parameter_config_id,
                                        &operands[0]
                                    );
                                    let min = eval_expression(
                                        game_config_provider,
                                        game_world,
                                        entity,
                                        parameter_config_id,
                                        &operands[1]
                                    );
                                    let max = eval_expression(
                                        game_config_provider,
                                        game_world,
                                        entity,
                                        parameter_config_id,
                                        &operands[2]
                                    );
                                    v.clamp(min, max)
                                }
                                ParameterOperator::Min => {
                                    if operands.len() != 2 {
                                        error!(
                                            target: "Эффект граф",
                                            "Обнаружен оператор min со списком операндов арности != 2 в черте с идентификатором {}",
                                            parameter_config_id.uuid
                                        );
                                        return 0f32;
                                    }
                                    let lhs = eval_expression(
                                        game_config_provider,
                                        game_world,
                                        entity,
                                        parameter_config_id,
                                        &operands[0]
                                    );
                                    let rhs = eval_expression(
                                        game_config_provider,
                                        game_world,
                                        entity,
                                        parameter_config_id,
                                        &operands[1]
                                    );
                                    lhs.min(rhs)
                                }
                                ParameterOperator::Max => {
                                    if operands.len() != 2 {
                                        error!(
                                            target: "Эффект граф",
                                            "Обнаружен оператор max со списком операндов арности != 2 в черте с идентификатором {}",
                                            parameter_config_id.uuid
                                        );
                                        return 0f32;
                                    }
                                    let lhs = eval_expression(
                                        game_config_provider,
                                        game_world,
                                        entity,
                                        parameter_config_id,
                                        &operands[0]
                                    );
                                    let rhs = eval_expression(
                                        game_config_provider,
                                        game_world,
                                        entity,
                                        parameter_config_id,
                                        &operands[1]
                                    );
                                    lhs.max(rhs)
                                }
                                ParameterOperator::Round => {
                                    if operands.len() != 1 {
                                        error!(
                                            target: "Эффект граф",
                                            "Обнаружен оператор round со списком операндов арности != 1 в черте с идентификатором {}",
                                            parameter_config_id.uuid
                                        );
                                        return 0f32;
                                    }
                                    eval_expression(
                                        game_config_provider,
                                        game_world,
                                        entity,
                                        parameter_config_id,
                                        &operands[0]
                                    ).round()
                                }
                                ParameterOperator::Rand => {
                                    if operands.len() != 2 {
                                        error!(
                                            target: "Эффект граф",
                                            "Обнаружен оператор rand со списком операндов арности != 2 в черте с идентификатором {}",
                                            parameter_config_id.uuid
                                        );
                                        return 0f32;
                                    }
                                    let mut rng = rand::rng();
                                    let start = eval_expression(
                                        game_config_provider,
                                        game_world,
                                        entity,
                                        parameter_config_id,
                                        &operands[0]
                                    );
                                    let end = eval_expression(
                                        game_config_provider,
                                        game_world,
                                        entity,
                                        parameter_config_id,
                                        &operands[1]
                                    );
                                    rng.random_range(start..=end)
                                }
                            }
                        }
                    }
                }

                match parameter_config.compiled_expression() {
                    CompiledExpressionParameterNode::Ok(expr_param_node) => {
                        return eval_expression(
                            game_config_provider,
                            game_world,
                            entity,
                            parameter_config_id,
                            expr_param_node
                        );
                    },
                    _ => {}
                }
            }
        }
    }
    0f32
}

pub fn get_entity_tag_count(
    game_world: &GameWorld,
    entity: EntityId,
    tag_config_id: ConfigId<TagConfig>,
) -> f32 {
    let Ok(holder) = game_world.get::<&EntityValueHolder>(entity) else {
        return 0f32;
    };
    holder.count_tag(tag_config_id)
}

pub fn increment_entity_tag_count(
    game_config_provider: &ConfigProvider,
    game_world: &mut GameWorld,
    entity: EntityId,
    tag_config_id: ConfigId<TagConfig>,
    delayed_effect_queue: &mut DelayedEffectQueue
) {
    let Ok(mut holder) = game_world.get::<&mut EntityValueHolder>(entity) else {
        return;
    };
    if get_entity_tag_count(game_world, entity, tag_config_id) < 1f32 {
        if let Some(tag_config) = game_config_provider.get_config(tag_config_id) {
            if let Some(effect_config_id) = tag_config.effect_mechanic {
                delayed_effect_queue.push(
                    effect_config_id,
                    EffectContext::TargetOnly { id: entity }
                )
            }
        }
    }
    holder.increment_tag(tag_config_id);
}

pub fn decrement_entity_tag_count(
    game_world: &mut GameWorld,
    entity: EntityId,
    tag_config_id: ConfigId<TagConfig>
) {
    let Ok(mut holder) = game_world.get::<&mut EntityValueHolder>(entity) else {
        return;
    };
    holder.decrement_tag(tag_config_id);
}

pub fn get_node_hash<N: EffectNode, H: Hash>(node: &N, salt_hash: H) -> u128 {
    let mut salt_hasher = DefaultHasher::new();
    salt_hash.hash(&mut salt_hasher);
    let salt_hash = salt_hasher.finish() as u128;

    let mut hasher = DefaultHasher::new();
    node.get_node_id().hash(&mut hasher);
    let result_hash = (hasher.finish() & 0xFFFF_FFFF) as u128;

    let pos = node.get_node_pos();
    let result_hash = (result_hash << 16) | ((pos.x as u128) & 0xFFFF);
    let result_hash = (result_hash << 16) | ((pos.y as u128) & 0xFFFF);

    let result_hash = (result_hash << 64) | salt_hash;
    result_hash
}

pub trait EffectNode {
    fn get_node_id(&self) -> NodeId;
    fn get_node_pos(&self) -> egui::Pos2;
    fn tick(
        &self,
        game_world: &mut GameWorld,
        effect_id: EntityId,
        delayed_effect_queue: &mut DelayedEffectQueue
    ) -> EffectFlow;
}

pub struct EffectRoot {
    setup: Option<Box<dyn EffectNode>>,
    tick: Box<dyn EffectNode>,
    on_destroy: Option<Box<dyn EffectNode>>,
}

impl EffectRoot {
    pub fn new(
        setup: Option<Box<dyn EffectNode>>,
        tick: Box<dyn EffectNode>,
        on_destroy: Option<Box<dyn EffectNode>>,
    ) -> Self {
        Self { setup, tick, on_destroy }
    }

    pub fn setup(
        &self,
        game_world: &mut GameWorld,
        effect_id: EntityId,
        delayed_effect_queue: &mut DelayedEffectQueue
    ) {
        let Some(setup) = &self.setup else {
            return;
        };

        match setup.tick(game_world, effect_id, delayed_effect_queue) {
            EffectFlow::Continue => {
                warn!(
                    target: "Эффект граф",
                    "В слот установки подключен узел, ожидающийся для работы в несколько тиков. Подобное поведение \
                    не поддерживается и скорее всего ожидаемый результат будет некорректным. Для достижения вызова \
                    продолжающегося поведения во время установки, нужно использовать узел, порождающий дочерний эффект."
                );
            }
            EffectFlow::Complete => {}
        }
    }

    pub fn tick(
        &self,
        game_world: &mut GameWorld,
        effect_id: EntityId,
        delayed_effect_queue: &mut DelayedEffectQueue
    ) -> EffectFlow {
        self.tick.tick(game_world, effect_id, delayed_effect_queue)
    }

    pub fn on_destroy(
        &self,
        game_world: &mut GameWorld,
        effect_id: EntityId,
        delayed_effect_queue: &mut DelayedEffectQueue
    ) {
        let Some(on_destroy) = &self.on_destroy else {
            return;
        };

        match on_destroy.tick(game_world, effect_id, delayed_effect_queue) {
            EffectFlow::Continue => {
                warn!(
                    target: "Эффект граф",
                    "В слот очистки подключен узел, ожидающийся для работы в несколько тиков. Подобное поведение \
                    не поддерживается и скорее всего ожидаемый результат будет некорректным. Для достижения вызова \
                    продолжающегося поведения во время очистки, нужно использовать узел, порождающий дочерний эффект."
                );
            }
            EffectFlow::Complete => {}
        }
    }
}