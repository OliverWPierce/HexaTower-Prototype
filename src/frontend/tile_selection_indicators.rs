use bevy::{ecs::query, prelude::*};

use crate::{
    backend::{
        game_actions::{Selectable, Selected},
        game_parameters::SetUpBoard,
        tiles::LogicalTileLocation,
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
                selected_indicator_management,
                selectable_indicator_managemement,
            )
                .in_set(FrontEndUpdateSystems),
        );
    }
}

#[derive(Debug, Resource, Clone)]
struct IndicatorHandles {
    selected: Handle<Scene>,
    selectable: Handle<Scene>,
}

#[derive(Debug, Component, Clone, Copy)]
struct SelectedIndicatorWatching(Entity);

#[derive(Debug, Component, Clone, Copy)]
struct SelectableIndicatorWatching(Entity);

fn initialize_indicator_model_handles(mut commands: Commands, assets: ResMut<AssetServer>) {
    commands.insert_resource(IndicatorHandles {
        selected: assets.load(GltfAssetLabel::Scene(0).from_asset("3d Selected Indicator.glb")),
        selectable: assets.load(GltfAssetLabel::Scene(0).from_asset("3d Selectable Indicator.glb")),
    });
}

fn selected_indicator_management(
    newly_selected: Query<(Entity, &LogicalTileLocation), Added<Selected>>,
    mut recently_deselected: RemovedComponents<Selected>,
    selected_indicators: Query<(Entity, &SelectedIndicatorWatching)>,
    mut commands: Commands,
    handles: Res<IndicatorHandles>,
) {
    for (log_tile, position) in newly_selected {
        commands.spawn((
            Transform::default().with_translation(Vec3 {
                x: position.read().x,
                y: 0.0,
                z: position.read().y,
            }),
            SceneRoot(handles.selected.clone()),
            SelectedIndicatorWatching(log_tile),
        ));
    }
    for log_tile in recently_deselected.read() {
        for (indicator, entity_watched) in selected_indicators {
            if log_tile == entity_watched.0 {
                commands.entity(indicator).despawn();
            }
        }
    }
}

fn selectable_indicator_managemement(
    newly_selectable: Query<(Entity, &LogicalTileLocation), Added<Selectable>>,
    mut recently_made_unselectable: RemovedComponents<Selectable>,
    selectable_indicators: Query<(Entity, &SelectableIndicatorWatching)>,
    mut commands: Commands,
    handles: Res<IndicatorHandles>,
) {
    for (log_tile, position) in newly_selectable {
        commands.spawn((
            Transform::default().with_translation(Vec3 {
                x: position.read().x,
                y: 0.0,
                z: position.read().y,
            }),
            SceneRoot(handles.selectable.clone()),
            SelectableIndicatorWatching(log_tile),
        ));
    }

    for log_tile in recently_made_unselectable.read() {
        for (indicator, entity_watched) in selectable_indicators {
            if log_tile == entity_watched.0 {
                commands.entity(indicator).despawn();
            }
        }
    }
}
