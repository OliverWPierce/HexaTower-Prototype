use bevy::prelude::*;

use bevy::{
    asset::{AssetLoader, LoadedFolder},
    ecs::schedule::ScheduleLabel,
    prelude::*,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::backend::BackEndSystems;

pub struct PiecesPlugin;

impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnLogPiece>();
        app.add_message::<SpawnedLogPieceInfo>();
        app.add_message::<LogPieceDespawned>();

        app.add_systems(
            Update,
            (spawn_logpiece, send_despawn_notifications).in_set(BackEndSystems),
        );
    }
}

#[derive(Debug, Serialize, Reflect, Deserialize)]
struct ProxyPiece {
    name: String,
    model_path: String,
    health: u32,
}

#[derive(Debug, Asset, Clone, TypePath)]
pub struct Piece {
    name: String,
    model: Handle<Scene>,
    health: u32,
}

#[derive(Debug, Default, TypePath)]
pub struct PieceAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum PieceAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for PieceAssetLoader {
    type Asset = Piece;
    type Settings = ();
    type Error = PieceAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let proxy = ron::de::from_bytes::<ProxyPiece>(&bytes)?;

        let piece = Piece {
            name: proxy.name.clone(),
            model: load_context.load(proxy.model_path),
            health: proxy.health,
        };

        Ok(piece)
    }

    fn extensions(&self) -> &[&str] {
        &["piece.ron"]
    }
}

#[derive(Debug, Message, Clone, PartialEq, PartialOrd)]
pub struct SpawnLogPiece {
    pub piece: Handle<Piece>,
    pub log_tile: Entity,
}

#[derive(Debug, Message, Clone, PartialEq, PartialOrd)]
pub struct SpawnedLogPieceInfo {
    enity: Entity,
    model: Handle<Scene>,
}

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

#[derive(Debug, Component)]
pub struct Health {
    max_health: u32,
    current_health: u32,
}

fn spawn_logpiece(
    mut spawn_requests: MessageReader<SpawnLogPiece>,
    mut commands: Commands,
    occupied_tiles: Query<Entity, With<OccupiedByPiece>>,
    mut notify_of_spawns: MessageWriter<SpawnedLogPieceInfo>,
    pieces: Res<Assets<Piece>>,
) {
    for SpawnLogPiece { piece, log_tile } in spawn_requests.read() {
        if occupied_tiles.contains(*log_tile) {
            warn!(
                "A request was sent to spawn a piece on a tile that was already occupied. The request was not fulfilled."
            )
        } else {
            let Some(piece_instructions) = pieces.get(piece) else {
                error!("a piece asset was not yet fully loaded.");
                continue;
            };

            let logpiece_ent = commands
                .spawn((
                    OccupiesTile {
                        log_tile: *log_tile,
                    },
                    Health {
                        max_health: piece_instructions.health,
                        current_health: piece_instructions.health,
                    },
                ))
                .id();

            notify_of_spawns.write(SpawnedLogPieceInfo {
                enity: logpiece_ent,
                model: piece_instructions.model.clone(),
            });
        }
    }
}

#[derive(Debug, Message)]
pub struct LogPieceDespawned(pub Entity);

fn send_despawn_notifications(
    mut despawned_logical_pieces: RemovedComponents<OccupiesTile>,
    mut despawns: MessageWriter<LogPieceDespawned>,
) {
    for logical_piece in despawned_logical_pieces.read() {
        despawns.write(LogPieceDespawned(logical_piece));
    }
}
