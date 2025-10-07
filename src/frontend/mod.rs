use bevy::prelude::*;

pub struct InputAndGraphics;

impl Plugin for InputAndGraphics {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            StartScreen,
            UiCam,
            ThemePlugin,
            PlayerCreation,
            InGameUiPlugin,
            HexagonsPlugin,
            InputPlugin,
        ));
    }
}
#[derive(Debug, Component)]
pub struct Watches(pub Entity);

mod cameras;
mod hexagons;
mod in_game_ui;
mod input_reactivity;
mod player_creation;
mod start_screen;
mod themes;

pub use cameras::*;
pub use hexagons::*;
pub use in_game_ui::*;
pub use input_reactivity::*;
pub use player_creation::*;
pub use start_screen::*;
pub use themes::*;
