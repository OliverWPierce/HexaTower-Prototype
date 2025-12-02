use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

use crate::backend::{
    BackEndUpdateSystems,
    pieces::{OccupiedByPiece, SpawnLogPiece},
    tiles::{DeleteLogTileRequest, LogicalTileLocation},
};

pub struct GameActionsPlugin;

impl Plugin for GameActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentActionFunctionality>();
        app.init_resource::<CurrentCriteraForSelectability>();

        app.add_observer(load_action_data_into_resources);

        app.add_systems(
            Update,
            (tmp_execute_action, tmp_load_actions, mark_selectable).in_set(BackEndUpdateSystems),
        );

        app.add_systems(ExecuteSelectedAction, (send_events, clear_selected).chain());
    }
}

#[derive(Debug, ScheduleLabel, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ExecuteSelectedAction;

#[derive(Debug, Resource, Default, Clone, Copy, Deref)]
struct CurrentActionFunctionality(pub Option<ActionFunctionality>);

#[derive(Debug, Clone, Copy)]
enum ActionFunctionality {
    DeleteTile,
    SpawnTower,
}

#[derive(Debug, Clone, Copy)]
pub enum MethodForDeterminingSelectability {
    AllTiles,
    AllPieces,
    UnoccupiedTiles,
}

#[derive(Debug, Clone, Copy)]
struct CriteraForTileToBeSelectable {
    method: MethodForDeterminingSelectability,
    max_tiles_selected: usize,
}

#[derive(Debug, Clone, Copy, Resource, Deref, Default)]
struct CurrentCriteraForSelectability(Option<CriteraForTileToBeSelectable>);

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Component)]
pub struct Selectable;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Component)]
pub struct Selected;

fn tmp_execute_action(inputs: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if inputs.just_pressed(KeyCode::Space) {
        commands.run_schedule(ExecuteSelectedAction);
    }
}

fn tmp_load_actions(inputs: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if inputs.just_pressed(KeyCode::KeyA) {
        commands.trigger(LoadAction(ActionData {
            functionality: ActionFunctionality::DeleteTile,
            valid_selections: CriteraForTileToBeSelectable {
                method: MethodForDeterminingSelectability::UnoccupiedTiles,
                max_tiles_selected: 3,
            },
        }));
    } else if inputs.just_pressed(KeyCode::KeyS) {
        commands.trigger(LoadAction(ActionData {
            functionality: ActionFunctionality::SpawnTower,
            valid_selections: CriteraForTileToBeSelectable {
                method: MethodForDeterminingSelectability::UnoccupiedTiles,
                max_tiles_selected: 1,
            },
        }));
    } else if inputs.just_pressed(KeyCode::KeyD) {
        commands.trigger(LoadAction(ActionData {
            functionality: ActionFunctionality::DeleteTile,
            valid_selections: CriteraForTileToBeSelectable {
                method: MethodForDeterminingSelectability::AllPieces,
                max_tiles_selected: 2,
            },
        }));
    } else if inputs.just_pressed(KeyCode::KeyF) {
        commands.trigger(LoadAction(ActionData {
            functionality: ActionFunctionality::DeleteTile,
            valid_selections: CriteraForTileToBeSelectable {
                method: MethodForDeterminingSelectability::AllTiles,
                max_tiles_selected: 5,
            },
        }));
    }
}

fn send_events(
    selected: Query<Entity, With<Selected>>,
    functionality: Res<CurrentActionFunctionality>,
    mut deletions: MessageWriter<DeleteLogTileRequest>,
    mut spawns: MessageWriter<SpawnLogPiece>,
) {
    if let Some(functionality) = functionality.0 {
        match functionality {
            ActionFunctionality::DeleteTile => {
                for log_tile in selected.iter() {
                    deletions.write(DeleteLogTileRequest(log_tile));
                }
            }
            ActionFunctionality::SpawnTower => {
                for log_tile in selected.iter() {
                    spawns.write(SpawnLogPiece {
                        piece_type: super::pieces::BasePieceType::Tower,
                        log_tile,
                    });
                }
            }
        }
    } else {
        warn!(
            "The system responsible for sending events after a game action is taken was triggered when the current game action was NONE"
        )
    }
}

fn clear_selected(selected: Query<Entity, With<Selected>>, mut commands: Commands) {
    for log_tile in selected {
        commands.entity(log_tile).remove::<Selected>();
    }
}

fn mark_selectable(
    tiles: Query<(Entity, Has<OccupiedByPiece>), With<LogicalTileLocation>>,
    mut commands: Commands,
    selection_critera: Res<CurrentCriteraForSelectability>,
    selected: Query<&Selected>,
) {
    if selection_critera.is_some_and(|crit| crit.max_tiles_selected > selected.count()) {
        match selection_critera.unwrap().method {
            MethodForDeterminingSelectability::AllTiles => {
                for (log_tile, _) in tiles {
                    if !selected.contains(log_tile) {
                        commands.entity(log_tile).try_insert(Selectable);
                    } else {
                        commands.entity(log_tile).try_remove::<Selectable>();
                    }
                }
            }
            MethodForDeterminingSelectability::AllPieces => {
                for (log_tile, is_occupied) in tiles {
                    if !selected.contains(log_tile) && is_occupied {
                        commands.entity(log_tile).try_insert(Selectable);
                    } else {
                        commands.entity(log_tile).try_remove::<Selectable>();
                    }
                }
            }
            MethodForDeterminingSelectability::UnoccupiedTiles => {
                for (log_tile, is_occupied) in tiles {
                    if !selected.contains(log_tile) && !is_occupied {
                        commands.entity(log_tile).try_insert(Selectable);
                    } else {
                        commands.entity(log_tile).try_remove::<Selectable>();
                    }
                }
            }
        }
    }
}

#[derive(Debug, Component, Clone, Copy)]
struct ActionData {
    functionality: ActionFunctionality,
    valid_selections: CriteraForTileToBeSelectable,
}

#[derive(Debug, Event, Clone, Copy)]
struct LoadAction(ActionData);

fn load_action_data_into_resources(
    action_data: On<LoadAction>,
    mut functionality: ResMut<CurrentActionFunctionality>,
    mut selection_critera: ResMut<CurrentCriteraForSelectability>,
) {
    functionality.0 = Some(action_data.0.functionality);
    selection_critera.0 = Some(action_data.0.valid_selections);
    println!("changing the loaded action")
}
