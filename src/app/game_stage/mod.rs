use hecs::World;

pub type EntityId = hecs::Entity;

pub struct GameWorld{
    ecs_world: World,
}