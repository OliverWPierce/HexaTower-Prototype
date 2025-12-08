use std::f32::consts::PI;

use bevy::{prelude::*, time::Stopwatch};
use rand::seq::IndexedRandom;

use crate::{
    backend::{
        game_actions::EligibileTile, game_parameters::SetUpBoard, tiles::LogicalTileLocation,
    },
    frontend::FrontEndUpdateSystems,
};

pub struct TileSelectionIndicationPlugin;

impl Plugin for TileSelectionIndicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, initialize_indicator_model_handles);

        app.add_systems(
            Update,
            (
                modify_or_create_indicators_of_new_or_changed_tiles,
                start_deletion_animations,
                update_animations,
            )
                .chain()
                .in_set(FrontEndUpdateSystems),
        );
    }
}

#[derive(Debug, Resource, Clone)]
struct IndicatorHandles {
    selected: Handle<Scene>,
    unselected: Handle<Scene>,
}

fn initialize_indicator_model_handles(mut commands: Commands, assets: ResMut<AssetServer>) {
    commands.insert_resource(IndicatorHandles {
        selected: assets.load(GltfAssetLabel::Scene(0).from_asset("3d Selected Indicator.glb")),
        unselected: assets.load(GltfAssetLabel::Scene(0).from_asset("3d Selectable Indicator.glb")),
    });
}

#[derive(Debug, Clone, Copy, Component)]
struct IndicatorWatches(Entity);

#[derive(Debug, Clone, Copy, Component)]
enum ScaleMode {
    In,
    Out,
    Pop(bool),
}

#[derive(Component, Clone)]
struct AnimationInstructions {
    t: Stopwatch,
    mode: ScaleMode,
    to_selected: bool,
    offset: i32,
}

fn modify_or_create_indicators_of_new_or_changed_tiles(
    eligible_tiles_that_changed: Query<
        (Entity, &EligibileTile, &LogicalTileLocation),
        Changed<EligibileTile>,
    >,
    indicators: Query<(&IndicatorWatches, Entity)>,
    mut commands: Commands,
    models: Res<IndicatorHandles>,
) {
    let offset_options = [1, 2, 3, 4, 5, 6];
    let mut rng = rand::rng();

    for (tile, EligibileTile { selected }, location) in eligible_tiles_that_changed {
        let mut current_indicator = None;

        for (tile_watched, indicator) in indicators.iter() {
            if tile_watched.0 == tile {
                current_indicator = Some(indicator);
                break;
            }
        }

        if let Some(indicator) = current_indicator {
            commands.entity(indicator).insert(AnimationInstructions {
                t: Stopwatch::new(),
                mode: ScaleMode::Pop(false),
                to_selected: *selected,
                offset: 0,
            });
        } else {
            commands.spawn((
                IndicatorWatches(tile),
                Transform::default()
                    .with_translation(Vec3 {
                        x: location.read().x,
                        y: 0.0,
                        z: location.read().y,
                    })
                    .with_scale(Vec3::ZERO)
                    .with_rotation(Quat::from_rotation_y(
                        (*offset_options.choose(&mut rng).unwrap() - 3) as f32 / 30.0,
                    )),
                SceneRoot(match selected {
                    true => models.selected.clone(),
                    false => models.unselected.clone(),
                }),
                AnimationInstructions {
                    t: Stopwatch::new(),
                    mode: ScaleMode::In,
                    to_selected: *selected,
                    offset: *offset_options.choose(&mut rng).unwrap(),
                },
            ));
        }
    }
}

fn update_animations(
    models: Res<IndicatorHandles>,
    to_update: Query<(Entity, &mut Transform, &mut AnimationInstructions)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (indicator, mut transform, mut instructions) in to_update {
        instructions.t.tick(time.delta());
        let instructions = instructions.into_inner();

        match &mut instructions.mode {
            ScaleMode::In => {
                const SCALE_SPEED: f32 = 9.0;
                const SCALE_MULTIPLIER: f32 = 1.1; // must be greater than 1
                let mut mapped_time =
                    instructions.t.elapsed_secs() - instructions.offset as f32 / 30.0;
                let time_finished: f32 =
                    (std::f32::consts::PI - (1.0 / SCALE_MULTIPLIER).asin()) / SCALE_SPEED;

                if mapped_time < 0.0 {
                    mapped_time = 0.0
                }

                if mapped_time >= time_finished {
                    transform.scale = Vec3::splat(1.0);
                    commands
                        .entity(indicator)
                        .try_remove::<AnimationInstructions>();
                } else {
                    transform.scale =
                        Vec3::splat(SCALE_MULTIPLIER * (mapped_time * SCALE_SPEED).sin());
                }
            }
            ScaleMode::Out => {
                let mut mapped_time =
                    instructions.t.elapsed_secs() - instructions.offset as f32 / 50.0;
                if mapped_time < 0.0 {
                    mapped_time = 0.0
                }
                const SPEED: f32 = 30.0;

                let time_squared = mapped_time * mapped_time;
                let time_finished_squared = 1.0 / SPEED;

                if time_squared >= time_finished_squared {
                    commands.entity(indicator).despawn();
                } else {
                    transform.scale = Vec3::splat(-SPEED * time_squared + 1.0);
                }
            }
            ScaleMode::Pop(already_swapped) => {
                const SPEED: f32 = 20.0;
                const MAX_SIZE_BOOST: f32 = 0.15;
                let mut mapped_time =
                    instructions.t.elapsed_secs() - instructions.offset as f32 / 30.0;
                if mapped_time < 0.0 {
                    mapped_time = 0.0
                }

                let time_finished = 2.0 * PI / SPEED;
                let swap_time = PI / (SPEED * 2.0);

                if mapped_time >= swap_time && !*already_swapped {
                    *already_swapped = true;
                    commands
                        .entity(indicator)
                        .insert(match instructions.to_selected {
                            true => SceneRoot(models.selected.clone()),
                            false => SceneRoot(models.unselected.clone()),
                        });
                    println!("swaped models");
                }

                if mapped_time >= time_finished {
                    transform.scale = Vec3::splat(1.0);
                    commands
                        .entity(indicator)
                        .try_remove::<AnimationInstructions>();
                } else {
                    transform.scale = Vec3::splat(
                        -MAX_SIZE_BOOST * (mapped_time * SPEED).cos() + MAX_SIZE_BOOST + 1.0,
                    );
                }
            }
        }
    }
}

fn start_deletion_animations(
    mut no_longer_eligible_tiles: RemovedComponents<EligibileTile>,
    indicators: Query<(&IndicatorWatches, Entity)>,
    mut commands: Commands,
) {
    let offset_options = [1, 2, 3, 4, 5, 6];
    let mut rng = rand::rng();

    for tile in no_longer_eligible_tiles.read() {
        for (tile_watched, indicator) in indicators.iter() {
            if tile_watched.0 == tile {
                commands
                    .entity(indicator)
                    .try_insert(AnimationInstructions {
                        t: Stopwatch::new(),
                        mode: ScaleMode::Out,
                        to_selected: false,
                        offset: *offset_options.choose(&mut rng).unwrap(),
                    });
                break;
            }
        }
    }
}
