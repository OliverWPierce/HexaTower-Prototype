use bevy::prelude::*;

pub struct InputAndGraphics;

impl Plugin for InputAndGraphics {
    fn build(&self, app: &mut App) {
        app.add_plugins((StartScreen, UiCam, ThemePlugin));
    }
}

mod input_reactivity;
mod player_creation;
mod start_screen;
mod themes;
mod ui_cam;

pub use input_reactivity::*;
pub use player_creation::*;
pub use start_screen::*;
pub use themes::*;
pub use ui_cam::*;
