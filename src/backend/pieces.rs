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
        app.add_message::<LogPieceSpawned>();
        app.add_message::<LogPieceDespawned>();

        app.add_systems(
            Update,
            (spawn_logpiece, send_despawn_notifications).in_set(BackEndSystems),
        );
    }
}

#[derive(Debug, Default, TypePath)]
pub struct PieceAssetLoader;

#[derive(Serialize, Debug, Deserialize, Reflect, Asset, Clone)]
pub struct PieceAsset {
    name: String,
    model_path: String,
}

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
    type Asset = PieceAsset;
    type Settings = ();
    type Error = PieceAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let card = ron::de::from_bytes::<PieceAsset>(&bytes)?;
        Ok(card)
    }

    fn extensions(&self) -> &[&str] {
        &["piece.ron"]
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
