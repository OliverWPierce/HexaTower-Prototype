use bevy::prelude::*;

use crate::backend::players::{PlayerCreationInstructions, PlayersToCreate};

pub struct StartupEvents;

impl Plugin for StartupEvents {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, send_game_setup_instructions);
    }
}

fn send_game_setup_instructions(mut commands: Commands) {
    commands.trigger(crate::backend::game_parameters::CreateGame {
        board_size: crate::backend::game_parameters::BoardSize::ExtraLarge,
        player_instructions: PlayersToCreate(vec![
            PlayerCreationInstructions {
                name: String::from("Billy"),
            },
            PlayerCreationInstructions {
                name: String::from("Joe"),
            },
            PlayerCreationInstructions {
                name: String::from("XX_EsmereldaDaBoss458_XX"),
            },
        ]),
    });
}
