use bevy::prelude::*;

pub struct GameLogic;

impl Plugin for GameLogic {
    fn build(&self, app: &mut App) {
        app.add_plugins((TilesPlugin, GameParametersPlugin, PiecesPlugin));
    }
}
pub mod game_parameters;
pub mod pieces;
pub mod tiles;

pub use game_parameters::GameParametersPlugin;
pub use pieces::PiecesPlugin;
pub use tiles::TilesPlugin;
