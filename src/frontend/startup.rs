use bevy::prelude::*;

pub struct StartupEvents;

impl Plugin for StartupEvents {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, send_game_setup_instructions);
    }
}

fn send_game_setup_instructions(mut commands: Commands) {
    commands.trigger(crate::backend::game_parameters::CreateGame {
        board_size: crate::backend::game_parameters::BoardSize::Small,
    });
}
