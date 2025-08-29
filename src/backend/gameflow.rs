use bevy::prelude::*;

#[derive(Debug, States, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    GameParameters,
    PlayerCreation,
    InGame,
    PostGame,
}

pub struct AppFlow;

impl Plugin for AppFlow {
    fn build(&self, app: &mut App) {
        app.insert_state(AppState::GameParameters);
    }
}
