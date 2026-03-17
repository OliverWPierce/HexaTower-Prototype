use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

use crate::backend::{
    BackEndSystems,
    game_actions::ClearBackendData,
    game_parameters::SetUpBoard,
    pieces::{
        CommandPoint, LogPieceOwnedByPlayer, OrdersPerTurn, OwnsLogPieces, PieceForSale,
        TransferPieceOwnership, WinCondition,
    },
};

pub struct PlayersPlugin;

impl Plugin for PlayersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            SetUpBoard,
            (create_basic_players, create_ghost_player).in_set(BackEndSystems),
        );

        app.add_observer(switch_player);
        app.add_observer(check_if_player_dead_or_game_over);
        app.add_message::<CreatedLogPlayer>();

        app.add_systems(
            SetUpBoard,
            remove_resource_with_player_creation_instructions.in_set(ClearBackendData),
        );

        app.add_systems(StartTurn, calculate_player_orders_this_turn);
        app.add_observer(sell_pieces);

        //TMP systems!!!
        app.add_systems(Update, tmp_update_and_check_start_delay);
        app.add_systems(SetUpBoard, tmp_add_start_turn_delay_timer);
    }
}

#[derive(Debug, Clone)]
pub struct PlayerCreationInstructions {
    pub name: String,
    pub base_pate_path: String,
}

#[derive(Debug, Resource, Clone)]
pub struct PlayersToCreate(pub Vec<PlayerCreationInstructions>);

#[derive(Debug, Resource)]
pub struct ActivePlayer(pub Entity);

#[derive(Debug, Message, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct CreatedLogPlayer(pub Entity, pub usize);

#[derive(Debug, Component)]
pub struct PlayerOrdersRemaining(pub u32);

pub fn create_basic_players(
    qued_players: Res<PlayersToCreate>,
    mut commands: Commands,
    mut created_players: MessageWriter<CreatedLogPlayer>,
) {
    let mut players_created = Vec::new();

    for (index, _) in qued_players.0.iter().enumerate() {
        let new_player = commands
            .spawn((
                PlayerMarker,
                PlayerState::HasNoTowerYet,
                PlayerOrdersRemaining(0),
            ))
            .id();
        created_players.write(CreatedLogPlayer(new_player, index));

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
}

#[derive(Debug, Component)]
pub struct GhostPlayer;

fn create_ghost_player(mut commands: Commands) {
    commands.spawn((PlayerMarker, PlayerState::Dead, GhostPlayer));
}

fn remove_resource_with_player_creation_instructions(mut commands: Commands) {
    commands.remove_resource::<PlayersToCreate>();
}

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
    players: Query<(&PlayerTurnOrder, &PlayerState)>,
    mut active: ResMut<ActivePlayer>,
) -> Result<(), BevyError> {
    let (
        PlayerTurnOrder {
            next_player: ideal_next_player,
        },
        current_player_life_state,
    ) = players.get(active.0)?;

    if *current_player_life_state == PlayerState::HasNoTowerYet {
        return Ok(());
    }

    let next_player = {
        let mut current_candidate = *ideal_next_player;

        loop {
            let (
                PlayerTurnOrder {
                    next_player: next_candidate,
                },
                life_state,
            ) = players.get(current_candidate)?;

            if *life_state != PlayerState::Dead {
                break current_candidate;
            } else {
                current_candidate = *next_candidate;
            }
        }
    };

    if next_player == active.0 {
        commands.trigger(CheckForWinner);
    }

    active.0 = next_player;

    commands.run_system_cached(replenish_piece_orders);

    commands.run_schedule(StartTurn);

    Ok(())
}

fn replenish_piece_orders(
    active_player: Res<ActivePlayer>,
    players: Query<&OwnsLogPieces>,
    mut pieces: Query<&mut OrdersPerTurn>,
) -> Result<(), BevyError> {
    let Ok(owned_pieces) = players.get(active_player.0) else {
        warn!("failed get player owned pieces");
        return Ok(());
    };

    for piece in owned_pieces.list() {
        let Ok(mut order_stats) = pieces.get_mut(*piece) else {
            warn!("failed get piece");
            return Ok(());
        };
        order_stats.current = order_stats.max;
    }

    Ok(())
}

fn calculate_player_orders_this_turn(
    active_player: Res<ActivePlayer>,
    pieces: Query<&LogPieceOwnedByPlayer, With<CommandPoint>>,
    mut orders_this_turn: Query<&mut PlayerOrdersRemaining>,
) -> Result<(), BevyError> {
    let extra_commands = pieces
        .iter()
        .filter(|LogPieceOwnedByPlayer(owner)| *owner == active_player.0)
        .count();

    orders_this_turn.get_mut(active_player.0)?.0 = 2 + extra_commands as u32;

    Ok(())
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
#[derive(Debug, Component, PartialEq, Eq)]
pub enum PlayerState {
    HasNoTowerYet,
    Alive,
    Dead,
}
#[derive(Debug, Event)]
pub struct CheckForWinner;

fn check_if_player_dead_or_game_over(
    _trigger: On<CheckForWinner>,
    active_player: Res<ActivePlayer>,
    players: Query<(Entity, &mut PlayerState, &OwnsLogPieces)>,
    living_win_conditions: Query<&LogPieceOwnedByPlayer, With<WinCondition>>,
    mut commands: Commands,
) {
    let mut living_players: u8 = 0;

    for (player_ent, mut player_state, owned_pieces) in players {
        if *player_state == PlayerState::HasNoTowerYet
            || owned_pieces
                .list()
                .iter()
                .any(|piece| living_win_conditions.get(*piece).is_ok())
        {
            living_players += 1;
        } else if *player_state != PlayerState::Dead {
            *player_state = PlayerState::Dead;
            commands.trigger(PlayerDied(player_ent));

            if active_player.0 == player_ent {
                commands.trigger(SwitchPlayerRequest);
            }
        }
    }

    if living_players > 1 {
        return;
    }

    commands.trigger(GameOver {
        winner: (living_players == 1).then(|| living_win_conditions.iter().next().unwrap().0),
    });
}

#[derive(Debug, Event)]
pub struct GameOver {
    pub winner: Option<Entity>,
}
#[derive(Debug, Event)]
struct PlayerDied(Entity);

fn sell_pieces(
    dead_player: On<PlayerDied>,
    owned_pieces: Query<&OwnsLogPieces>,
    ghost_player: Single<Entity, With<GhostPlayer>>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let ghost_ent = ghost_player.entity();

    for piece in owned_pieces.get(dead_player.0)?.list() {
        commands.entity(*piece).insert(PieceForSale);
        commands.trigger(TransferPieceOwnership {
            piece: *piece,
            to_player: ghost_ent,
        });
    }

    Ok(())
}
