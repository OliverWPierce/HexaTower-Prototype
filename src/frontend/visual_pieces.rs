use bevy::{prelude::*, time::Stopwatch};

use crate::{
    backend::{
        game_parameters::SetUpBoard,
        pieces::{BasePieceType, LogPieceDespawned, LogPieceSpawned, OccupiesTile},
        tiles::LogicalTileLocation,
    },
    frontend::FrontEndUpdateSystems,
};

pub struct VisPiecesPlugin;

impl Plugin for VisPiecesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, initialize_piece_model_handles);
        app.add_systems(
            Update,
            (
                spawn_peice_visuals,
                scale_visuals,
                start_scale_out_for_destroyed_pieces,
            )
                .in_set(FrontEndUpdateSystems),
        );
    }
}

#[derive(Resource, Clone)]
struct PieceModelHandles {
    tower: Handle<Scene>,
}
#[derive(Debug, Component)]
pub struct VisPieceOf(pub Entity);

fn initialize_piece_model_handles(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.insert_resource(PieceModelHandles {
        tower: asset_server.load(GltfAssetLabel::Scene(0).from_asset("RedPeiceBasePlate.glb")),
    });
}

#[derive(Debug, Component)]
struct AnimationStopwatch {
    elapsed: Stopwatch,
    scale_in: bool,
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
                scale: Vec3::ZERO,
                ..Default::default()
            },
            VisPieceOf(*log_piece),
            AnimationStopwatch {
                elapsed: Stopwatch::new(),
                scale_in: true,
            },
        ));
    }
}

fn scale_visuals(
    time: Res<Time>,
    mut vis_pieces_to_scale: Query<(Entity, &mut Transform, &mut AnimationStopwatch)>,
    mut commands: Commands,
) {
    const SPEED: f32 = 7.0;

    for (piece_ent, mut transform, mut watch) in vis_pieces_to_scale.iter_mut() {
        watch.elapsed.tick(time.delta());

        if watch.scale_in {
            //do this if spawning
            let new_scale = SPEED * watch.elapsed.elapsed_secs() * watch.elapsed.elapsed_secs();

            if new_scale >= 1.0 {
                transform.scale = Vec3::splat(1.0);
                commands.entity(piece_ent).remove::<AnimationStopwatch>();
            } else {
                transform.scale = Vec3::splat(new_scale);
            }
        } else {
            //do this if despawning
            let new_scale =
                1.0 - SPEED * watch.elapsed.elapsed_secs() * watch.elapsed.elapsed_secs();

            if new_scale <= 0.0 {
                transform.scale = Vec3::splat(0.0);
                commands.entity(piece_ent).despawn();
            } else {
                transform.scale = Vec3::splat(new_scale);
            }
        }
    }
}

fn start_scale_out_for_destroyed_pieces(
    mut despawned: MessageReader<LogPieceDespawned>,
    vis_pieces: Query<(Entity, &VisPieceOf)>,
    mut commands: Commands,
) {
    for LogPieceDespawned(despawned_log_piece) in despawned.read() {
        for (vis_ent, VisPieceOf(log_piece)) in vis_pieces {
            if log_piece == despawned_log_piece {
                commands.entity(vis_ent).insert(AnimationStopwatch {
                    elapsed: Stopwatch::new(),
                    scale_in: false,
                });
                break;
            }
        }
    }
}
