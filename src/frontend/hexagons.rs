use std::thread::spawn;

use bevy::{
    ecs::component::TickCells, gltf::GltfMaterialName, math::VectorSpace, prelude::*,
    scene::SceneInstanceReady, time::Stopwatch,
};

use crate::{
    backend::{ActiveTile, AppState, TileCreated, TileLocation, ValidMove},
    frontend::Watches,
};

pub struct HexagonsPlugin;

impl Plugin for HexagonsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_visuals);
    }
}

#[derive(Debug, Component)]
struct Hexagon;

#[derive(Debug, Component)]
struct Ring;

fn spawn_visuals(
    mut reader: EventReader<TileCreated>,
    mut commands: Commands,
    server: ResMut<AssetServer>,
    tiles: Query<&TileLocation>,
) {
    let basic_hex = SceneRoot(server.load(GltfAssetLabel::Scene(0).from_asset("BasicTile.glb")));
    let hex_ring =
        SceneRoot(server.load(GltfAssetLabel::Scene(0).from_asset("TileHighlightRing.glb")));

    for created in reader.read() {
        let true_pos = tiles
            .get(created.0)
            .expect("The tile had no location")
            .read();
        let translation = Vec3::new(true_pos.x, 0.0, true_pos.y);

        commands.spawn((
            basic_hex.clone(),
            Hexagon,
            Transform::default().with_translation(translation),
            Watches(created.0),
            children![(
                Ring,
                hex_ring.clone(),
                Transform::default(),
                Watches(created.0)
            )],
        ));
    }
}
