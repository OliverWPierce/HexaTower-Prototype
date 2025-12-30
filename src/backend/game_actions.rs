use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

use crate::backend::{
    BackEndSystems,
    pieces::{OccupiedByPiece, SpawnLogPiece},
    tiles::{DeleteLogTileRequest, LogicalTileLocation},
};

pub struct GameActionsPlugin;

impl Plugin for GameActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentAction>();
        app.add_systems(
            Update,
            handle_action_change
                .run_if(resource_changed::<CurrentAction>)
                .in_set(BackEndSystems),
        );
        app.add_observer(select_tile);

        app.add_systems(ExecuteSelectedAction, execute_action.in_set(BackEndSystems));

        app.add_systems(
            ActionOrSelectionChanged,
            evaluate_tiles.in_set(BackEndSystems),
        );

        app.add_systems(
            Update,
            (tmp_send_a_load_actions, tmp_execute_action).in_set(BackEndSystems),
        );
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ActionFunctionality {
    DeleteTile,
    SpawnTower,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EligibilityDeterminationMethod {
    AllTiles,
    AllPieces,
    UnoccupiedTiles,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ActionInfo {
    pub functionality: ActionFunctionality,
    pub eligibility_method: EligibilityDeterminationMethod,
    pub maximum_selected_tiles: usize,
}

#[derive(Debug, Resource, PartialEq, Eq, Default)]
pub struct CurrentAction(pub Option<ActionInfo>);

#[derive(Debug, ScheduleLabel, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExecuteSelectedAction;

#[derive(Debug, Component)]
pub struct LogicallySelected;

// selected tiles are not eligible for the next selection
#[derive(Debug, Component)]
pub struct EligibileForNextSelection;

fn execute_action(
    mut action: ResMut<CurrentAction>,
    selected_tiles: Query<Entity, With<LogicallySelected>>,
    mut deletions: MessageWriter<DeleteLogTileRequest>,
    mut piece_spawns: MessageWriter<SpawnLogPiece>,
) {
    if let Some(action_info) = &action.0 {
        // do the action
        match action_info.functionality {
            ActionFunctionality::DeleteTile => {
                for log_tile in selected_tiles {
                    deletions.write(DeleteLogTileRequest(log_tile));
                }
            }
            ActionFunctionality::SpawnTower => {
                for log_tile in selected_tiles {
                    piece_spawns.write(SpawnLogPiece {
                        piece_type: super::pieces::BasePieceType::Tower,
                        log_tile,
                    });
                }
            }
        }
        // clean up
        *action = CurrentAction(None);
    }
}

#[derive(Debug, ScheduleLabel, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActionOrSelectionChanged;

fn evaluate_tiles(
    tile_info: Query<(Entity, Has<OccupiedByPiece>), With<LogicalTileLocation>>,
    selected_tiles: Query<Entity, With<LogicallySelected>>,
    action: Res<CurrentAction>,
    mut commands: Commands,
) {
    let Some(action_info) = &action.0 else { return };
    if selected_tiles.count() >= action_info.maximum_selected_tiles {
        for (tile, _) in tile_info {
            if !selected_tiles.contains(tile) {
                commands.entity(tile).remove::<EligibileForNextSelection>();
            }
        }
        return;
    }

    // Note: each implementation of the method must ensure that nothing is selected and eligible at the same time.
    match action_info.eligibility_method {
        EligibilityDeterminationMethod::AllTiles => {
            for (log_tile, _) in tile_info {
                if selected_tiles.contains(log_tile) {
                    commands
                        .entity(log_tile)
                        .remove::<EligibileForNextSelection>();
                    continue;
                }
                commands.entity(log_tile).insert(EligibileForNextSelection);
            }
        }
        EligibilityDeterminationMethod::AllPieces => {
            for (log_tile, is_occupied) in tile_info {
                if selected_tiles.contains(log_tile) {
                    commands
                        .entity(log_tile)
                        .remove::<EligibileForNextSelection>();
                    continue;
                }

                if is_occupied {
                    commands.entity(log_tile).insert(EligibileForNextSelection);
                } else {
                    commands
                        .entity(log_tile)
                        .remove::<EligibileForNextSelection>();
                }
            }
        }
        EligibilityDeterminationMethod::UnoccupiedTiles => {
            for (log_tile, is_occupied) in tile_info {
                if selected_tiles.contains(log_tile) {
                    commands
                        .entity(log_tile)
                        .remove::<EligibileForNextSelection>();
                    continue;
                }
                if !is_occupied {
                    commands.entity(log_tile).insert(EligibileForNextSelection);
                } else {
                    commands
                        .entity(log_tile)
                        .remove::<EligibileForNextSelection>();
                }
            }
        }
    }
}

fn tmp_send_a_load_actions(mut action: ResMut<CurrentAction>, inputs: Res<ButtonInput<KeyCode>>) {
    if inputs.just_pressed(KeyCode::KeyA) {
        action.0 = Some(ActionInfo {
            functionality: ActionFunctionality::DeleteTile,
            eligibility_method: EligibilityDeterminationMethod::AllTiles,
            maximum_selected_tiles: 5,
        });
    } else if inputs.just_pressed(KeyCode::KeyS) {
        action.0 = Some(ActionInfo {
            functionality: ActionFunctionality::SpawnTower,
            eligibility_method: EligibilityDeterminationMethod::UnoccupiedTiles,
            maximum_selected_tiles: 3,
        });
    } else if inputs.just_pressed(KeyCode::KeyD) {
        action.0 = Some(ActionInfo {
            functionality: ActionFunctionality::DeleteTile,
            eligibility_method: EligibilityDeterminationMethod::AllPieces,
            maximum_selected_tiles: 1,
        });
    } else if inputs.just_pressed(KeyCode::KeyF) {
        action.0 = Some(ActionInfo {
            functionality: ActionFunctionality::DeleteTile,
            eligibility_method: EligibilityDeterminationMethod::UnoccupiedTiles,
            maximum_selected_tiles: 6,
        });
    }
}

fn handle_action_change(
    mut commands: Commands,
    all_tiles: Query<Entity, With<LogicalTileLocation>>,
) {
    for tile in all_tiles {
        commands
            .entity(tile)
            .remove::<(LogicallySelected, EligibileForNextSelection)>();
    }
    commands.run_schedule(ActionOrSelectionChanged);
}

#[derive(Debug, Event)]
pub struct SelectLogTile(pub Entity);

fn select_tile(
    tile: On<SelectLogTile>,
    eligible_for_selection: Query<(), With<EligibileForNextSelection>>,

    mut commands: Commands,
) {
    if eligible_for_selection.contains(tile.0) {
        commands.entity(tile.0).insert(LogicallySelected);
        commands.run_schedule(ActionOrSelectionChanged);
    }
}

fn tmp_execute_action(inputs: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if inputs.just_pressed(KeyCode::Space) {
        commands.run_schedule(ExecuteSelectedAction);
    }
}
