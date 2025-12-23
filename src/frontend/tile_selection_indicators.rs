use bevy::prelude::*;
use rand::{rng, seq::IteratorRandom};

use crate::{
    backend::{
        game_actions::{
            ActionOrSelectionChanged, CurrentAction, EligibleForSelection, LogicallySelected,
        },
        game_parameters::SetUpBoard,
        tiles::{LogTileDeleted, LogicalTileCreated, LogicalTileLocation},
    },
    frontend::FrontEndUpdateSystems,
};

pub struct TileSelectionIndicationPlugin;

impl Plugin for TileSelectionIndicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            ActionOrSelectionChanged,
            update_indicators.in_set(FrontEndUpdateSystems),
        );

        app.add_systems(SetUpBoard, initialize_indicator_model_handles);

        app.add_systems(
            Update,
            (remove_indicators, add_indicators).in_set(FrontEndUpdateSystems),
        );
    }
}

#[derive(Debug, Resource, Clone)]
struct IndicatorHandles {
    selected: Handle<Scene>,
    selectable_but_unselected: Handle<Scene>,
    not_selected_or_selectable: Handle<Scene>,
}

fn initialize_indicator_model_handles(mut commands: Commands, assets: ResMut<AssetServer>) {
    commands.insert_resource(IndicatorHandles {
        selected: assets.load(GltfAssetLabel::Scene(0).from_asset("3d Selected Indicator.glb")),
        selectable_but_unselected: assets
            .load(GltfAssetLabel::Scene(0).from_asset("3d Selectable Indicator.glb")),
        not_selected_or_selectable: assets
            .load(GltfAssetLabel::Scene(0).from_asset("IneligibleMarker(Test).glb")),
    });
}

#[derive(Debug, Component, Clone, Copy)]
struct Indicator {
    watches_log_tile: Entity,
}

fn update_indicators(
    mut commands: Commands,
    indicators: Query<(Entity, &Indicator)>,
    eligible_tiles: Query<(), With<EligibleForSelection>>,
    selected_tiles: Query<(), With<LogicallySelected>>,
    handles: Res<IndicatorHandles>,
) {
    for (indicator, indicator_data) in indicators {
        if selected_tiles.contains(indicator_data.watches_log_tile) {
            commands
                .entity(indicator)
                .insert(SceneRoot(handles.selected.clone()));
        } else if eligible_tiles.contains(indicator_data.watches_log_tile) {
            commands
                .entity(indicator)
                .insert(SceneRoot(handles.selectable_but_unselected.clone()));
        } else {
            commands
                .entity(indicator)
                .insert(SceneRoot(handles.not_selected_or_selectable.clone()));
        }
    }
}

fn add_indicators(
    mut new_tiles: MessageReader<LogicalTileCreated>,
    tile_locations: Query<&LogicalTileLocation>,
    mut commands: Commands,
) {
    let options = -2..2;
    let mut rng = rng();

    for LogicalTileCreated(tile) in new_tiles.read() {
        if let Ok(location) = tile_locations.get(*tile) {
            commands.spawn((
                Indicator {
                    watches_log_tile: *tile,
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
                            .expect("the rotation range had length zero")
                            as f32
                            / 16.0,
                    )),
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
            if info.watches_log_tile == *tile {
                commands.entity(indicator).despawn();
                break;
            }
        }
    }
}
