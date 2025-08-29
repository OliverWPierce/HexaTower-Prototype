use core::str;

use bevy::prelude::*;

use crate::backend::{AppState, Class};

pub struct PlayerSpawningPlugin;

impl Plugin for PlayerSpawningPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerCreated>();
        app.add_event::<PlayerCount>();
        app.add_event::<PlayerCreationInstructions>();

        app.add_observer(spawn_player_entities);
        app.add_observer(add_core_player_components);
    }
}

#[derive(Resource, Debug)]
pub struct LivingPlayerIDs(Vec<PlayerID>);

#[derive(Resource, Debug)]
pub struct ActivePlayer(pub Entity);

pub struct GoToNextPlayer;

#[derive(Debug, Component)]
pub struct PlayerID(u8);

impl PlayerID {
    pub fn id(&self) -> u8 {
        self.0
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Event)]
#[repr(u8)]
pub enum PlayerCount {
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
}

fn spawn_player_entities(
    trigger: Trigger<PlayerCount>,
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
) {
    let mut ids: Vec<PlayerID> = Vec::new();

    for id in 0..*trigger.event() as u8 {
        commands.spawn(PlayerID(id));
        ids.push(PlayerID(id));
    }
    commands.insert_resource(LivingPlayerIDs(ids));
    state.set(AppState::PlayerCreation);
}

#[derive(Debug, Clone, Copy, PartialEq, Event)]
pub struct PlayerCreationInstructions {
    entity: Entity,
    class: Class,
    color: ThemeColorId,
}

#[derive(Debug, Event)]
pub struct PlayerCreated(pub Entity);

#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThemeColorId {
    Choice1,
    Choice2,
    Choice3,
    Choice4,
    Choice5,
    Choice6,
    Choice7,
    Choice8,
    Choice9,
    Choice10,
}

#[derive(Debug, Resource, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AvailibleThemeColors(Vec<ThemeColorId>);

impl Default for AvailibleThemeColors {
    fn default() -> Self {
        AvailibleThemeColors(vec![
            ThemeColorId::Choice1,
            ThemeColorId::Choice2,
            ThemeColorId::Choice3,
            ThemeColorId::Choice4,
            ThemeColorId::Choice5,
            ThemeColorId::Choice6,
            ThemeColorId::Choice7,
            ThemeColorId::Choice8,
            ThemeColorId::Choice9,
            ThemeColorId::Choice10,
        ])
    }
}

fn add_core_player_components(
    trigger: Trigger<PlayerCreationInstructions>,
    mut commands: Commands,
    mut availible_colors: ResMut<AvailibleThemeColors>,
) {
    commands
        .entity(trigger.entity)
        .insert((trigger.color, trigger.class));

    // makes sure the color is no longer availible for other players to use.
    for (index, colorid) in availible_colors.0.iter().enumerate() {
        if colorid == &trigger.color {
            availible_colors.0.remove(index);
            break;
        }
    }

    commands.trigger(PlayerCreated(trigger.entity));
}
