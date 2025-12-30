use bevy::prelude::*;

pub struct InputAndGraphics;

impl Plugin for InputAndGraphics {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TmpCamAndLights,
            StartupEvents,
            VisTilesPlugin,
            VisPiecesPlugin,
            TileSelectionIndicationPlugin,
            InputsPlugin,
        ));

        app.configure_sets(
            ActionOrSelectionChanged,
            (BackEndSystems, FrontEndSystems).chain(),
        );
        app.configure_sets(
            ExecuteSelectedAction,
            (BackEndSystems, FrontEndSystems).chain(),
        );

        app.configure_sets(SetUpBoard, (BackEndSystems, FrontEndSystems).chain());
    }
}

#[derive(Debug, SystemSet, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct FrontEndSystems;

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

use crate::backend::{
    BackEndSystems,
    game_actions::{ActionOrSelectionChanged, ExecuteSelectedAction},
    game_parameters::SetUpBoard,
};
