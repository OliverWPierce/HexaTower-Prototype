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
    commands.remove_resource::<PlayersToCreate>();
    commands.insert_resource(ActivePlayer(
        *players_created
            .first()
            .expect("There were no players to create"),
    ));
}

#[derive(Debug, Component)]
pub struct DisplayName(String);

#[derive(Debug, Component)]
pub struct PlayerMarker;

#[derive(Debug, ScheduleLabel, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct StartTurn;

fn tmp_switch_player(
    inputs: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    players: Query<(Entity, &DisplayName)>,
    mut active: ResMut<ActivePlayer>,
) {
    if !inputs.just_pressed(KeyCode::KeyS) {
        return;
    }

    let mut rng = rng();
    let player = players.iter().choose(&mut rng).expect("No players existed");

    active.0 = player.0;

    println!("Made {} the active player", player.1.0);

    commands.run_schedule(StartTurn);
}
