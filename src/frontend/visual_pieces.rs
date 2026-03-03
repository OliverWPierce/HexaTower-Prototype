use bevy::{math::ops::sin, prelude::*, time::Stopwatch};

use crate::{
    backend::{
        game_actions::CurrentAction,
        game_parameters::SetUpBoard,
        pieces::{
            ActiveLogPiece, LogPieceDespawned, LogPieceOwnedByPlayer, OccupiesTile, Piece,
            SetPieceToActive, SpawnedLogPieceInfo,
        },
        tiles::LogicalTileLocation,
    },
    frontend::{
        FrontEndSystems,
        inputs::ClickCounter,
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

        app.add_systems(SetUpBoard, init_active_ring_model);
        app.add_observer(set_active_request);

        app.add_systems(SetUpBoard, initialize_vis_error_model);

        app.add_systems(Update, tmp_animate_indicator);

        app.add_systems(
            Update,
            tmp_spawn_active_indicator
                .run_if(resource_changed::<ActiveLogPiece>)
                .in_set(FrontEndSystems),
        );
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
#[derive(Resource)]
struct ActivePieceRingModel(Handle<Scene>);

fn init_active_ring_model(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.insert_resource(ActivePieceRingModel(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("ActivePieceRing.glb")),
    ));
}

fn set_active_request(
    click: On<Pointer<Click>>,
    vis_pieces: Query<&VisPieceOf>,
    mut commands: Commands,
    loaded_action: Res<CurrentAction>,
    mut meaningful_clicks: ResMut<ClickCounter>,
) {
    if loaded_action.0.is_some() {
        return;
    }

    let Ok(log_piece) = vis_pieces.get(click.entity) else {
        return;
    };

    meaningful_clicks.0 += 1;
    commands.trigger(SetPieceToActive(Some(log_piece.0)));
}

#[derive(Component, Debug)]
struct ActiveRingsAnimData {
    offset: f32,
    t: Stopwatch,
    should_exist: bool,
}

fn tmp_spawn_active_indicator(
    log_active_piece: Res<ActiveLogPiece>,
    mut commands: Commands,
    vis_pieces: Query<(Entity, &VisPieceOf)>,
    indicators: Query<&mut ActiveRingsAnimData>,
    model: Res<ActivePieceRingModel>,
) {
    for mut anim in indicators {
        anim.should_exist = false;
    }

    let Some(active_piece) = log_active_piece.0 else {
        return;
    };

    for (visual, log_piece_represented) in vis_pieces {
        if log_piece_represented.0 != active_piece {
            continue;
        }

        commands.spawn((
            ActiveRingsAnimData {
                offset: 0.0,
                t: Stopwatch::new(),
                should_exist: true,
            },
            ChildOf(visual),
            SceneRoot(model.0.clone()),
            Transform::from_scale(Vec3::ZERO),
        ));

        commands.spawn((
            ActiveRingsAnimData {
                offset: 0.3,
                t: Stopwatch::new(),
                should_exist: true,
            },
            ChildOf(visual),
            SceneRoot(model.0.clone()),
            Transform::from_scale(Vec3::ZERO),
        ));

        break;
    }
}

fn tmp_animate_indicator(
    indicators: Query<(Entity, &mut Transform, &mut ActiveRingsAnimData)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    const SCALE_IN_SPEED: f32 = 3.0;
    const SCALE_OUT_SPEED: f32 = -5.0;

    const HOVER_HEIGHT: f32 = 0.5;
    const HOVER_SPEED: f32 = 6.0;
    const HOVER_MAGNITUDE: f32 = 0.2;

    for (entity, mut transform, mut anim_data) in indicators {
        anim_data.t.tick(time.delta());

        if anim_data.should_exist {
            transform.scale = Vec3::splat(
                ((anim_data.t.elapsed_secs() + anim_data.offset) * SCALE_IN_SPEED).clamp(0.0, 1.0),
            );
        } else {
            let scale = (anim_data.t.elapsed_secs() + anim_data.offset) * SCALE_OUT_SPEED;
            if scale <= 0.001 {
                commands.entity(entity).despawn();
            } else {
                transform.scale = Vec3::splat(scale);
            }
        }

        transform.translation.y = sin(anim_data.t.elapsed_secs() + anim_data.offset * HOVER_SPEED)
            * HOVER_MAGNITUDE
            + HOVER_HEIGHT;
    }
}
