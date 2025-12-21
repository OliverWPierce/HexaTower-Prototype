use std::f32::consts::PI;

use bevy::{prelude::*, time::Stopwatch};

use crate::{
    backend::{
        game_actions::{EvaluateEligibility, ExecuteSelectedAction, IsEligible, SelectionRequest},
        game_parameters::SetUpBoard,
        tiles::{LogTileDeleted, LogicalTileCreated, LogicalTileLocation},
    },
    frontend::FrontEndUpdateSystems,
};

pub struct TileSelectionIndicationPlugin;

impl Plugin for TileSelectionIndicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, initialize_indicator_model_handles);

        // app.add_systems(Update, (update_animations,).in_set(FrontEndUpdateSystems));

        // app.add_systems(
        //     EvaluateEligibility,
        //     update_indicator_states.in_set(FrontEndUpdateSystems),
        // );
        // app.add_systems(
        //     ExecuteSelectedAction,
        //     update_indicator_states.in_set(FrontEndUpdateSystems),
        // );

        app.add_systems(
            Update,
            (
                add_indicators,
                remove_indicators,
                change_indicator,
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
    ineligibe: Handle<Scene>,
}

fn initialize_indicator_model_handles(mut commands: Commands, assets: ResMut<AssetServer>) {
    commands.insert_resource(IndicatorHandles {
        selected: assets.load(GltfAssetLabel::Scene(0).from_asset("3d Selected Indicator.glb")),
        unselected: assets.load(GltfAssetLabel::Scene(0).from_asset("3d Selectable Indicator.glb")),
        ineligibe: assets.load(GltfAssetLabel::Scene(0).from_asset("IneligibleMarker(Test).glb")),
    });
}

#[derive(Debug, Clone, Copy, Component)]
enum ScaleMode {
    In,
    Out {
        despawn: bool,
    },
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
                    transform.scale = Vec3::ONE;
                    commands
                        .entity(indicator)
                        .try_remove::<AnimationInstructions>();
                } else {
                    transform.scale =
                        Vec3::splat(SCALE_MULTIPLIER * (mapped_time * SCALE_SPEED).sin());
                }
            }
            ScaleMode::Out { despawn } => {
                let mut mapped_time =
                    instructions.t.elapsed_secs() - instructions.offset as f32 / 50.0;
                if mapped_time < 0.0 {
                    mapped_time = 0.0
                }
                const SPEED: f32 = 30.0;

                let time_squared = mapped_time * mapped_time;
                let time_finished_squared = 1.0 / SPEED;

                if time_squared >= time_finished_squared {
                    if *despawn {
                        commands.entity(indicator).despawn();
                    } else {
                        commands.entity(indicator).remove::<AnimationInstructions>();
                        transform.scale = Vec3::ZERO;
                        println!("Removed the scene root and animation instructions.")
                    }
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
                    transform.scale = Vec3::ONE;
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

enum IndicatorState {
    Selected,
    Unselected,
    Ineligible,
}

#[derive(Component)]
pub struct Indicator {
    pub watches: Entity,
    state: IndicatorState,
}

fn add_indicators(
    mut new_tiles: MessageReader<LogicalTileCreated>,
    tile_locations: Query<&LogicalTileLocation>,
    mut commands: Commands,
) {
    for LogicalTileCreated(tile) in new_tiles.read() {
        if let Ok(location) = tile_locations.get(*tile) {
            commands.spawn((
                Indicator {
                    watches: *tile,
                    state: IndicatorState::Ineligible,
                },
                Transform::default().with_translation(Vec3 {
                    x: location.read().x,
                    y: 0.0,
                    z: location.read().y,
                }),
            ));
        }
    }
}

fn remove_indicators(
    mut despawned_tiles: MessageReader<LogTileDeleted>,
    indicators: Query<(Entity, &Indicator)>,
    mut commands: Commands,
) {
    for LogTileDeleted(tile) in despawned_tiles.read() {
        for (indicator, info) in indicators {
            if info.watches == *tile {
                commands.entity(indicator).insert(AnimationInstructions {
                    t: Stopwatch::new(),
                    mode: ScaleMode::Out { despawn: true },
                    offset: 0,
                });
                break;
            }
        }
    }
}

fn change_indicator(
    eligible_tiles: Query<&IsEligible>,
    mut indicators: Query<(Entity, &mut Indicator)>,
    mut commands: Commands,
    models: Res<IndicatorHandles>,
) {
    for (indicator, mut info) in indicators.iter_mut() {
        if let Ok(eligibility_info) = eligible_tiles.get(info.watches) {
            match eligibility_info.selected {
                true => match info.state {
                    IndicatorState::Selected => {
                        //do nothing
                    }
                    IndicatorState::Unselected => {
                        info.state = IndicatorState::Selected;
                        commands
                            .entity(indicator)
                            .insert(SceneRoot(models.selected.clone()));
                    }
                    IndicatorState::Ineligible => {
                        info.state = IndicatorState::Selected;
                        commands
                            .entity(indicator)
                            .insert(SceneRoot(models.selected.clone()));
                    }
                },
                false => match info.state {
                    IndicatorState::Selected => {
                        info.state = IndicatorState::Unselected;
                        commands
                            .entity(indicator)
                            .insert(SceneRoot(models.unselected.clone()));
                    }
                    IndicatorState::Unselected => {
                        //do nothing
                    }
                    IndicatorState::Ineligible => {
                        info.state = IndicatorState::Unselected;
                        commands
                            .entity(indicator)
                            .insert(SceneRoot(models.unselected.clone()));
                    }
                },
            }
        } else {
            match info.state {
                IndicatorState::Selected => {
                    info.state = IndicatorState::Ineligible;
                    commands
                        .entity(indicator)
                        .insert(SceneRoot(models.ineligibe.clone()));
                }
                IndicatorState::Unselected => {
                    info.state = IndicatorState::Ineligible;
                    commands
                        .entity(indicator)
                        .insert(SceneRoot(models.ineligibe.clone()));
                }
                IndicatorState::Ineligible => {
                    //do nothing
                }
            }
        }
    }
}
