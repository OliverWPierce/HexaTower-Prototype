use std::f32::consts::PI;

use bevy::{prelude::*, time::Stopwatch};
use rand::{rng, seq::IteratorRandom};

use crate::{
    backend::{
        game_actions::{
            ActionOrSelectionChanged, EligibilityDeterminationMethod,
            dangerous_selection_mechanics::{SelectionState, TileSelectionStatus},
        },
        game_parameters::SetUpBoard,
        tiles::{
            EssentialTileCreationSystems, LogTileDeleted, LogicalTileCreated, LogicalTileLocation,
        },
    },
    frontend::FrontEndSystems,
};

pub struct TileSelectionIndicationPlugin;

impl Plugin for TileSelectionIndicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            SetUpBoard,
            initialize_indicator_model_handles.before(add_indicators),
        );

        // app.add_systems(Update, add_indicators.in_set(FrontEndSystems));
        app.add_systems(
            Update,
            delete_indicators
                .in_set(FrontEndSystems)
                .after(update_indicator_data),
        );
        app.add_systems(
            SetUpBoard,
            add_indicators
                .in_set(FrontEndSystems)
                .after(EssentialTileCreationSystems),
        );

        app.add_systems(Update, run_animations.in_set(FrontEndSystems));
        app.add_systems(
            ActionOrSelectionChanged,
            update_indicator_data.in_set(FrontEndSystems),
        );
    }
}

#[derive(Debug, Resource, Clone)]
struct IndicatorHandles {
    selected: Handle<Scene>,
    selectable_but_unselected: Handle<Scene>,
}

fn initialize_indicator_model_handles(mut commands: Commands, assets: ResMut<AssetServer>) {
    commands.insert_resource(IndicatorHandles {
        selected: assets.load(GltfAssetLabel::Scene(0).from_asset("IneligibleMarker(Test).glb")),
        selectable_but_unselected: assets
            .load(GltfAssetLabel::Scene(0).from_asset("3d Selectable Indicator.glb")),
        // not_selected_or_selectable: assets
        //     .load(GltfAssetLabel::Scene(0).from_asset("IneligibleMarker(Test).glb")),
    });
}
#[derive(Debug, Component)]
struct Indicator {
    watches: Entity,
    target_state: SelectionState,
}

fn add_indicators(
    mut new_log_tiles: MessageReader<LogicalTileCreated>,
    locations: Query<&LogicalTileLocation>,
    mut commands: Commands,
) {
    let options = -2..2;
    let mut rng = rng();

    for LogicalTileCreated(log_tile) in new_log_tiles.read() {
        let Ok(location) = locations.get(*log_tile) else {
            panic!("A logical tile had no location")
        };

        commands.spawn((
            Indicator {
                watches: *log_tile,
                target_state: SelectionState::Neither,
            },
            Transform::default()
                .with_translation(Vec3 {
                    x: location.read().x,
                    y: 0.0,
                    z: location.read().y,
                })
                .with_rotation(Quat::from_rotation_y(
                    options
                        .clone()
                        .choose(&mut rng)
                        .expect("the rotation range had length zero") as f32
                        / 16.0,
                )),
        ));
    }
}

#[derive(Debug, Component, Clone)]
struct AnimationData {
    mode: ScaleMode,
    t: Stopwatch,
    t_offset: f32,
}

#[derive(Debug, Clone)]
enum ScaleMode {
    In {
        use_model: Handle<Scene>,
        mesh_already_spawned: bool,
    },
    Pop {
        use_model: Handle<Scene>,
        mesh_already_swapped: bool,
    },
    Out {
        with_despawn: bool,
    },
}

impl From<ScaleMode> for AnimationData {
    fn from(mode: ScaleMode) -> Self {
        AnimationData {
            mode,
            t: Stopwatch::new(),
            t_offset: (0..10)
                .choose(&mut rng())
                .expect("The range for random offsets of indicaton animations was empty.")
                as f32
                / 30.0,
        }
    }
}

fn update_indicator_data(
    mut indicators: Query<(Entity, &mut Indicator)>,
    tile_states: Query<&TileSelectionStatus>,
    handles: Res<IndicatorHandles>,
    mut commands: Commands,
) {
    for (indicator_ent, mut indicator_data) in indicators.iter_mut() {
        if let Ok(tile_state) = tile_states.get(indicator_data.watches) {
            match indicator_data.target_state {
                SelectionState::Selected => match tile_state.read() {
                    SelectionState::Selected => (),
                    SelectionState::Eligible => {
                        commands.entity(indicator_ent).insert(AnimationData::from(
                            ScaleMode::Pop {
                                use_model: handles.selectable_but_unselected.clone(),
                                mesh_already_swapped: false,
                            },
                        ));

                        indicator_data.target_state = tile_state.read();
                    }
                    SelectionState::Neither => {
                        commands.entity(indicator_ent).insert(AnimationData::from(
                            ScaleMode::Out {
                                with_despawn: false,
                            },
                        ));
                        indicator_data.target_state = tile_state.read();
                    }
                },
                SelectionState::Eligible => match tile_state.read() {
                    SelectionState::Selected => {
                        commands.entity(indicator_ent).insert(AnimationData::from(
                            ScaleMode::Pop {
                                use_model: handles.selected.clone(),
                                mesh_already_swapped: false,
                            },
                        ));
                        indicator_data.target_state = tile_state.read();
                    }
                    SelectionState::Eligible => (),
                    SelectionState::Neither => {
                        commands.entity(indicator_ent).insert(AnimationData::from(
                            ScaleMode::Out {
                                with_despawn: false,
                            },
                        ));
                        indicator_data.target_state = tile_state.read();
                    }
                },
                SelectionState::Neither => match tile_state.read() {
                    SelectionState::Selected => {
                        commands
                            .entity(indicator_ent)
                            .insert(AnimationData::from(ScaleMode::In {
                                use_model: handles.selected.clone(),
                                mesh_already_spawned: false,
                            }));
                        indicator_data.target_state = tile_state.read();
                    }
                    SelectionState::Eligible => {
                        commands
                            .entity(indicator_ent)
                            .insert(AnimationData::from(ScaleMode::In {
                                use_model: handles.selectable_but_unselected.clone(),
                                mesh_already_spawned: false,
                            }));
                        indicator_data.target_state = tile_state.read();
                    }
                    SelectionState::Neither => (),
                },
            }
        }
    }
}

fn delete_indicators(
    mut despawned_tiles: MessageReader<LogTileDeleted>,
    indicators: Query<(Entity, &Indicator)>,
    mut commands: Commands,
) {
    for LogTileDeleted(tile) in despawned_tiles.read() {
        for (indicator, Indicator { watches, .. }) in indicators {
            if watches == tile {
                commands
                    .entity(indicator)
                    .insert(AnimationData::from(ScaleMode::Out { with_despawn: true }));

                break;
            }
        }
    }
}

fn run_animations(
    to_animate: Query<(Entity, &mut AnimationData, &mut Transform)>,
    time: Res<Time>,
    models: Res<IndicatorHandles>,
    mut commands: Commands,
) {
    const IN_SCALE_SPEED: f32 = 9.0;
    const IN_SCALE_MULTIPLIER: f32 = 1.1; // must be greater than 1
    // I cannot make this a constant, so I'm putting it outside for speed's sake. The constants must be outside for scope.
    let in_time_finished: f32 =
        (std::f32::consts::PI - (1.0 / IN_SCALE_MULTIPLIER).asin()) / IN_SCALE_SPEED;

    for (indicator, mut animation, mut transform) in to_animate {
        animation.t.tick(time.delta());
        let elapsed = animation.t.elapsed_secs();
        let offset = animation.t_offset;

        match &mut animation.mode {
            ScaleMode::In {
                use_model,
                mesh_already_spawned,
            } => {
                let elapsed_mapped = (elapsed - offset).clamp(0.0, f32::MAX);

                if !*mesh_already_spawned {
                    *mesh_already_spawned = true;
                    commands
                        .entity(indicator)
                        .insert(SceneRoot(use_model.clone()));
                }

                if elapsed_mapped >= in_time_finished {
                    transform.scale = Vec3::splat(1.0);
                    commands.entity(indicator).remove::<AnimationData>();
                } else {
                    transform.scale =
                        Vec3::splat(IN_SCALE_MULTIPLIER * (elapsed_mapped * IN_SCALE_SPEED).sin());
                }
            }
            ScaleMode::Pop {
                use_model,
                mesh_already_swapped,
            } => {
                const POP_SPEED: f32 = 10.0;
                const POP_MAX_SIZE_BOOST: f32 = 0.15;
                const POP_TIME_FINISHED: f32 = 2.0 * PI / POP_SPEED;
                const POP_SPWAP_TIME: f32 = PI / (POP_SPEED * 2.0);

                let elapsed_mapped = if *use_model == models.selected {
                    elapsed
                } else {
                    (elapsed - offset).clamp(0.0, f32::MAX)
                };

                if !*mesh_already_swapped && elapsed_mapped >= POP_SPWAP_TIME {
                    *mesh_already_swapped = true;
                    commands
                        .entity(indicator)
                        .insert(SceneRoot(use_model.clone()));
                    println!("swaped models");
                }

                if elapsed_mapped >= POP_TIME_FINISHED {
                    transform.scale = Vec3::splat(1.0);
                    commands.entity(indicator).remove::<AnimationData>();
                } else {
                    transform.scale = Vec3::splat(
                        -POP_MAX_SIZE_BOOST * (elapsed_mapped * POP_SPEED).cos()
                            + POP_MAX_SIZE_BOOST
                            + 1.0,
                    );
                }
            }
            ScaleMode::Out { with_despawn } => {
                const OUT_SPEED: f32 = 16.0;

                let elapsed_mapped = (elapsed - offset).clamp(0.0, f32::MAX);
                let mapped_squared = elapsed_mapped * elapsed_mapped;
                let time_finished_squared = 1.0 / OUT_SPEED;

                if mapped_squared >= time_finished_squared {
                    match with_despawn {
                        true => commands.entity(indicator).despawn(),
                        false => {
                            transform.scale = Vec3::ZERO;
                            commands
                                .entity(indicator)
                                .remove::<(AnimationData, SceneRoot)>();
                        }
                    }
                } else {
                    transform.scale = Vec3::splat(-OUT_SPEED * mapped_squared + 1.0);
                }
            }
        }
    }
}
