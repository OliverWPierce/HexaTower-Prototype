use bevy::prelude::*;

use crate::backend::AppState;

pub struct UiCam;

impl Plugin for UiCam {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::GameParameters), ui_cam);
    }
}

fn ui_cam(mut commands: Commands) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
}
