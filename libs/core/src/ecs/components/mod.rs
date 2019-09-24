use serde_derive::{Deserialize, Serialize};

pub mod damage_history;
pub mod missile;

use amethyst::{
    ecs::{Component, DenseVecStorage, Entity, FlaggedStorage, NullStorage, ReaderId, VecStorage},
    network::NetEvent,
};
use shrinkwraprs::Shrinkwrap;

use std::time::{Duration, Instant};

use crate::{
    actions::{
        mob::MobAction,
        player::{PlayerCastAction, PlayerLookAction, PlayerWalkAction},
        Action,
    },
    math::{Vector2, ZeroVector},
    net::{EncodedMessage, NetIdentifier},
};

#[derive(Clone, Debug, Serialize, Deserialize, Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct WorldPosition {
    #[shrinkwrap(main_field)]
    pub position: Vector2,
}

impl WorldPosition {
    pub fn new(position: Vector2) -> Self {
        Self { position }
    }
}

impl Component for WorldPosition {
    type Storage = VecStorage<Self>;
}

#[derive(Clone, Debug)]
pub struct Player {
    pub health: f32,
    pub velocity: Vector2,
    pub walking_direction: Vector2,
    pub looking_direction: Vector2,
    pub radius: f32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            health: 100.0,
            velocity: Vector2::zero(),
            walking_direction: Vector2::new(0.0, 1.0),
            looking_direction: Vector2::new(0.0, 1.0),
            radius: 20.0,
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Player {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PlayerActions {
    pub walk_action: PlayerWalkAction,
    pub look_action: PlayerLookAction,
    pub cast_action: Option<PlayerCastAction>,
}

impl Component for PlayerActions {
    type Storage = DenseVecStorage<Self>;
}

/// We write the actions to this component right on input from client, they get processed and
/// inserted to PlayerActions component (and optionally scheduled to be sent to a server)
/// in ActionSystem.
#[derive(Default, Debug, Clone)]
pub struct ClientPlayerActions {
    pub walk_action: PlayerWalkAction,
    pub look_action: PlayerLookAction,
    pub cast_action: Option<PlayerCastAction>,
}

impl Component for ClientPlayerActions {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct PlayerLastCastedSpells {
    pub missile: Duration,
}

impl Component for PlayerLastCastedSpells {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Clone, Debug)]
pub struct Monster {
    pub health: f32,
    pub attack_damage: f32,
    pub destination: Vector2,
    pub velocity: Vector2,
    pub action: Action<MobAction<Entity>>,
    pub name: String,
    pub radius: f32,
}

impl Component for Monster {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Clone, Default)]
pub struct Dead;

impl Component for Dead {
    type Storage = FlaggedStorage<Self, NullStorage<Self>>;
}

pub struct NetConnectionModel {
    pub id: NetIdentifier,
    pub reader: ReaderId<NetEvent<EncodedMessage>>,
    pub created_at: Instant,
    pub last_pinged_at: Instant,
    pub last_acknowledged_update: Option<u64>,
}

impl NetConnectionModel {
    pub fn new(id: NetIdentifier, reader: ReaderId<NetEvent<EncodedMessage>>) -> Self {
        Self {
            id,
            reader,
            created_at: Instant::now(),
            last_pinged_at: Instant::now(),
            last_acknowledged_update: None,
        }
    }
}

impl Component for NetConnectionModel {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Clone, Copy)]
pub struct EntityNetMetadata {
    pub id: NetIdentifier,
    pub spawned_frame_number: u64,
}

impl Component for EntityNetMetadata {
    type Storage = VecStorage<Self>;
}