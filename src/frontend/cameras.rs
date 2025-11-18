use std::f32::consts::PI;

use crate::backend::game_parameters::{BoardSize, SetUpBoard};
use bevy::prelude::*;

pub struct TmpCamAndLights;

impl Plugin for TmpCamAndLights {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, (lights, cameras));
    }
}

fn cameras(board_size: Res<BoardSize>, mut commands: Commands) {
    let distance = match *board_size {
        BoardSize::Small => 16.0,
        BoardSize::Medium => 22.0,
        BoardSize::Large => 25.0,
        BoardSize::ExtraLarge => 50.0,
    };

    let angle = match *board_size {
        BoardSize::Small => PI / 3.0,
        BoardSize::Medium => PI / 2.9,
        BoardSize::Large => PI / 2.8,
        BoardSize::ExtraLarge => PI / 2.7,
    };

    commands.spawn((
        Transform::default()
            .with_translation(
                Vec3::default()
                    .with_z(angle.cos() * distance)
                    .with_y(angle.sin() * distance),
            )
            .looking_at(Vec3::ZERO, Vec3::Y),
        Camera3d::default(),
    ));
}

fn lights(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 3000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::default()
            .with_translation(vec3(100.0, 200.0, 300.0))
            .looking_at(Vec3::ZERO, Dir3::Y),
    ));

    commands.spawn((
        PointLight {
            intensity: 120000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::default().with_translation(vec3(0.0, 2.5, 0.0)),
    ));
}
