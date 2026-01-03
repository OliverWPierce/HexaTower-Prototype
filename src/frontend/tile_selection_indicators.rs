use core::panic;
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
        app.add_systems(SetUpBoard, initialize_indicator_model_handles);

        app.init_resource::<TimeSinceLastTileEvaluation>();

        app.add_systems(Update, spawn_indicators);
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
struct TileWatched(Entity);

#[derive(Debug, Component)]
struct TargetState(SelectionState);

#[derive(Debug, Component)]
struct MeshEntities {
    selected: Entity,
    eligible: Entity,
}

#[derive(Debug, Component)]
struct AnimationInstructions {
    offset: f32,
    mode: ScaleMode,
}
#[derive(Debug)]
enum ScaleMode {
    In { to_selected: bool },
    Pop,
    Out { despawn: bool },
}

#[derive(Debug, Resource, Default)]
struct TimeSinceLastTileEvaluation(Stopwatch);

fn spawn_indicators(
    mut new_tiles: MessageReader<LogicalTileCreated>,
    locations: Query<&LogicalTileLocation>,
    mut commands: Commands,
    meshes: Res<IndicatorHandles>,
) {
    let options = -2..2;
    let mut rng = rng();

    let selectable = SceneRoot(meshes.selectable_but_unselected.clone());
    let selected = SceneRoot(meshes.selected.clone());

    for LogicalTileCreated(tile) in new_tiles.read() {
        let Ok(tile_loc) = locations.get(*tile) else {
            warn!("A logical tile had no location!");
            return;
        };

        let indicator_parent = commands
            .spawn((
                Transform::default()
                    .with_translation(Vec3 {
                        x: tile_loc.read().x,
                        y: 0.0,
                        z: tile_loc.read().y,
                    })
                    .with_rotation(Quat::from_rotation_y(
                        options
                            .clone()
                            .choose(&mut rng)
                            .expect("the rotation range had length zero")
                            as f32
                            / 16.0,
                    )),
                TileWatched(*tile),
                TargetState(SelectionState::Neither),
            ))
            .id();

        let selected_indicator = commands
            .spawn((
                Transform::default(),
                selected.clone(),
                Visibility::Hidden,
                ChildOf(indicator_parent),
            ))
            .id();
        let selectable_indicator = commands
            .spawn((
                Transform::default(),
                selectable.clone(),
                Visibility::Hidden,
                ChildOf(indicator_parent),
            ))
            .id();

        commands.entity(indicator_parent).insert(MeshEntities {
            selected: selected_indicator,
            eligible: selectable_indicator,
        });

        println!("Spawned an indicator.")
    }
}

fn update_indicators(
    tiles: Query<&TileSelectionStatus>,
    mut indicators: Query<(Entity, &TileWatched, &mut TargetState)>,
    mut reset_elapsed_time: ResMut<TimeSinceLastTileEvaluation>,
) {
    reset_elapsed_time.0.reset();

    for (indicator, log_tile_watched, mut target_state) in indicators.iter_mut() {
        if let Ok(tile_state) = tiles.get(log_tile_watched.0) {
            match tile_state.read() {
                SelectionState::Selected => {
                    match target_state.0 {
                        SelectionState::Selected => (),
                        SelectionState::Eligible => todo!(), //Pop
                        SelectionState::Neither => todo!(),  // Scale in
                    }
                }
                SelectionState::Eligible => {
                    match target_state.0 {
                        SelectionState::Selected => todo!(), // Pop,
                        SelectionState::Eligible => (),
                        SelectionState::Neither => todo!(), // scale in
                    }
                }
                SelectionState::Neither => {
                    match target_state.0 {
                        SelectionState::Selected => todo!(), // scale out,
                        SelectionState::Eligible => todo!(), // scale out,
                        SelectionState::Neither => (),
                    }
                }
            }

            target_state.0 = tile_state.read()
        } else {
            // despawn the indicator
            // Problem: theoretically the player could keep an indicator alive indefinitely by constantly switching game actions as this would cause the animation to constantly reset.
        }
    }
}
