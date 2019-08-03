use amethyst::{
    assets::Handle,
    core::{Time, Transform},
    ecs::{Entities, ReadExpect, System, WriteExpect, WriteStorage},
    renderer::{Material, Mesh},
};
use num;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use ha_core::math::{Vector2, ZeroVector};

use crate::{
    actions::{
        mob::{MobAction, MobActionType},
        monster_spawn::{SpawnActions, SpawnType},
    },
    ecs::{
        components::{damage_history::DamageHistory, Monster, WorldPosition},
        resources::{
            graphics::EntityGraphics, GameLevelState, MonsterDefinition, MonsterDefinitions,
        },
    },
};

pub struct MonsterSpawnerSystem;

impl<'s> System<'s> for MonsterSpawnerSystem {
    type SystemData = (
        Entities<'s>,
        ReadExpect<'s, Time>,
        ReadExpect<'s, MonsterDefinitions>,
        ReadExpect<'s, GameLevelState>,
        WriteExpect<'s, SpawnActions>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Handle<Mesh>>,
        WriteStorage<'s, Handle<Material>>,
        WriteStorage<'s, Monster>,
        WriteStorage<'s, DamageHistory>,
        WriteStorage<'s, WorldPosition>,
    );

    fn run(
        &mut self,
        (
            entities,
            time,
            monster_definitions,
            game_scene,
            mut spawn_actions,
            mut transforms,
            mut meshes,
            mut materials,
            mut monsters,
            mut damage_histories,
            mut world_positions,
        ): Self::SystemData,
    ) {
        let mut rng = rand::thread_rng();
        let SpawnActions(ref mut spawn_actions) = *spawn_actions;
        for spawn_action in spawn_actions.drain(..) {
            let ghoul = monster_definitions
                .0
                .get("Ghoul")
                .expect("Failed to get Ghoul monster definition");

            let mut spawn_monster =
                |position: Vector2, action: MobAction, monster_definition: &MonsterDefinition| {
                    let mut transform = Transform::default();
                    transform.set_translation_xyz(position.x, position.y, 5.0);
                    let destination = if let MobActionType::Move(destination) = action.action_type {
                        destination
                    } else {
                        Vector2::zero()
                    };

                    let MonsterDefinition {
                        name,
                        base_health: health,
                        base_speed: _base_speed,
                        base_attack_damage: attack_damage,
                        graphics: EntityGraphics { mesh, material },
                        radius,
                        ..
                    } = monster_definition.clone();
                    entities
                        .build_entity()
                        .with(mesh, &mut meshes)
                        .with(material, &mut materials)
                        .with(transform, &mut transforms)
                        .with(WorldPosition::new(position), &mut world_positions)
                        .with(
                            Monster {
                                health,
                                attack_damage,
                                destination,
                                velocity: Vector2::zero(),
                                action,
                                name,
                                radius,
                            },
                            &mut monsters,
                        )
                        .with(DamageHistory::default(), &mut damage_histories)
                        .build();
                };

            match spawn_action.spawn_type {
                SpawnType::Random => {
                    for _ in 0..spawn_action.monsters.num {
                        let (side_start, side_end, _) = spawning_side(rand::random(), &game_scene);
                        let d = side_start - side_end;
                        let random_displacement = Vector2::new(
                            if d.x == 0.0 {
                                0.0
                            } else {
                                rng.gen_range(0.0, d.x.abs()) * d.x.signum()
                            },
                            if d.y == 0.0 {
                                0.0
                            } else {
                                rng.gen_range(0.0, d.y.abs()) * d.y.signum()
                            },
                        );
                        let position = side_start + random_displacement;
                        spawn_monster(position, MobAction::idle(time.absolute_time()), ghoul);
                    }
                }
                SpawnType::Borderline => {
                    let spawn_margin = 50.0;
                    let (side_start, side_end, destination) =
                        spawning_side(rand::random(), &game_scene);
                    let d = (side_start - side_end) / spawn_margin;
                    let monsters_to_spawn = num::Float::max(d.x.abs(), d.y.abs()).round() as u8;
                    let spawn_distance = (side_end - side_start) / f32::from(monsters_to_spawn);

                    let mut position = side_start;
                    for _ in 0..monsters_to_spawn {
                        let action = MobAction {
                            started_at: time.absolute_time(),
                            action_type: MobActionType::Move(position + destination),
                        };
                        spawn_monster(position, action, ghoul);
                        position += spawn_distance;
                    }
                }
            }
        }
    }
}

fn spawning_side(side: Side, game_scene: &GameLevelState) -> (Vector2, Vector2, Vector2) {
    let scene_halfsize = game_scene.dimensions / 2.0;
    let border_distance = 100.0;
    let padding = 25.0;
    match side {
        Side::Top => (
            Vector2::new(
                -scene_halfsize.x + padding,
                scene_halfsize.y + border_distance,
            ),
            Vector2::new(
                scene_halfsize.x - padding,
                scene_halfsize.y + border_distance,
            ),
            Vector2::new(0.0, -game_scene.dimensions.y + border_distance),
        ),
        Side::Right => (
            Vector2::new(
                scene_halfsize.x + border_distance,
                scene_halfsize.y - padding,
            ),
            Vector2::new(
                scene_halfsize.x + border_distance,
                -scene_halfsize.y + padding,
            ),
            Vector2::new(-game_scene.dimensions.x + border_distance, 0.0),
        ),
        Side::Bottom => (
            Vector2::new(
                scene_halfsize.x - padding,
                -scene_halfsize.y - border_distance,
            ),
            Vector2::new(
                -scene_halfsize.x + padding,
                -scene_halfsize.y - border_distance,
            ),
            Vector2::new(0.0, game_scene.dimensions.y - border_distance),
        ),
        Side::Left => (
            Vector2::new(
                -scene_halfsize.x - border_distance,
                -scene_halfsize.y + padding,
            ),
            Vector2::new(
                -scene_halfsize.x - border_distance,
                scene_halfsize.y - padding,
            ),
            Vector2::new(game_scene.dimensions.x - border_distance, 0.0),
        ),
    }
}

enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

impl Distribution<Side> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Side {
        match rng.gen_range(0, 4) {
            0 => Side::Top,
            1 => Side::Right,
            2 => Side::Bottom,
            _ => Side::Left,
        }
    }
}