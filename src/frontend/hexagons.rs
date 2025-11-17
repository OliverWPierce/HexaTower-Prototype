use crate::{
    backend::{AppState, ChangedActiveTile, TileCreated, TileLocation, ValidMove},
    frontend::{Watches, hexagons},
};

use bevy::prelude::*;

pub struct HexagonsPlugin;

impl Plugin for HexagonsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_visuals);
        app.add_systems(Startup, initialize_handles);
    }
}

fn spawn_visuals(
    mut reader: EventReader<TileCreated>,
    mut commands: Commands,
    basic_hex: Res<BasicHexHandle>,
    tiles: Query<&TileLocation>,
) {
    for created in reader.read() {
        let true_pos = tiles
            .get(created.0)
            .expect("The tile had no location")
            .read();
        let translation = Vec3::new(true_pos.x, 0.0, true_pos.y);

        commands.spawn((
            Transform::default().with_translation(translation),
            Watches(created.0),
            children![(SceneRoot(basic_hex.0.clone()))],
        ));
    }
}

#[derive(Resource, Debug)]
struct BasicHexHandle(Handle<Scene>);

#[derive(Resource, Debug)]
struct ActiveHexHandle(Handle<Scene>);

#[derive(Resource, Debug)]
struct ValidHexHandle(Handle<Scene>);

fn initialize_handles(mut commands: Commands, assets: ResMut<AssetServer>) {
    commands.insert_resource(BasicHexHandle(
        assets.load(GltfAssetLabel::Scene(0).from_asset("BasicTile.glb")),
    ));

    commands.insert_resource(ActiveHexHandle(
        assets.load(GltfAssetLabel::Scene(0).from_asset("ActiveTile.glb")),
    ));

    commands.insert_resource(ValidHexHandle(
        assets.load(GltfAssetLabel::Scene(0).from_asset("ValidTile.glb")),
    ));
}
