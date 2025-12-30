use bevy::prelude::*;

use crate::backend::{
    game_actions::dangerous_selection_mechanics::TileSelectionStatus,
    tiles::{LogicalTileCreated, LogicalTileLocation},
};

pub struct TileSelectionIndicationPlugin;

impl Plugin for TileSelectionIndicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, initialize_indicator_model_handles);
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

struct Indicator {
    watches: Entity,
}

fn add_indicators(
    mut new_log_tiles: MessageReader<LogicalTileCreated>,
    locations: Query<(&LogicalTileLocation)>,
    mut commands: Commands,
) {
    for LogicalTileCreated(log_tile) in new_log_tiles.read() {
        let Ok(location) = locations.get(*log_tile) else {
            panic!("A logical tile had no location")
        };
    }
}
