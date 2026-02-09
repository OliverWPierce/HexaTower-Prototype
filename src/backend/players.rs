use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

use crate::backend::{BackEndSystems, game_parameters::SetUpBoard};

pub struct PlayersPlugin;

impl Plugin for PlayersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, create_basic_players.in_set(BackEndSystems));

        app.add_observer(switch_player);

        //TMP systems!!!
        app.add_systems(Update, tmp_update_and_check_start_delay);
        app.add_systems(SetUpBoard, tmp_add_start_turn_delay_timer);
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
pub struct DisplayName(pub String);

#[derive(Debug, Component)]
pub struct PlayerMarker;

#[derive(Debug, Component)]
struct PlayerTurnOrder {
    next_player: Entity,
}

#[derive(Debug, ScheduleLabel, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct StartTurn;

#[derive(Debug, Event)]
pub struct SwitchPlayerRequest;

fn switch_player(
    _request: On<SwitchPlayerRequest>,
    mut commands: Commands,
    players: Query<(&PlayerTurnOrder)>,
    mut active: ResMut<ActivePlayer>,
) {
    let Ok(next) = players.get(active.0) else {
        error!("The entity listed as the current player was not a player.");
        return;
    };

    active.0 = next.next_player;

    commands.run_schedule(StartTurn);
}

/// The purpose of this is to wait for all background assets to load before starting the first turn. Later, it should be replaced with an actual system for tracking loaded assets.
#[derive(Debug, Resource)]
struct TmpTimerForFirstTurn(Timer);

fn tmp_add_start_turn_delay_timer(mut commands: Commands) {
    commands.insert_resource(TmpTimerForFirstTurn(Timer::from_seconds(
        5.0,
        TimerMode::Once,
    )));
}

fn tmp_update_and_check_start_delay(
    mut commands: Commands,
    mut timer: If<ResMut<TmpTimerForFirstTurn>>,
    time: Res<Time>,
) {
    timer.0.0.tick(time.delta());
    if timer.0.0.is_finished() {
        commands.remove_resource::<TmpTimerForFirstTurn>();
        commands.run_schedule(StartTurn);
    }
}
