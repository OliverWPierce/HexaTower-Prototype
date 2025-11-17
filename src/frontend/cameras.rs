use std::f32::consts::PI;

use bevy::{core_pipeline::bloom::Bloom, math::VectorSpace, prelude::*};

use crate::backend::{AppState, BoardSize};

pub struct UiCam;

impl Plugin for UiCam {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::GameParameters), ui_cam);
        app.add_observer(board_cam_params);
        app.add_systems(OnEnter(AppState::InGame), (in_game_cams, lights));
        app.add_systems(Update, tmp_cam_controls);
    }
}

fn ui_cam(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        IsDefaultUiCamera,
        Camera {
            order: 1,
            ..default()
        },
    ));
}

#[derive(Debug, Resource)]
struct BoardCamParameters {
    angle: f32,
    distance: f32,
}

fn board_cam_params(trigger: Trigger<BoardSize>, mut commands: Commands) {
    let distance = match trigger.event() {
        BoardSize::Small => 16.0,
        BoardSize::Medium => 22.0,
        BoardSize::Large => 25.0,
        BoardSize::ExtraLarge => 50.0,
    };

    let angle = match trigger.event() {
        BoardSize::Small => PI / 3.0,
        BoardSize::Medium => PI / 2.9,
        BoardSize::Large => PI / 2.8,
        BoardSize::ExtraLarge => PI / 2.7,
    };

    commands.insert_resource(BoardCamParameters { angle, distance });
}

#[derive(Debug, Component)]
struct CamControler;

fn in_game_cams(cam_params: Res<BoardCamParameters>, mut commands: Commands) {
    commands.spawn((
        Transform::default(),
        CamControler,
        children![(
            Transform::default()
                .with_translation(
                    Vec3::default()
                        .with_z(cam_params.angle.cos() * cam_params.distance)
                        .with_y(cam_params.angle.sin() * cam_params.distance)
                )
                .looking_at(Vec3::ZERO, Vec3::Y),
            Camera3d::default(),
            Camera {
                order: 0,
                ..default()
            },
            Bloom::NATURAL
        )],
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

fn tmp_cam_controls(
    cam: Single<&mut Transform, With<CamControler>>,
    inputs: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if inputs.pressed(KeyCode::KeyQ) {
        cam.into_inner().rotate_y(0.5 * time.delta_secs());
    } else if inputs.pressed(KeyCode::KeyE) {
        cam.into_inner().rotate_y(-0.5 * time.delta_secs());
    }
}
