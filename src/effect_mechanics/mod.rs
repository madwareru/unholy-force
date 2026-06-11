use std::{
    collections::{HashMap},
    hash::{DefaultHasher, Hash, Hasher}
};
use std::fmt::Debug;
use bumpalo::{Bump};
use rand::RngExt;
use tracing::{error, info, warn};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::{
    effect_mechanics::{
        nodes::{
            get_value_holder,
            get_value_holder_mut,
            add_tag::AddTagNode,
            branch::BranchNode,
            effect_context_is_expired,
            spawn_sub_effect::SpawnSubEffectNode,
            wait_cond::WaitForConditionNode,
            wait_ticks::WaitTicksNode
        }
    },
    app::game_stage::{EntityId, GameWorld},
    game_config::{
        ConfigId,
        ConfigProvider,
        ConfigProviderImpl,
        effects::EffectConfig,
        parameters::{
            CompiledExpression,
            ExpressionParameterNode,
            ParameterConfig,
            ParameterOperator,
            ParameterType,
            TagConfig
        }
    },
};

pub mod nodes;

pub const EFFECT_GRAPH_TARGET: &str = "Эффект граф";

#[derive(Default)]
pub struct EffectRegistry {
    effect_roots: HashMap<ConfigId<EffectConfig>, EffectRoot>
}

impl EffectRegistry {
    fn get_or_create_effect_root(
        &mut self,
        game_config_provider: &ConfigProvider,
        effect_config_id: ConfigId<EffectConfig>
    ) -> Option<&EffectRoot> {
        if !self.effect_roots.contains_key(&effect_config_id) {
            match game_config_provider.get_config(effect_config_id) {
                None => {
                    error!(
                        target: EFFECT_GRAPH_TARGET,
                        "Не найден эффект с идентификатором {:?}",
                        effect_config_id
                    );
                }
                Some(effect_config) => {
                    self.effect_roots.insert(effect_config_id, effect_config.root_node());
                }
            }
        }
        self.effect_roots.get(&effect_config_id)
    }

    pub fn create_effect(
        &mut self,
        bump: &Bump,
        game_config_provider: &ConfigProvider,
        game_world: &mut GameWorld,
        effect_config_id: ConfigId<EffectConfig>,
        effect_context: EffectContext
    ) {
        let mut queue = EffectQueue::new_in(bump);

        queue.push(effect_config_id, effect_context);

        let mut offset = 0;
        while offset < queue.0.len() {
            let Some(ScheduledEffect { effect_config_id, effect_context }) = queue.0.get(offset).copied() else {
                continue;
            };
            offset += 1;

            let Some(root_node) = self.get_or_create_effect_root(
                game_config_provider,
                effect_config_id
            ) else {
                continue;
            };

            let effect_env = EffectEnv::new();
            let effect_id = game_world.spawn((effect_config_id, effect_env, effect_context));
            root_node.setup(game_config_provider, game_world, effect_id, &mut queue);
        }
    }

    pub fn tick(
        &mut self,
        bump: &Bump,
        game_config_provider: &ConfigProvider,
        effect_id: EntityId,
        mut current_node_id: Option<EffectNodeId>,
        game_world: &mut GameWorld
    ) -> Option<EffectNodeId> {
        let effect_config_id = {
            match game_world.get::<&ConfigId<EffectConfig>>(effect_id) {
                Ok(mechanic_setting) => *mechanic_setting,
                _ => panic!("No effect found for {:?}", effect_id)
            }
        };

        let Some(root_node) = self.get_or_create_effect_root(
            game_config_provider,
            effect_config_id
        ) else {
            return None;
        };

        let mut queue = EffectQueue(bumpalo::collections::Vec::new_in(bump));

        'chain: loop {
            let result = root_node.tick(
                game_config_provider,
                game_world,
                effect_id,
                current_node_id,
                &mut queue
            );

            match result {
                EffectControlFlow::Complete => {
                    root_node.on_destroy(game_config_provider, game_world, effect_id, &mut queue);
                    game_world.despawn(effect_id).expect("Failed to despawn effect");
                    return None;
                }
                EffectControlFlow::AndThen(then_node_id) => {
                    current_node_id = Some(then_node_id);
                    continue 'chain;
                }
                EffectControlFlow::Suspend => {
                    break 'chain;
                }
            }
        }

        let mut offset = 0;
        while offset < queue.0.len() {
            let Some(ScheduledEffect { effect_config_id, effect_context }) = queue.0.get(offset).copied() else {
                continue;
            };
            offset += 1;

            let Some(root_node) = self.get_or_create_effect_root(
                game_config_provider,
                effect_config_id
            ) else {
                continue;
            };

            let effect_env = EffectEnv::new();
            let effect_id = game_world.spawn((effect_config_id, effect_env, effect_context));
            root_node.setup(game_config_provider, game_world, effect_id, &mut queue);
        }

        current_node_id
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ScheduledEffect {
    effect_config_id: ConfigId<EffectConfig>,
    effect_context: EffectContext
}

impl ScheduledEffect {
    pub fn effect_config_id(&self) -> ConfigId<EffectConfig> {
        self.effect_config_id
    }
    pub fn effect_context(&self) -> EffectContext {
        self.effect_context
    }
}

pub struct EffectQueue<'a>(bumpalo::collections::Vec<'a, ScheduledEffect>);
impl<'a> EffectQueue<'a> {
    pub fn new_in(bump: &'a Bump) -> Self {
        EffectQueue(bumpalo::collections::Vec::new_in(bump))
    }

    pub fn push(
        &mut self,
        effect_config_id: ConfigId<EffectConfig>,
        effect_context: EffectContext
    ) {
        self.0.push(ScheduledEffect { effect_config_id, effect_context } );
    }

    pub fn drain(&mut self) -> impl Iterator<Item =ScheduledEffect> {
        self.0.drain(..)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct EffectContext {
    pub current_node_id: Option<EffectNodeId>,
    caster_id: EntityId,
    target_id: EntityId
}

pub enum EffectCompletionCause {
    Finished,
    Failed
}

pub trait EffectFlowExt<T> {
    fn continue_execution(v: T) -> Self;
    fn complete_execution() -> Self;
    fn fail_execution() -> Self;
}

pub enum EffectControlFlow {
    Suspend,
    AndThen(EffectNodeId),
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

    pub fn get<N: EffectNodeImpl, H: Hash>(&self, node: &N, salt_hash: H) -> Option<f32> {
        let id = get_node_hash(node, salt_hash);
        self.buckets.iter().find(|it| it.0 == id).map(|it| it.1)
    }

    pub fn set<N: EffectNodeImpl, H: Hash>(&mut self, node: &N, salt_hash: H, value: f32) {
        let id = get_node_hash(node, salt_hash);
        if self.buckets.iter_mut().find(|it| it.0 == id).map(|it| it.1 = value).is_none() {
            self.buckets.push((id, value));
        }
    }
}

pub struct GlobalValuesHolder;
impl GlobalValuesHolder {
    pub fn get_entity_id(game_world: &GameWorld) -> Option<EntityId> {
        game_world
            .query::<(EntityId, &GlobalValuesHolder)>()
            .iter()
            .next()
            .map(|(id, _)| id)
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
}

pub fn get_entity_parameter_value(
    game_config_provider: &ConfigProvider,
    game_world: &GameWorld,
    entity_id: EntityId,
    parameter_config_id: ConfigId<ParameterConfig>,
) -> Option<f32> {
    fn eval_expression(
        game_config_provider: &ConfigProvider,
        game_world: &GameWorld,
        entity_id: EntityId,
        parameter_config_id: ConfigId<ParameterConfig>,
        expr_param_node: &ExpressionParameterNode
    ) -> Option<f32> {
        match expr_param_node {
            ExpressionParameterNode::ParameterValue(param_config_id) => {
                if *param_config_id == parameter_config_id {
                    error!(
                        target: EFFECT_GRAPH_TARGET,
                        "Обнаружено рекурсивное выражение в черте с идентификатором {}",
                        parameter_config_id.uuid
                    );
                    return None;
                }
                get_entity_parameter_value(
                    game_config_provider,
                    game_world,
                    entity_id,
                    *param_config_id
                )
            }
            ExpressionParameterNode::TagCount(tag_config_id) => {
                get_entity_tag_count(
                    game_world,
                    entity_id,
                    *tag_config_id
                )
            }
            ExpressionParameterNode::Constant(value) => {
                Some(*value)
            }
            ExpressionParameterNode::Operator(op, operands) => {
                match op {
                    ParameterOperator::Plus => {
                        if operands.is_empty() {
                            error!(
                                target: EFFECT_GRAPH_TARGET,
                                "Обнаружен оператор + с пустым списком операндов в черте с идентификатором {}",
                                parameter_config_id.uuid
                            );
                            return None;
                        }
                        let mut acc = eval_expression(
                            game_config_provider,
                            game_world,
                            entity_id,
                            parameter_config_id,
                            operands.get(0)?
                        )?;
                        let rest = &operands[1..];
                        for next in rest.iter() {
                            acc += eval_expression(
                                game_config_provider,
                                game_world,
                                entity_id,
                                parameter_config_id,
                                next
                            )?;
                        }
                        Some(acc)
                    }
                    ParameterOperator::Minus => {
                        if operands.is_empty() {
                            error!(
                                target: EFFECT_GRAPH_TARGET,
                                "Обнаружен оператор - с пустым списком операндов в черте с идентификатором {}",
                                parameter_config_id.uuid
                            );
                            return None;
                        }
                        let mut acc = eval_expression(
                            game_config_provider,
                            game_world,
                            entity_id,
                            parameter_config_id,
                            operands.get(0)?
                        )?;
                        let rest = &operands[1..];
                        if rest.is_empty() {
                            return Some(-acc);
                        }
                        for next in rest.iter() {
                            acc -= eval_expression(
                                game_config_provider,
                                game_world,
                                entity_id,
                                parameter_config_id,
                                next
                            )?;
                        }
                        Some(acc)
                    }
                    ParameterOperator::Mul => {
                        if operands.len() < 2 {
                            error!(
                                target: EFFECT_GRAPH_TARGET,
                                "Обнаружен оператор * со списком операндов арности меньше 2 в черте с идентификатором {}",
                                parameter_config_id.uuid
                            );
                            return None;
                        }
                        let mut acc = eval_expression(
                            game_config_provider,
                            game_world,
                            entity_id,
                            parameter_config_id,
                            operands.get(0)?
                        )?;
                        acc *= eval_expression(
                            game_config_provider,
                            game_world,
                            entity_id,
                            parameter_config_id,
                            operands.get(1)?
                        )?;
                        let rest = &operands[2..];
                        for next in rest.iter() {
                            acc *= eval_expression(
                                game_config_provider,
                                game_world,
                                entity_id,
                                parameter_config_id,
                                next
                            )?;
                        }
                        Some(acc)
                    }
                    ParameterOperator::Div => {
                        if operands.len() < 2 {
                            error!(
                                target: EFFECT_GRAPH_TARGET,
                                "Обнаружен оператор / со списком операндов арности меньше 2 в черте с идентификатором {}",
                                parameter_config_id.uuid
                            );
                            return None;
                        }
                        let mut acc = eval_expression(
                            game_config_provider,
                            game_world,
                            entity_id,
                            parameter_config_id,
                            operands.get(1)?
                        )?;
                        acc /= eval_expression(
                            game_config_provider,
                            game_world,
                            entity_id,
                            parameter_config_id,
                            operands.get(1)?
                        )?;
                        let rest = &operands[2..];
                        for next in rest.iter() {
                            acc /= eval_expression(
                                game_config_provider,
                                game_world,
                                entity_id,
                                parameter_config_id,
                                next
                            )?;
                        }
                        Some(acc)
                    }
                    ParameterOperator::Clamp => {
                        if operands.len() != 3 {
                            error!(
                                target: EFFECT_GRAPH_TARGET,
                                "Обнаружен оператор clamp со списком операндов арности != 3 в черте с идентификатором {}",
                                parameter_config_id.uuid
                            );
                            return None;
                        }
                        let v = eval_expression(
                            game_config_provider,
                            game_world,
                            entity_id,
                            parameter_config_id,
                            operands.get(0)?
                        )?;
                        let min = eval_expression(
                            game_config_provider,
                            game_world,
                            entity_id,
                            parameter_config_id,
                            operands.get(1)?
                        )?;
                        let max = eval_expression(
                            game_config_provider,
                            game_world,
                            entity_id,
                            parameter_config_id,
                            operands.get(2)?
                        )?;
                        Some(v.clamp(min, max))
                    }
                    ParameterOperator::Min => {
                        if operands.len() != 2 {
                            error!(
                                target: EFFECT_GRAPH_TARGET,
                                "Обнаружен оператор min со списком операндов арности != 2 в черте с идентификатором {}",
                                parameter_config_id.uuid
                            );
                            return None;
                        }
                        let lhs = eval_expression(
                            game_config_provider,
                            game_world,
                            entity_id,
                            parameter_config_id,
                            operands.get(0)?
                        )?;
                        let rhs = eval_expression(
                            game_config_provider,
                            game_world,
                            entity_id,
                            parameter_config_id,
                            operands.get(1)?
                        )?;
                        Some(lhs.min(rhs))
                    }
                    ParameterOperator::Max => {
                        if operands.len() != 2 {
                            error!(
                                target: EFFECT_GRAPH_TARGET,
                                "Обнаружен оператор max со списком операндов арности != 2 в черте с идентификатором {}",
                                parameter_config_id.uuid
                            );
                            return None;
                        }
                        let lhs = eval_expression(
                            game_config_provider,
                            game_world,
                            entity_id,
                            parameter_config_id,
                            operands.get(0)?
                        )?;
                        let rhs = eval_expression(
                            game_config_provider,
                            game_world,
                            entity_id,
                            parameter_config_id,
                            operands.get(1)?
                        )?;
                        Some(lhs.max(rhs))
                    }
                    ParameterOperator::Round => {
                        if operands.len() != 1 {
                            error!(
                                target: EFFECT_GRAPH_TARGET,
                                "Обнаружен оператор round со списком операндов арности != 1 в черте с идентификатором {}",
                                parameter_config_id.uuid
                            );
                            return None;
                        }
                        let rounded = eval_expression(
                            game_config_provider,
                            game_world,
                            entity_id,
                            parameter_config_id,
                            operands.get(0)?
                        )?.round();
                        Some(rounded)
                    }
                    ParameterOperator::Rand => {
                        if operands.len() != 2 {
                            error!(
                                target: EFFECT_GRAPH_TARGET,
                                "Обнаружен оператор rand со списком операндов арности != 2 в черте с идентификатором {}",
                                parameter_config_id.uuid
                            );
                            return None;
                        }
                        let mut rng = rand::rng();
                        let start = eval_expression(
                            game_config_provider,
                            game_world,
                            entity_id,
                            parameter_config_id,
                            operands.get(0)?
                        )?;
                        let end = eval_expression(
                            game_config_provider,
                            game_world,
                            entity_id,
                            parameter_config_id,
                            operands.get(1)?
                        )?;
                        let rand_value = rng.random_range(start..=end);
                        Some(rand_value)
                    }
                }
            }
        }
    }
    let Some(holder) = get_value_holder(game_world, entity_id) else {
        error!(
            target: EFFECT_GRAPH_TARGET,
            "Попытка вычислить выражение провалилась, так как у сущности нет хранилища значений"
        );
        return None;
    };
    match game_config_provider.get_config(parameter_config_id) {
        Some(parameter_config) => {
            match parameter_config.parameter_type {
                ParameterType::Constant => {
                    let parameter_value =
                        holder
                            .parameter_buckets
                            .iter()
                            .find(|it| it.0 == parameter_config_id.uuid)
                            .map_or(0f32, |it| it.1);
                    Some(parameter_value)
                },
                ParameterType::Expression(_) => {
                    match parameter_config.compiled_expression() {
                        CompiledExpression::Ok(expr_param_node) => {
                            eval_expression(
                                game_config_provider,
                                game_world,
                                entity_id,
                                parameter_config_id,
                                expr_param_node
                            )
                        },
                        CompiledExpression::Error { compile_error } => {
                            error!(
                                target: EFFECT_GRAPH_TARGET,
                                "Попытка вычислить выражение провалилась, так как обнаружена ошибка синтаксиса {}",
                                compile_error
                            );
                            None
                        },
                        CompiledExpression::None => {
                            error!(
                                target: EFFECT_GRAPH_TARGET,
                                "Попытка вычислить выражение провалилась, так как выражение пустое"
                            );
                            None
                        }
                    }
                }
            }
        }
        None => {
            error!(
                target: EFFECT_GRAPH_TARGET,
                "Попытка вычислить выражение провалилась, так как обнаружен некорректный идентификатор черты {:?}",
                parameter_config_id.uuid
            );
            None
        },
    }
}

pub fn get_entity_tag_count(
    game_world: &GameWorld,
    entity_id: EntityId,
    tag_config_id: ConfigId<TagConfig>,
) -> Option<f32> {
    get_value_holder(game_world, entity_id)
        .map(|value_holder| {
            value_holder.tag_buckets
                .iter()
                .find(|it| it.0 == tag_config_id.uuid)
                .map_or(0f32, |it| it.1)
        })
}

pub fn add_entity_tag_count(
    game_config_provider: &ConfigProvider,
    game_world: &mut GameWorld,
    entity_id: EntityId,
    tag_config_id: ConfigId<TagConfig>,
    value: f32,
    delayed_effect_queue: &mut EffectQueue
) -> bool {
    if let Err(_) = game_world.entity(entity_id) {
        return false;
    }

    if let Some(entity_tag_count) = get_entity_tag_count(game_world, entity_id, tag_config_id)
        && entity_tag_count < 1f32
        && (entity_tag_count + value) >= 1f32
    {
        if let Some(tag_config) = game_config_provider.get_config(tag_config_id) {
            if let Some(effect_config_id) = tag_config.effect_mechanic {
                // Считаем, что значения для кастера статусных эффектов берутся из глобального хранилища значений
                if let Some(global_values_entity_id) = GlobalValuesHolder::get_entity_id(game_world) {
                    delayed_effect_queue.push(
                        effect_config_id,
                        EffectContext {
                            current_node_id: None,
                            caster_id: global_values_entity_id,
                            target_id: entity_id
                        }
                    )
                }
            }
        }
    }
    get_value_holder_mut(game_world, entity_id)
        .map(|mut value_holder| {
            let value_before = value_holder.tag_buckets
                .iter()
                .find(|it| it.0 == tag_config_id.uuid)
                .map_or(0f32, |it| it.1);
            let new_value = value_before + value;
            if value_holder.tag_buckets
                .iter_mut()
                .find(|it| it.0 == tag_config_id.uuid).map(|it| it.1 = new_value)
                .is_none() {
                value_holder.tag_buckets.push((tag_config_id.uuid, value));
            }
            if new_value <= 0f32 {
                value_holder.tag_buckets.retain(|it| it.0 != tag_config_id.uuid);
            }
        });
    true
}

pub fn get_node_hash<N: EffectNodeImpl, H: Hash>(node: &N, salt_hash: H) -> u128 {
    let mut salt_hasher = DefaultHasher::new();
    salt_hash.hash(&mut salt_hasher);
    let salt_hash = salt_hasher.finish() as u128;

    let pos = node.get_node_pos();
    let result_hash = (node.get_node_id().0 as u128) & 0xFFFF_FFFF;
    let result_hash = (result_hash << 16) | ((pos.x as u128) & 0xFFFF);
    let result_hash = (result_hash << 16) | ((pos.y as u128) & 0xFFFF);
    let result_hash = (result_hash << 64) | salt_hash;
    result_hash
}

pub trait EffectNodeImpl {
    fn get_node_id(&self) -> EffectNodeId;
    fn get_node_pos(&self) -> egui::Pos2;
    fn tick(
        &self,
        game_config_provider: &ConfigProvider,
        game_world: &mut GameWorld,
        effect_id: EntityId,
        effect_queue: &mut EffectQueue
    ) -> EffectControlFlow;
}

macro_rules! enum_dispatched {
    (pub enum $name:ident { $($variant:ident),* }) => {
        #[derive(Copy, Clone, Debug, Serialize, Deserialize)]
        pub enum $name {
            $($variant($variant)),*
        }
        impl EffectNodeImpl for $name {
            fn get_node_id(&self) -> EffectNodeId {
                match self {
                    $(Self::$variant(x) => x.get_node_id(),)*
                }
            }
            fn get_node_pos(&self) -> egui::Pos2 {
                match self {
                    $(Self::$variant(x) => x.get_node_pos(),)*
                }
            }
            fn tick(
                &self,
                game_config_provider: &ConfigProvider,
                game_world: &mut GameWorld,
                effect_id: EntityId,
                effect_queue: &mut EffectQueue
            ) -> EffectControlFlow {
                match self {
                    $(Self::$variant(x) => x.tick(game_config_provider, game_world, effect_id, effect_queue),)*
                }
            }
        }
    };
}

enum_dispatched!(
    pub enum EffectNode {
        AddTagNode,
        BranchNode,
        SpawnSubEffectNode,
        WaitForConditionNode,
        WaitTicksNode
    }
);


#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Hash)]
pub struct EffectNodeId(u32);
impl EffectNodeId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EffectRoot {
    nodes: Vec<EffectNode>,
    setup: Option<EffectNodeId>,
    tick: Option<EffectNodeId>,
    on_destroy: Option<EffectNodeId>,
}

impl EffectRoot {
    pub fn new(
        nodes: Vec<EffectNode>,
        setup: Option<EffectNodeId>,
        tick: Option<EffectNodeId>,
        on_destroy: Option<EffectNodeId>,
    ) -> Self {
        Self { nodes, setup, tick, on_destroy }
    }

    pub fn setup(
        &self,
        game_config_provider: &ConfigProvider,
        game_world: &mut GameWorld,
        effect_id: EntityId,
        effect_queue: &mut EffectQueue
    ) {
        if effect_context_is_expired(game_world, effect_id) {
            info!(
                target: EFFECT_GRAPH_TARGET,
                "Актуальность контекста эффекта {:?} утеряна, установка проигнорирована",
                effect_id
            );
            return;
        }

        let result = self.setup
            .map(|id| self.nodes[id.0 as usize].tick(
                game_config_provider,
                game_world,
                effect_id,
                effect_queue
            )).unwrap_or(EffectControlFlow::Complete);

        match result {
            EffectControlFlow::Complete => {}
            _ => warn!(
                target: EFFECT_GRAPH_TARGET,
                "В слот установки подключен узел, ожидающийся для работы в несколько тиков. Подобное поведение \
                не поддерживается и скорее всего ожидаемый результат будет некорректным. Для достижения вызова \
                продолжающегося поведения во время установки, нужно использовать узел, порождающий дочерний эффект."
            )
        }
    }

    pub fn tick(
        &self,
        game_config_provider: &ConfigProvider,
        game_world: &mut GameWorld,
        effect_id: EntityId,
        mut current_node_id: Option<EffectNodeId>,
        effect_queue: &mut EffectQueue
    ) -> EffectControlFlow {
        if effect_context_is_expired(game_world, effect_id) {
            info!(
                target: EFFECT_GRAPH_TARGET,
                "Цепочка эффекта {:?} прервана ввиду утери актуальности контекста",
                effect_id
            );
            return EffectControlFlow::Complete;
        }

        if current_node_id.is_none() {
            current_node_id = self.tick;
        }

        current_node_id
            .map(|id| self.nodes[id.0 as usize].tick(
                game_config_provider,
                game_world,
                effect_id,
                effect_queue
            ))
            .unwrap_or(EffectControlFlow::Complete)
    }

    pub fn on_destroy(
        &self,
        game_config_provider: &ConfigProvider,
        game_world: &mut GameWorld,
        effect_id: EntityId,
        effect_queue: &mut EffectQueue
    ) {
        if effect_context_is_expired(game_world, effect_id) {
            info!(
                target: EFFECT_GRAPH_TARGET,
                "Актуальность контекста эффекта {:?} утеряна, очистка проигнорирована",
                effect_id
            );
            return;
        }

        let result = self.on_destroy.as_ref()
            .map(|id| self.nodes[id.0 as usize].tick(
                game_config_provider,
                game_world,
                effect_id,
                effect_queue
            )).unwrap_or(EffectControlFlow::Complete);

        match result {
            EffectControlFlow::Complete => {}
            _ => warn!(
                target: EFFECT_GRAPH_TARGET,
                "В слот очистки подключен узел, ожидающийся для работы в несколько тиков. Подобное поведение \
                не поддерживается и скорее всего ожидаемый результат будет некорректным. Для достижения вызова \
                продолжающегося поведения во время очистки, нужно использовать узел, порождающий дочерний эффект."
            )
        }
    }
}