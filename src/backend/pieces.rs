use bevy::prelude::*;

use crate::backend::BackEndUpdateSystems;

pub struct PiecesPlugin;

impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnLogPiece>();
        app.add_message::<LogPieceSpawned>();

        app.add_systems(Update, spawn_logpiece.in_set(BackEndUpdateSystems));
    }
}
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Component)]
pub enum BasePieceType {
    Tower,
}

impl BasePieceType {
    fn bundle(&self) -> impl Bundle {
        match self {
            BasePieceType::Tower => (Name::new("TowerPiece"), *self),
        }
    }
}

#[derive(Debug, Message, Clone, Copy, PartialEq, PartialOrd)]
pub struct SpawnLogPiece {
    pub piece_type: BasePieceType,
    pub log_tile: Entity,
}

#[derive(Debug, Message, Clone, Copy, PartialEq, PartialOrd)]
pub struct LogPieceSpawned(pub Entity);

#[derive(Clone, Component, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[relationship(relationship_target = OccupiedByPiece)]
pub struct OccupiesTile {
    pub log_tile: Entity,
}

#[derive(Clone, Component, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[relationship_target(relationship = OccupiesTile)]
pub struct OccupiedByPiece {
    log_piece: Entity,
}

fn spawn_logpiece(
    mut spawn_requests: MessageReader<SpawnLogPiece>,
    mut commands: Commands,
    occupied_tiles: Query<Entity, With<OccupiedByPiece>>,
    mut notify_of_spawns: MessageWriter<LogPieceSpawned>,
) {
    for SpawnLogPiece {
        piece_type,
        log_tile,
    } in spawn_requests.read()
    {
        if occupied_tiles.contains(*log_tile) {
            warn!(
                "A request was sent to spawn a piece on a tile that was already occupied. The request was not fulfilled."
            )
        } else {
            let logpiece_ent = commands
                .spawn((
                    piece_type.bundle(),
                    OccupiesTile {
                        log_tile: *log_tile,
                    },
                ))
                .id();

            notify_of_spawns.write(LogPieceSpawned(logpiece_ent));
        }
    }
}
