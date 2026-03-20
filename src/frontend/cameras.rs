use std::f32::consts::PI;

use crate::backend::game_parameters::{BoardSize, SetUpBoard};
use bevy::{camera::Viewport, prelude::*, window::WindowResized};

pub struct TmpCamAndLights;

impl Plugin for TmpCamAndLights {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, (lights, cam_3d, ui_cam));
        app.add_systems(SetUpBoard, initial_resize_event.after(cam_3d));
        app.add_systems(Update, (resize_3d_viewport, move_3d_cam));
    }
}
#[derive(Debug, Component)]
struct InGame3dCam;

fn cam_3d(board_size: Res<BoardSize>, mut commands: Commands) {
    let distance = match *board_size {
        BoardSize::Small => 16.0,
        BoardSize::Medium => 22.0,
        BoardSize::Large => 25.0,
        BoardSize::ExtraLarge => 38.0,
    };

    let angle = match *board_size {
        BoardSize::Small => PI / 3.0,
        BoardSize::Medium => PI / 2.9,
        BoardSize::Large => PI / 2.8,
        BoardSize::ExtraLarge => PI / 5.0,
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
        Camera {
            order: 0,
            ..Default::default()
        },
        InGame3dCam,
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

/// As a percent
pub const LOWER_PANEL_HEIGHT: f32 = 20.0;
/// As a percent
pub const LEFT_PANEL_WIDTH: f32 = 20.0;
/// As a percent
pub const RIGHT_PANEL_WIDTH: f32 = 17.0;

fn resize_3d_viewport(
    windows: Query<&Window>,
    mut resize_events: MessageReader<WindowResized>,
    mut cam_3d: Single<&mut Camera, With<InGame3dCam>>,
) {
    for resize_event in resize_events.read() {
        let window = windows.get(resize_event.window).unwrap();

        cam_3d.viewport = Some(Viewport {
            physical_position: UVec2 {
                x: window.physical_width() * (LEFT_PANEL_WIDTH as u32) / 100,
                y: 0,
            },
            physical_size: UVec2 {
                x: window.physical_width()
                    * ((100.0 - LEFT_PANEL_WIDTH - RIGHT_PANEL_WIDTH) as u32)
                    / 100,
                y: window.physical_height() * ((100.0 - LOWER_PANEL_HEIGHT) as u32) / 100,
            },
            ..default()
        });
    }
}

// this system just emits an event with the same window info as it starts with to get the "resize_3d_viewport" function to run without the player needing to resize the window.
fn initial_resize_event(
    mut writer: MessageWriter<WindowResized>,
    windows: Query<(&Window, Entity)>,
) {
    for (window, window_entity) in windows {
        writer.write(WindowResized {
            window: window_entity,
            width: window.width(),
            height: window.height(),
        });
    }
}

fn move_3d_cam(
    inputs: Res<ButtonInput<KeyCode>>,
    camera: Single<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
) {
    const VERT_SPEED: f32 = 1.2;
    const HORIZONTAL_SPEED: f32 = 0.7;

    camera.into_inner().translation += Vec3 {
        x: (inputs.pressed(KeyCode::KeyD) as i8 - inputs.pressed(KeyCode::KeyA) as i8) as f32
            * HORIZONTAL_SPEED,
        y: (inputs.pressed(KeyCode::Space) as i8 - inputs.pressed(KeyCode::ShiftLeft) as i8) as f32
            * VERT_SPEED,
        z: (inputs.pressed(KeyCode::KeyS) as i8 - inputs.pressed(KeyCode::KeyW) as i8) as f32
            * HORIZONTAL_SPEED,
    } * time.delta_secs();
}
