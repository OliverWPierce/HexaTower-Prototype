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
        board_size: crate::backend::game_parameters::BoardSize::Medium,
        player_instructions: PlayersToCreate(vec![
            PlayerCreationInstructions {
                name: String::from("Gavin"),
                base_pate_path: String::from("base_plates/RedPieceBasePlate.glb"),
            },
            PlayerCreationInstructions {
                name: String::from("Oliver"),
                base_pate_path: String::from("base_plates/BluePieceBasePlate.glb"),
            },
            PlayerCreationInstructions {
                name: String::from("Tyce"),
                base_pate_path: String::from("base_plates/ForestGreenPieceBasePlate.glb"),
            },
        ]),
    });
}
