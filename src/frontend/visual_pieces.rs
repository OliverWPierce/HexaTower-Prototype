use bevy::{prelude::*, time::Stopwatch};

use crate::{
    backend::{
        pieces::{LogPieceDespawned, OccupiesTile, SpawnedLogPieceInfo},
        players::ActivePlayer,
        tiles::LogicalTileLocation,
    },
    frontend::{
        FrontEndSystems,
        visual_player_data::{DataForPlayer, PieceBasePlateModel},
    },
};

pub struct VisPiecesPlugin;

impl Plugin for VisPiecesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_peice_visuals,
                scale_visuals,
                start_scale_out_for_destroyed_pieces,
            )
                .in_set(FrontEndSystems),
        );
    }
}

#[derive(Debug, Component)]
pub struct VisPieceOf(pub Entity);

#[derive(Debug, Component)]
struct AnimationStopwatch {
    elapsed: Stopwatch,
    scale_in: bool,
}

const PIECE_BASEPLATE_THICKNESS: f32 = 0.113;

fn spawn_peice_visuals(
    mut new_log_spawns: MessageReader<SpawnedLogPieceInfo>,
    log_pieces: Query<&OccupiesTile>,
    log_tile_location: Query<&LogicalTileLocation>,
    active_player: Res<ActivePlayer>,
    base_plates: Query<(&DataForPlayer, &PieceBasePlateModel)>,
    asset_server: ResMut<AssetServer>,
    mut commands: Commands,
) {
    for SpawnedLogPieceInfo {
        log_piece_entity,
        model,
    } in new_log_spawns.read()
    {
        let (log_occupied) = log_pieces
            .get(*log_piece_entity)
            .expect("A logical piece spawned message did not contain a logical piece");

        let physical_position = log_tile_location
            .get(log_occupied.log_tile)
            .expect("the logical piece did not occupy a logical tile with a logical location.")
            .read();

        let mut base_plate_model: Handle<Scene> =
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("VisualError3d.glb"));

        for (player_represented, plate_model) in base_plates.iter() {
            if active_player.0 != player_represented.0 {
                continue;
            }
            base_plate_model = plate_model.0.clone();
            break;
        }

        commands.spawn((
            SceneRoot(base_plate_model),
            Transform {
                translation: Vec3::new(physical_position.x, 0.0, physical_position.y),
                scale: Vec3::ZERO,
                ..Default::default()
            },
            VisPieceOf(*log_piece_entity),
            AnimationStopwatch {
                elapsed: Stopwatch::new(),
                scale_in: true,
            },
            children![(
                SceneRoot(model.clone()),
                Transform {
                    translation: Vec3::new(0.0, PIECE_BASEPLATE_THICKNESS, 0.0),
                    scale: Vec3::ONE,
                    ..Default::default()
                },
            )],
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
