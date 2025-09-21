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
        ));
    }
}

#[derive(Debug, Component)]
#[relationship (relationship_target = AbsoluteData)]
pub struct VisualOf(pub Entity);

#[derive(Debug, Component)]
#[relationship_target (relationship = VisualOf)]
pub struct AbsoluteData(Entity);

mod hexagons;
mod in_game_ui;
mod input_reactivity;
mod player_creation;
mod start_screen;
mod themes;
mod ui_cam;

pub use hexagons::*;
pub use in_game_ui::*;
pub use input_reactivity::*;
pub use player_creation::*;
pub use start_screen::*;
pub use themes::*;
pub use ui_cam::*;
