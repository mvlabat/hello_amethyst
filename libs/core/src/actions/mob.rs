use amethyst::ecs::{storage::GenericReadStorage, Entity};
use rand::Rng;
use serde_derive::{Deserialize, Serialize};

use crate::{
    ecs::{components::EntityNetMetadata, resources::net::EntityNetMetadataStorage},
    math::Vector2,
    net::NetIdentifier,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MobAction<T> {
    /// Just spawned.
    Idle,
    /// Moving to the specified position.
    Move(Vector2),
    /// Chasing an entity with the specified id.
    Chase(T),
    /// Attacking a target.
    Attack(MobAttackAction<T>),
}

impl<T> Default for MobAction<T> {
    fn default() -> Self {
        Self::Idle
    }
}

impl MobAction<Entity> {
    pub fn load_entity_net_id(
        &self,
        entity_net_metadata: &impl GenericReadStorage<Component = EntityNetMetadata>,
    ) -> MobAction<NetIdentifier> {
        match self {
            MobAction::Idle => MobAction::Idle,
            MobAction::Move(destination) => MobAction::Move(*destination),
            MobAction::Chase(target) => {
                MobAction::Chase(entity_net_metadata.get(*target).unwrap().id)
            }
            MobAction::Attack(MobAttackAction {
                target,
                attack_type,
            }) => MobAction::Attack(MobAttackAction {
                target: entity_net_metadata.get(*target).unwrap().id,
                attack_type: attack_type.clone(),
            }),
        }
    }
}

impl MobAction<NetIdentifier> {
    pub fn load_entity_by_net_id(
        self,
        entity_net_metadata_service: &EntityNetMetadataStorage,
    ) -> MobAction<Entity> {
        match self {
            MobAction::Idle => MobAction::Idle,
            MobAction::Move(destination) => MobAction::Move(destination),
            MobAction::Chase(target) => {
                MobAction::Chase(entity_net_metadata_service.get_entity(target).unwrap())
            }
            MobAction::Attack(MobAttackAction {
                target,
                attack_type,
            }) => MobAction::Attack(MobAttackAction {
                target: entity_net_metadata_service.get_entity(target).unwrap(),
                attack_type,
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobAttackAction<T> {
    /// Entity id.
    pub target: T,
    pub attack_type: MobAttackType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MobAttackType {
    #[allow(dead_code)]
    Melee,
    SlowMelee {
        cooldown: f32,
    },
    #[allow(dead_code)]
    Range,
}

impl MobAttackType {
    pub fn randomize_params(&self, factor: f32) -> Self {
        let mut rng = rand::thread_rng();
        match self {
            MobAttackType::SlowMelee { cooldown } => {
                let cooldown = rng.gen_range(cooldown * (1.0 - factor), cooldown * (1.0 + factor));
                MobAttackType::SlowMelee { cooldown }
            }
            other => other.clone(),
        }
    }
}
