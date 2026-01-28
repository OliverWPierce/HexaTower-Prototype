use bevy::{ecs::schedule::ScheduleLabel, prelude::*};
use rand::{rng, seq::IteratorRandom};

use crate::backend::{BackEndSystems, game_parameters::SetUpBoard};

pub struct PlayersPlugin;

impl Plugin for PlayersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, create_basic_players.in_set(BackEndSystems));
        app.add_systems(Update, tmp_switch_player.in_set(BackEndSystems));
    }
}

#[derive(Debug, Clone)]
pub struct PlayerCreationInstructions {
    pub name: String,
}

#[derive(Debug, Resource, Clone)]
pub struct PlayersToCreate(pub Vec<PlayerCreationInstructions>);

#[derive(Debug, Resource)]
pub struct ActivePlayer(pub Entity);

pub fn create_basic_players(qued_players: Res<PlayersToCreate>, mut commands: Commands) {
    let mut players_created = Vec::new();

    for PlayerCreationInstructions { name } in qued_players.0.iter() {
        let new_player = commands
            .spawn((DisplayName(name.clone()), PlayerMarker))
            .id();

        players_created.push(new_player);
    }

    commands.insert_resource(ActivePlayer(
        *players_created
            .first()
            .expect("There were no players to create"),
    ));

    for (index, player) in players_created.iter().enumerate() {
        commands.entity(*player).insert(PlayerTurnOrder {
            next_player: *players_created.get(index + 1).unwrap_or(
                players_created
                    .first()
                    .expect("There were no players created."),
            ),
        });
    }

    commands.remove_resource::<PlayersToCreate>();
}

#[derive(Debug, Component)]
pub struct DisplayName(String);

#[derive(Debug, Component)]
pub struct PlayerMarker;

#[derive(Debug, Component)]
struct PlayerTurnOrder {
    next_player: Entity,
}

#[derive(Debug, ScheduleLabel, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct StartTurn;

fn tmp_switch_player(
    inputs: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    players: Query<(&PlayerTurnOrder, &DisplayName)>,
    mut active: ResMut<ActivePlayer>,
) {
    if !inputs.just_pressed(KeyCode::KeyS) {
        return;
    }

    let Ok((next, name)) = players.get(active.0) else {
        error!("The entity listed as the current player was not a player.");
        return;
    };

    active.0 = next.next_player;

    println!("Made {} the active player", name.0);

    commands.run_schedule(StartTurn);
}
