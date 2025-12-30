use bevy::{prelude::*, time::Stopwatch};

use crate::{
    backend::{
        game_parameters::SetUpBoard,
        tiles::{LogTileDeleted, LogicalTileCreated, LogicalTileLocation},
    },
    frontend::FrontEndSystems,
};

pub struct VisTilesPlugin;

impl Plugin for VisTilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, create_vis_tiles_if_needed);
        app.add_systems(SetUpBoard, initialize_handles);

        app.add_systems(
            Update,
            (delete_vis_tiles, scale_new_spawns).in_set(FrontEndSystems),
        );
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct VisTileOf(pub Entity);

#[derive(Debug, Clone, Component)]
struct SpawnningAnimationTimeData {
    stopwatch: Stopwatch,
    pretend_start_time: f32,
}

fn create_vis_tiles_if_needed(
    mut reader: MessageReader<LogicalTileCreated>,
    mut commands: Commands,
    log_tiles: Query<&LogicalTileLocation>,
    basic_hex: Res<BasicHexHandle>,
) {
    for (animation_time_offset, log_tile) in reader.read().enumerate() {
        let true_pos = log_tiles
            .get(log_tile.0)
            .expect("The tile had no location")
            .read();
        let translation = Vec3::new(true_pos.x, 0.0, true_pos.y);

        commands.spawn((
            Transform::default().with_translation(translation),
            VisTileOf(log_tile.0),
            SceneRoot(basic_hex.0.clone()),
            Pickable {
                is_hoverable: true,
                should_block_lower: true,
            },
            SpawnningAnimationTimeData {
                stopwatch: Stopwatch::new(),
                pretend_start_time: animation_time_offset as f32 / 30.0,
            },
        ));
    }
}

fn scale_new_spawns(
    spawns: Query<(Entity, &mut SpawnningAnimationTimeData, &mut Transform)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    const SCALE_SPEED: f32 = 3.0;
    const SCALE_MULTIPLIER: f32 = 1.2; // must be greater than 1
    let time_finished: f32 = (std::f32::consts::PI - (1.0 / SCALE_MULTIPLIER).asin()) / SCALE_SPEED;

    for (vis_tile_ent, mut animation_timer_data, mut transform) in spawns {
        animation_timer_data.stopwatch.tick(time.delta());

        let mut mapped_time =
            animation_timer_data.stopwatch.elapsed_secs() - animation_timer_data.pretend_start_time;

        if mapped_time < 0.0 {
            mapped_time = 0.0
        }

        if mapped_time >= time_finished {
            transform.scale = Vec3::splat(1.0);
            commands
                .entity(vis_tile_ent)
                .remove::<SpawnningAnimationTimeData>();
        } else {
            transform.scale = Vec3::splat(SCALE_MULTIPLIER * (mapped_time * SCALE_SPEED).sin());
        }
    }
}

#[derive(Resource, Debug)]
struct BasicHexHandle(Handle<Scene>);

fn initialize_handles(mut commands: Commands, assets: ResMut<AssetServer>) {
    commands.insert_resource(BasicHexHandle(
        assets.load(GltfAssetLabel::Scene(0).from_asset("BasicTile.glb")),
    ));
}

fn delete_vis_tiles(
    mut reader: MessageReader<LogTileDeleted>,
    mut commands: Commands,
    vis_tiles: Query<(Entity, &VisTileOf)>,
) {
    for deleted_ent in reader.read() {
        for (vis_ent, vis_of_ent) in vis_tiles {
            if vis_of_ent.0 == deleted_ent.0 {
                commands.entity(vis_ent).despawn();
            }
        }
    }
}
