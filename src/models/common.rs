use amethyst::{
    assets::{Handle, Prefab},
    ecs::Entity,
    renderer::SpriteSheet,
    ui::FontHandle,
};

use animation_prefabs::GameSpriteAnimationPrefab;

use crate::{data_resources::EntityGraphics, Vector2};

pub enum GameState {
    Loading,
    Playing,
}

#[derive(Clone)]
pub struct MonsterDefinition {
    pub name: String,
    pub base_health: f32,
    pub base_speed: f32,
    pub base_attack: f32,
    pub graphics: EntityGraphics,
    pub radius: f32,
}

#[derive(Clone)]
pub struct AssetsHandles {
    pub hero_prefab: Handle<Prefab<GameSpriteAnimationPrefab>>,
    pub landscape: Handle<SpriteSheet>,
    pub ui_font: FontHandle,
}

#[derive(Clone)]
pub enum MissileTarget {
    Target(Entity),
    Destination(Vector2),
}
