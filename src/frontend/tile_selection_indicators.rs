use bevy::prelude::*;

use crate::backend::game_parameters::SetUpBoard;

pub struct TileSelectionIndicationPlugin;

impl Plugin for TileSelectionIndicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, initialize_indicator_model_handles);
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

#[derive(Debug, Component)]
enum ScaleMode {
    In,
    Out,
}
