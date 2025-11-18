use bevy::prelude::*;

pub struct InputAndGraphics;

impl Plugin for InputAndGraphics {
    fn build(&self, app: &mut App) {
        app.add_plugins((TmpCamAndLights, StartupEvents, VisTilesPlugin));
    }
}

mod cameras;
mod startup;
mod visual_tiles;

pub use cameras::TmpCamAndLights;
pub use startup::StartupEvents;
pub use visual_tiles::VisTilesPlugin;
