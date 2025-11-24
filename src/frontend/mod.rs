use bevy::prelude::*;

pub struct InputAndGraphics;

impl Plugin for InputAndGraphics {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TmpCamAndLights,
            StartupEvents,
            VisTilesPlugin,
            VisPiecesPlugin,
        ));
    }
}

mod cameras;
mod startup;
mod visual_pieces;
mod visual_tiles;

pub use cameras::TmpCamAndLights;
pub use startup::StartupEvents;
pub use visual_pieces::VisPiecesPlugin;
pub use visual_tiles::VisTilesPlugin;
