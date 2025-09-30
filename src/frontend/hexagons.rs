use std::thread::spawn;

use bevy::{ecs::component::TickCells, math::VectorSpace, prelude::*, time::Stopwatch};

use crate::backend::{
    ActiveTile, AppState, TileReadyForVisual, TilesPlugin, TrueTileLocation, ValidMove, VisualOf,
};

pub struct HexagonsPlugin;

impl Plugin for HexagonsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_hex,
                tick_spawn_timer,
                upscale_recent_spawns,
                raise_active,
                raise_valid_moves,
            )
                .run_if(in_state(AppState::InGame)),
        );
        app.add_systems(OnEnter(AppState::InGame), spawn_placeholder_cam);
        app.add_systems(Update, add_stopwatch.run_if(resource_added::<ActiveTile>));
        app.add_systems(
            Update,
            reset_stopwatch
                .run_if(in_state(AppState::InGame))
                .run_if(resource_exists_and_changed::<ActiveTile>),
        );
    }
}
#[derive(Debug, Component)]
struct AwaitVisualSpawnTimer(Timer);

#[derive(Debug, Component)]
struct VisualTile;

#[derive(Debug, Component)]
struct TileRing;

fn spawn_hex(
    mut reader: EventReader<TileReadyForVisual>,
    mut commands: Commands,
    visual_tiles: Query<&VisualOf>,
    backend_tiles: Query<&TrueTileLocation>,
    asset_server: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let basic_tile =
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("BasicTile.glb")));
    let tile_ring =
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("TileHighlightRing.glb")));

    const SQRT3: f32 = 1.7320508;

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
            MeshMaterial3d(materials.add(Color::Srgba(Srgba {
                red: if true {
                    tile_location
                        .read()
                        .normalize()
                        .dot(vec2(1.5, 0.5 * SQRT3).normalize())
                } else {
                    0.0
                },
                green: if true {
                    tile_location
                        .read()
                        .normalize()
                        .dot(vec2(0.0, SQRT3).normalize())
                } else {
                    0.0
                },
                blue: if true {
                    tile_location
                        .read()
                        .normalize()
                        .dot(vec2(-1.5, 0.5 * SQRT3).normalize())
                } else {
                    0.0
                },
                alpha: 1.0,
            }))),
            basic_tile.clone(),
            Mesh3d(meshes.add((Cuboid::new(1.0, 0.2, 1.0)))),
            VisualTile,
            children![(tile_ring.clone(), Transform::default(), TileRing)],
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
            .with_translation(Vec3::new(10.0, 18.0, 0.0))
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
#[derive(Resource, Debug)]
struct SelectedHexagonStopwatch(Stopwatch);

fn reset_stopwatch(mut stopwatch: ResMut<SelectedHexagonStopwatch>) {
    stopwatch.0.reset();
}

fn add_stopwatch(mut commands: Commands) {
    commands.insert_resource(SelectedHexagonStopwatch(Stopwatch::new()));
}

// why use a component here?
fn raise_active(
    mut transform: Query<(&mut Transform, &VisualOf)>,
    time: Res<Time>,
    mut stopwatch: ResMut<SelectedHexagonStopwatch>,
    active: Res<ActiveTile>,
) {
    const OFFSET: f32 = 1.0;
    const FREQUENCY: f32 = 2.5;
    const AMPLITUDE: f32 = 0.3;
    const INITIAL_SHARPNESS: f32 = 0.1;

    stopwatch.0.tick(time.delta());
    let t = stopwatch.0.elapsed_secs();

    for (mut transform, visual_of) in transform {
        if visual_of.0 == active.read() {
            transform.translation.y = (-INITIAL_SHARPNESS / (t + INITIAL_SHARPNESS))
                + OFFSET
                + AMPLITUDE * (FREQUENCY * t).sin();
            return;
        }
    }
}

fn raise_valid_moves(
    mut rings: Query<(&mut Transform, &VisualOf), With<VisualTile>>,
    highlighted: Query<&ValidMove>,
    time: Res<Time>,
) {
    const TARGET_HEIGHT: f32 = 1.0;
    const SPEED: f32 = 0.5;

    for (mut transform, visual_of) in rings.iter_mut() {
        if highlighted.get(visual_of.0).is_ok() {
            transform.translation.y = {
                let this = transform.translation.y;
                let t = time.delta_secs() * SPEED;
                this * (1. - t) + TARGET_HEIGHT * t
            }
        } else {
            transform.translation.y = {
                let this = transform.translation.y;
                let t = time.delta_secs() * SPEED;
                this * (1. - t) + 0.0 * t
            }
        }
    }
}

// A HexagonMesh (with peice and ring children.)
// The mesh is located at a "y" of negative "CONSTANT" based on how where the center of the tile mesh is.
// Resource for the active HexagonMesh, which is updated whenever the active tile changes.
