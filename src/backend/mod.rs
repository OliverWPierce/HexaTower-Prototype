use bevy::prelude::*;

pub struct GameLogic;

impl Plugin for GameLogic {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TilesPlugin,
            GameParametersPlugin,
            PiecesPlugin,
            GameActionsPlugin,
            CardsPlugin,
        ));
    }
}

#[derive(Debug, SystemSet, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct BackEndSystems;

pub mod cards;
pub mod game_actions;
pub mod game_parameters;
pub mod pieces;
pub mod tiles;

pub use cards::CardsPlugin;
pub use game_actions::GameActionsPlugin;
pub use game_parameters::GameParametersPlugin;
pub use pieces::PiecesPlugin;
pub use tiles::TilesPlugin;
