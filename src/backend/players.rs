use bevy::prelude::*;

use crate::backend::{AppState, Class, PlayerCount};

pub struct PlayerSpawningPlugin;

impl Plugin for PlayerSpawningPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerCreated>();
        app.add_event::<PlayerCreationInstructions>();
        app.add_event::<SetNextPlayerAsActive>();

        app.add_observer(spawn_player_entities);
        app.add_observer(add_core_player_components);
        app.add_observer(set_next_player_as_active);

        app.init_resource::<AvailibleThemeColors>();

        app.add_systems(OnEnter(AppState::PlayerCreation), set_p1_as_active);
    }
}

#[derive(Resource, Debug)]
pub struct ActivePlayer(pub Entity);

#[derive(Debug, Component, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlayerID(u8);

impl PlayerID {
    pub fn id(&self) -> u8 {
        self.0
    }
}

fn spawn_player_entities(
    trigger: Trigger<PlayerCount>,
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
) {
    for id in 0..*trigger.event() as u8 {
        let entity = commands.spawn(PlayerID(id)).id();
    }
    state.set(AppState::PlayerCreation);
}

#[derive(Debug, Clone, Copy, PartialEq, Event)]
pub struct PlayerCreationInstructions {
    pub entity: Entity,
    pub class: Class,
    pub color: ThemeColorId,
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
pub struct AvailibleThemeColors(pub Vec<ThemeColorId>);

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

#[derive(Debug, Event)]
pub struct SetNextPlayerAsActive;

fn set_next_player_as_active(
    trigger: Trigger<SetNextPlayerAsActive>,
    mut active: ResMut<ActivePlayer>,
    player_ids: Query<&PlayerID>,
    players: Query<(Entity, &PlayerID)>,
) {
    let active_id = player_ids
        .get(active.0)
        .expect("the active player had no id");

    let next_player_id = if let Some(id) = player_ids.iter().filter(|x| x > &active_id).min() {
        id
    } else {
        player_ids.iter().min().unwrap() // we already checked that at least one player exists.
    };

    if next_player_id == active_id {
        error!("The active player was chosen to be the next active player.")
    }

    for (ent, id) in players {
        if id == next_player_id {
            active.0 = ent;
            return;
        }
    }
}

pub fn set_p1_as_active(mut commands: Commands, players: Query<(Entity, &PlayerID)>) {
    for (ent, id) in players.iter() {
        if id.0 == 0 {
            commands.insert_resource(ActivePlayer(ent));
            break;
        }
    }
}
