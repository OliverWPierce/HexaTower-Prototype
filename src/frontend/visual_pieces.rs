use bevy::{prelude::*, time::Stopwatch};

use crate::{
    backend::{
        game_parameters::SetUpBoard,
        pieces::{
            LogPieceDespawned, LogPieceOwnedByPlayer, OccupiesTile, Piece, SpawnedLogPieceInfo,
        },
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

        app.add_systems(SetUpBoard, initialize_vis_error_model);
    }
}
#[derive(Debug, Resource)]
pub struct VisError3DModel(pub Handle<Scene>);

fn initialize_vis_error_model(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.insert_resource(VisError3DModel(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("VisualError3d.glb")),
    ));
}

#[derive(Debug, Component)]
pub struct VisPieceOf(pub Entity);

#[derive(Debug, Component)]
pub struct PieceName(pub String);

#[derive(Debug, Component)]
struct AnimationStopwatch {
    elapsed: Stopwatch,
    scale_in: bool,
}

const PIECE_BASEPLATE_THICKNESS: f32 = 0.113;

fn spawn_peice_visuals(
    mut new_log_spawns: MessageReader<SpawnedLogPieceInfo>,
    log_pieces: Query<(&OccupiesTile, &LogPieceOwnedByPlayer)>,
    log_tile_location: Query<&LogicalTileLocation>,
    base_plates: Query<(&PieceBasePlateModel, &DataForPlayer)>,
    pieces: Res<Assets<Piece>>,
    vis_error_3d: Res<VisError3DModel>,
    mut commands: Commands,
) {
    for SpawnedLogPieceInfo {
        log_piece_entity,
        from_asset: piece,
    } in new_log_spawns.read()
    {
        let (log_occupied, commanding_player) = log_pieces
            .get(*log_piece_entity)
            .expect("A logical piece spawned message did not contain a logical piece");

        let physical_position = log_tile_location
            .get(log_occupied.log_tile)
            .expect("the logical piece did not occupy a logical tile with a logical location.")
            .read();

        let mut base_plate_model = &vis_error_3d.0;

        for (base_plate, player) in base_plates {
            if player.0 != commanding_player.0 {
                continue;
            } else {
                base_plate_model = &base_plate.0
            }
        }

        let Some(piece_instructions) = pieces.get(piece) else {
            error!("a piece asset was not yet fully loaded.");
            continue;
        };

        commands.spawn((
            SceneRoot(base_plate_model.clone()),
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
            PieceName(piece_instructions.name.clone()),
            children![(
                SceneRoot(piece_instructions.model.clone()),
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
