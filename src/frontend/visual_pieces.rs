use bevy::prelude::*;

use crate::{
    backend::{
        game_parameters::SetUpBoard,
        pieces::{BasePieceType, LogPieceSpawned, OccupiesTile},
        tiles::LogicalTileLocation,
    },
    frontend::FrontEndUpdateSystems,
};

pub struct VisPiecesPlugin;

impl Plugin for VisPiecesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, initialize_piece_model_handles);
        app.add_systems(Update, spawn_peice_visuals.in_set(FrontEndUpdateSystems));
    }
}

#[derive(Resource, Clone)]
struct PieceModelHandles {
    tower: Handle<Scene>,
}

fn initialize_piece_model_handles(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.insert_resource(PieceModelHandles {
        tower: asset_server.load(GltfAssetLabel::Scene(0).from_asset("RedPeiceBasePlate.glb")),
    });
}

fn spawn_peice_visuals(
    mut new_log_spawns: MessageReader<LogPieceSpawned>,
    mesh_handles: Res<PieceModelHandles>,
    log_pieces: Query<(&BasePieceType, &OccupiesTile)>,
    log_tile_location: Query<&LogicalTileLocation>,
    mut commands: Commands,
) {
    for LogPieceSpawned(log_piece) in new_log_spawns.read() {
        let (piece_type, log_occupied) = log_pieces
            .get(*log_piece)
            .expect("A logical piece spawned message did not contain a logical piece");

        let handle = match piece_type {
            BasePieceType::Tower => mesh_handles.tower.clone(),
        };

        let physical_position = log_tile_location
            .get(log_occupied.log_tile)
            .expect("the logical piece did not occupy a logical tile with a logical location.")
            .read();

        commands.spawn((
            SceneRoot(handle),
            Transform {
                translation: Vec3::new(physical_position.x, 0.0, physical_position.y),
                ..Default::default()
            },
        ));
    }
}
