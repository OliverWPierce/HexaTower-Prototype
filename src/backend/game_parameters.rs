use std::process::Command;

use bevy::prelude::*;

pub struct GameParameters;

impl Plugin for GameParameters {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerCount>();
        app.add_event::<SetupInstructions>();
        app.add_event::<BoardSize>();
        app.add_observer(start_setup);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Event)]
pub struct SetupInstructions {
    player_count: PlayerCount,
    board_size: BoardSize,
}

pub const PLAYER_COUNTS: [PlayerCount; 5] = [
    PlayerCount::Two,
    PlayerCount::Three,
    PlayerCount::Four,
    PlayerCount::Five,
    PlayerCount::Six,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Event, Component)]
#[repr(u8)]
pub enum PlayerCount {
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
}

pub const BOARD_SIZES: [BoardSize; 4] = [
    BoardSize::Small,
    BoardSize::Medium,
    BoardSize::Large,
    BoardSize::ExtraLarge,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Event, Component)]
pub enum BoardSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

fn start_setup(trigger: Trigger<SetupInstructions>, mut commands: Commands) {
    commands.trigger(trigger.player_count);
    commands.trigger(trigger.board_size);
}
