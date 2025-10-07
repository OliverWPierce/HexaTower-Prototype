use std::f32::consts::PI;

use bevy::{core_pipeline::bloom::Bloom, prelude::*};

use crate::backend::{AppState, BoardSize};

pub struct UiCam;

impl Plugin for UiCam {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::GameParameters), ui_cam);
        app.add_observer(board_cam_params);
        app.add_systems(OnEnter(AppState::InGame), in_game_cams);
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
        BoardSize::Small => 10.0,
        BoardSize::Medium => 14.0,
        BoardSize::Large => 16.0,
        BoardSize::ExtraLarge => 30.0,
    };

    let angle = match trigger.event() {
        BoardSize::Small => PI / 3.0,
        BoardSize::Medium => PI / 3.2,
        BoardSize::Large => PI / 3.6,
        BoardSize::ExtraLarge => PI / 4.0,
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
