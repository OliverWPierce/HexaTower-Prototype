use bevy::prelude::*;

use crate::backend::{
    game_parameters::SetUpBoard,
    tiles::{LogicalTileCreated, LogicalTileLocation},
};

pub struct VisTilesPlugin;

impl Plugin for VisTilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, create_vis_tiles_if_needed);
        app.add_systems(SetUpBoard, initialize_handles);
    }
}

#[derive(Debug, Clone, Copy, Component)]
struct VisTileOf(Entity);

fn create_vis_tiles_if_needed(
    mut reader: EventReader<LogicalTileCreated>,
    mut commands: Commands,
    log_tiles: Query<&LogicalTileLocation>,
    basic_hex: Res<BasicHexHandle>,
) {
    for log_tile in reader.read() {
        let true_pos = log_tiles
            .get(log_tile.0)
            .expect("The tile had no location")
            .read();
        let translation = Vec3::new(true_pos.x, 0.0, true_pos.y);

        commands.spawn((
            Transform::default().with_translation(translation),
            VisTileOf(log_tile.0),
            SceneRoot(basic_hex.0.clone()),
        ));
    }
}

#[derive(Resource, Debug)]
struct BasicHexHandle(Handle<Scene>);

fn initialize_handles(mut commands: Commands, assets: ResMut<AssetServer>) {
    commands.insert_resource(BasicHexHandle(
        assets.load(GltfAssetLabel::Scene(0).from_asset("BasicTile.glb")),
    ));
}
