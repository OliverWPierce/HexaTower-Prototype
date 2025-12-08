use std::f32::consts::PI;

use bevy::{prelude::*, time::Stopwatch};

use crate::{
    backend::{
        game_actions::{EvaluateEligibility, ExecuteSelectedAction, IsEligible},
        game_parameters::SetUpBoard,
        tiles::LogicalTileLocation,
    },
    frontend::FrontEndUpdateSystems,
};

pub struct TileSelectionIndicationPlugin;

impl Plugin for TileSelectionIndicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, initialize_indicator_model_handles);

        app.add_systems(Update, (update_animations,).in_set(FrontEndUpdateSystems));

        app.add_systems(
            EvaluateEligibility,
            update_indicator_states.in_set(FrontEndUpdateSystems),
        );
        app.add_systems(
            ExecuteSelectedAction,
            update_indicator_states.in_set(FrontEndUpdateSystems),
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
struct IndicatorInfo {
    tile_watched: Entity,
    visually_selected: bool,
}

#[derive(Debug, Clone, Copy, Component)]
enum ScaleMode {
    In,
    Out,
    Pop {
        already_swapped_model: bool,
        to_selected: bool,
    },
}

#[derive(Component, Clone)]
struct AnimationInstructions {
    t: Stopwatch,
    mode: ScaleMode,
    offset: i32,
}

fn update_indicator_states(
    eligible_tiles: Query<(Entity, &IsEligible, &LogicalTileLocation)>,
    existing_indicators: Query<(Entity, &IndicatorInfo)>,
    models: Res<IndicatorHandles>,
    mut commands: Commands,
) {
    for (
        indicator,
        IndicatorInfo {
            tile_watched,
            visually_selected,
        },
    ) in existing_indicators
    {
        if let Ok((_, IsEligible { selected }, _)) = eligible_tiles.get(*tile_watched)
            && selected != visually_selected
        {
            commands.entity(indicator).insert(AnimationInstructions {
                t: Stopwatch::new(),
                mode: ScaleMode::Pop {
                    already_swapped_model: false,
                    to_selected: *selected,
                },
                offset: 0,
            });
        } else {
            commands.entity(indicator).insert(AnimationInstructions {
                t: Stopwatch::new(),
                mode: ScaleMode::Out,
                offset: 0,
            });
        }
    }

    for (tile, eligibility, location) in eligible_tiles {
        let mut has_indicator: bool = false;

        for (
            _,
            IndicatorInfo {
                tile_watched,
                visually_selected: _,
            },
        ) in existing_indicators
        {
            if tile == *tile_watched {
                has_indicator = true;
                break;
            }
        }

        if !has_indicator {
            commands.spawn((
                IndicatorInfo {
                    tile_watched: tile,
                    visually_selected: eligibility.selected,
                },
                AnimationInstructions {
                    t: Stopwatch::new(),
                    mode: ScaleMode::In,
                    offset: 0,
                },
                Transform::default()
                    .with_translation(Vec3 {
                        x: location.read().x,
                        y: 0.0,
                        z: location.read().y,
                    })
                    .with_scale(Vec3::ZERO),
                SceneRoot(match eligibility.selected {
                    true => models.selected.clone(),
                    false => models.unselected.clone(),
                }),
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
            ScaleMode::Pop {
                already_swapped_model,
                to_selected,
            } => {
                const SPEED: f32 = 20.0;
                const MAX_SIZE_BOOST: f32 = 0.15;
                let mut mapped_time =
                    instructions.t.elapsed_secs() - instructions.offset as f32 / 30.0;
                if mapped_time < 0.0 {
                    mapped_time = 0.0
                }

                let time_finished = 2.0 * PI / SPEED;
                let swap_time = PI / (SPEED * 2.0);

                if mapped_time >= swap_time && !*already_swapped_model {
                    *already_swapped_model = true;
                    commands.entity(indicator).insert(match to_selected {
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
