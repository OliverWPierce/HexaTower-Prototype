use bevy::{prelude::*, time::Stopwatch};
use rand::{rng, seq::IteratorRandom};

use crate::{
    backend::{
        game_actions::{ActionOrSelectionChanged, EligibileForNextSelection, LogicallySelected},
        game_parameters::SetUpBoard,
        tiles::{LogTileDeleted, LogicalTileCreated, LogicalTileLocation},
    },
    frontend::FrontEndUpdateSystems,
};

pub struct TileSelectionIndicationPlugin;

impl Plugin for TileSelectionIndicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, initialize_indicator_model_handles);
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
