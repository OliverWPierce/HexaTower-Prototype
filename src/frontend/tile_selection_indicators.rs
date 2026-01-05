use bevy::prelude::*;
use rand::{rng, seq::IteratorRandom};

use crate::{
    backend::{
        game_actions::dangerous_selection_mechanics::{SelectionState, TileSelectionStatus},
        game_parameters::SetUpBoard,
        tiles::{LogicalTileCreated, LogicalTileLocation},
    },
    frontend::FrontEndSystems,
};

pub struct TileSelectionIndicationPlugin;

impl Plugin for TileSelectionIndicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, initialize_indicator_model_handles);

        app.add_systems(
            Update,
            (spawn_indicators, scale_indicators).in_set(FrontEndSystems),
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
        selected: assets.load(GltfAssetLabel::Scene(0).from_asset("3d Selected Indicator.glb")),
        selectable_but_unselected: assets
            .load(GltfAssetLabel::Scene(0).from_asset("3d Selectable Indicator.glb")),
        // not_selected_or_selectable: assets
        //     .load(GltfAssetLabel::Scene(0).from_asset("IneligibleMarker(Test).glb")),
    });
}

#[derive(Debug, Component)]
struct TileWatched(Entity);

/// The "x" value of the graph of the function describing an indicator's scale.
#[derive(Debug, Component)]
struct MapValue(f32);

#[derive(Debug, Component)]
struct RepresentsState(SelectionState);

fn spawn_indicators(
    mut new_tiles: MessageReader<LogicalTileCreated>,
    locations: Query<&LogicalTileLocation>,
    mut commands: Commands,
    meshes: Res<IndicatorHandles>,
) {
    let options = -2..2;
    let mut rng = rng();

    for LogicalTileCreated(tile) in new_tiles.read() {
        let Ok(tile_loc) = locations.get(*tile) else {
            warn!("A logical tile had no location!");
            return;
        };

        commands.spawn((
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
                        .expect("the rotation range had length zero") as f32
                        / 16.0,
                ))
                .with_scale(Vec3::ZERO),
            SceneRoot(meshes.selected.clone()),
            TileWatched(*tile),
            RepresentsState(SelectionState::Selected),
            MapValue(0.0),
        ));

        commands.spawn((
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
                        .expect("the rotation range had length zero") as f32
                        / 16.0,
                ))
                .with_scale(Vec3::ZERO),
            SceneRoot(meshes.selectable_but_unselected.clone()),
            TileWatched(*tile),
            RepresentsState(SelectionState::Eligible),
            MapValue(0.0),
        ));
    }
}

fn scale_indicators(
    indicators: Query<(
        Entity,
        &mut Transform,
        &mut MapValue,
        &RepresentsState,
        &TileWatched,
    )>,
    tiles: Query<&TileSelectionStatus>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let delta = time.delta_secs();
    const OVERSHOOT: f32 = 1.2;
    const SPEED: f32 = 3.5;

    for (indicator, mut transform, mut x, state, watched) in indicators {
        if let Ok(watched_state) = tiles.get(watched.0) {
            x.0 = (x.0
                + if watched_state.read() == state.0 {
                    delta * SPEED
                } else {
                    -delta * SPEED
                })
            .clamp(0.0, 1.0);

            let x = x.0;

            transform.scale =
                Vec3::splat((-2.0 * OVERSHOOT * x * x * x) + ((2.0 * OVERSHOOT + 1.0) * x * x));
        } else {
            x.0 -= delta * SPEED;
            let x = x.0;

            if x <= 0.0 {
                commands.entity(indicator).despawn();
            } else {
                transform.scale =
                    Vec3::splat((-2.0 * OVERSHOOT * x * x * x) + ((2.0 * OVERSHOOT + 1.0) * x * x));
            }
        }
    }
}
