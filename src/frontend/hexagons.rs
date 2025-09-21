use std::thread::spawn;

use bevy::{ecs::component::TickCells, math::VectorSpace, prelude::*};

use crate::{
    backend::{AppState, TileReadyForVisual, TilesPlugin, TrueTileLocation},
    frontend::VisualOf,
};

pub struct HexagonsPlugin;

impl Plugin for HexagonsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_hex, tick_spawn_timer, upscale_recent_spawns).run_if(in_state(AppState::InGame)),
        );
        app.add_systems(OnEnter(AppState::InGame), spawn_placeholder_cam);
    }
}
#[derive(Debug, Component)]
struct AwaitVisualSpawnTimer(Timer);

#[derive(Debug, Component)]
struct VisualTile;

fn spawn_hex(
    mut reader: EventReader<TileReadyForVisual>,
    mut commands: Commands,
    visual_tiles: Query<&VisualOf>,
    backend_tiles: Query<&TrueTileLocation>,
    asset_server: ResMut<AssetServer>,
) {
    let basic_tile =
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("BasicTile.glb")));
    let tile_ring =
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("TileHighlightRing.glb")));

    for (id, TileReadyForVisual(visual)) in reader.read().enumerate() {
        let tile_location = if let Ok(VisualOf(tile)) = visual_tiles.get(*visual) {
            *backend_tiles
                .get(*tile)
                .expect("The backend tile had no location.")
        } else {
            panic!("The visual tile had no VisualOf component.")
        };

        commands.entity(*visual).insert((
            (AwaitVisualSpawnTimer(Timer::from_seconds((id as f32) / 50.0, TimerMode::Once))),
            Transform {
                translation: tile_location.into(),
                rotation: Quat::default(),
                scale: Vec3::splat(0.0),
            },
            basic_tile.clone(),
            VisualTile,
            children![tile_ring.clone()],
        ));
    }
}

fn tick_spawn_timer(
    spawned: Query<(Entity, &mut AwaitVisualSpawnTimer)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let delta = time.delta();

    for (ent, mut timer) in spawned {
        timer.0.tick(delta);
        if timer.0.just_finished() {
            commands.entity(ent).insert(UpscaleVisualTile);
            commands.entity(ent).remove::<AwaitVisualSpawnTimer>();
        }
    }
}
#[derive(Component, Clone, Copy, Debug)]
struct UpscaleVisualTile;

fn upscale_recent_spawns(
    mut spawns: Query<(Entity, &mut Transform), (With<UpscaleVisualTile>, With<VisualTile>)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    let target = Vec3::splat(1.0);

    for (entity, mut tile) in spawns.iter_mut() {
        tile.scale = tile.scale.move_towards(target, 4.0 * delta);

        if tile.scale == target {
            commands.entity(entity).remove::<UpscaleVisualTile>();
        }
    }
}

fn spawn_placeholder_cam(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::default()
            .with_translation(Vec3::new(0.1, 18.0, 0.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        Camera {
            order: 1,
            ..Default::default()
        },
    ));

    commands.spawn((
        PointLight {
            intensity: 100000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::default().with_translation(Vec3::new(0.0, 10.0, 0.0)),
    ));
}
