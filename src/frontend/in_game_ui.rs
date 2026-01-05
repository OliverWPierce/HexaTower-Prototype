use bevy::prelude::*;

use crate::backend::game_parameters::SetUpBoard;

pub struct InGameUI;

impl Plugin for InGameUI {
    fn build(&self, app: &mut App) {
        // app.add_systems(SetUpBoard, spawn_test_sprite);
    }
}

fn spawn_test_sprite(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(0.3)),
        Sprite::from_image(asset_server.load("Card1 copy.png")),
        Pickable {
            should_block_lower: true,
            is_hoverable: true,
        },
    ));
}
