use bevy::prelude::*;

pub struct InputAndGraphics;

impl Plugin for InputAndGraphics {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TmpCamAndLights,
            StartupEvents,
            VisTilesPlugin,
            VisPiecesPlugin,
            InputsPlugin,
        ));
    }
}

#[derive(Debug, SystemSet, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct FrontEndUpdateSystems;

mod cameras;
mod inputs;
mod startup;
mod tile_selection_indicators;
mod visual_pieces;
mod visual_tiles;

pub use cameras::TmpCamAndLights;
pub use inputs::InputsPlugin;
pub use startup::StartupEvents;
pub use tile_selection_indicators::TileSelectionIndicationPlugin;
pub use visual_pieces::VisPiecesPlugin;
pub use visual_tiles::VisTilesPlugin;
