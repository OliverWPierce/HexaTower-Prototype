use bevy::prelude::*;

pub struct GameParametersPlugin;

impl Plugin for GameParametersPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(set_up_resources_and_kickoff_setup_sequence);
    }
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Event)]
pub struct CreateGame {
    pub board_size: BoardSize,
}

use bevy::ecs::schedule::ScheduleLabel;
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct SetUpBoard;

fn set_up_resources_and_kickoff_setup_sequence(game: On<CreateGame>, mut commands: Commands) {
    commands.insert_resource(game.board_size);

    commands.run_schedule(SetUpBoard);
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Resource, Clone, Copy)]
pub enum BoardSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}
