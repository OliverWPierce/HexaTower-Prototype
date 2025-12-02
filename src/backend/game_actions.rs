use bevy::{
    ecs::{entity, schedule::ScheduleLabel},
    prelude::*,
};

use crate::backend::{
    BackEndUpdateSystems,
    pieces::{OccupiedByPiece, SpawnLogPiece},
    tiles::{AdjacentTiles, DeleteLogTileRequest, LogicalTileLocation},
};

pub struct GameActionsPlugin;

impl Plugin for GameActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentActionFunctionality>();
        app.init_resource::<CurrentSelectionCritera>();

        app.add_systems(
            Update,
            (
                tmp_execute_action,
                tmp_change_action_functionality,
                tmp_change_selection_critera,
                mark_selectable,
            )
                .in_set(BackEndUpdateSystems),
        );

        app.add_systems(ExecuteSelectedAction, (send_events, clear_selected).chain());
    }
}

#[derive(Debug, ScheduleLabel, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ExecuteSelectedAction;

#[derive(Debug, Resource, Default, Clone, Copy, Deref)]
struct CurrentActionFunctionality(pub Option<ActionFunctionality>);

#[derive(Debug, Default, Clone, Copy)]
enum ActionFunctionality {
    // this is both used as a resource and as a simple data type
    #[default]
    DeleteTile,
    SpawnTower,
}

#[derive(Debug, Clone, Copy)]
pub enum CriteraForDeterminingSelectableTiles {
    AllTiles,
    OccupiedTiles,
    UnoccupiedTiles,
}

#[derive(Debug, Clone, Copy, Resource, Deref, Default)]
struct CurrentSelectionCritera(pub Option<CriteraForDeterminingSelectableTiles>);

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Component)]
pub struct Selectable;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Component)]
pub struct Selected;

fn tmp_execute_action(inputs: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if inputs.just_pressed(KeyCode::Space) {
        commands.run_schedule(ExecuteSelectedAction);
    }
}

fn tmp_change_action_functionality(
    inputs: Res<ButtonInput<KeyCode>>,
    mut functionality: ResMut<CurrentActionFunctionality>,
) {
    if inputs.just_pressed(KeyCode::KeyQ) {
        functionality.0 = Some(ActionFunctionality::DeleteTile);
    } else if inputs.just_pressed(KeyCode::KeyW) {
        functionality.0 = Some(ActionFunctionality::SpawnTower);
    }
}

fn tmp_change_selection_critera(
    mut critera: ResMut<CurrentSelectionCritera>,
    inputs: Res<ButtonInput<KeyCode>>,
) {
    if inputs.just_pressed(KeyCode::ArrowLeft) {
        *critera = CurrentSelectionCritera(Some(CriteraForDeterminingSelectableTiles::AllTiles))
    } else if inputs.just_pressed(KeyCode::ArrowRight) {
        *critera =
            CurrentSelectionCritera(Some(CriteraForDeterminingSelectableTiles::OccupiedTiles))
    } else if inputs.just_pressed(KeyCode::ArrowDown) {
        *critera =
            CurrentSelectionCritera(Some(CriteraForDeterminingSelectableTiles::UnoccupiedTiles))
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
    tiles: Query<(Entity, Has<Selected>, Has<OccupiedByPiece>), With<LogicalTileLocation>>,
    mut commands: Commands,
    selection_critera: Res<CurrentSelectionCritera>,
) {
    if selection_critera.is_none() {
    } else {
        match selection_critera.unwrap() {
            CriteraForDeterminingSelectableTiles::AllTiles => {
                for (log_tile, already_selected, _) in tiles {
                    if !already_selected {
                        commands.entity(log_tile).try_insert(Selectable);
                    } else {
                        commands.entity(log_tile).try_remove::<Selectable>();
                    }
                }
            }
            CriteraForDeterminingSelectableTiles::OccupiedTiles => {
                for (log_tile, already_selected, is_occupied) in tiles {
                    if !already_selected && is_occupied {
                        commands.entity(log_tile).try_insert(Selectable);
                    } else {
                        commands.entity(log_tile).try_remove::<Selectable>();
                    }
                }
            }
            CriteraForDeterminingSelectableTiles::UnoccupiedTiles => {
                for (log_tile, already_selected, is_occupied) in tiles {
                    if !already_selected && !is_occupied {
                        commands.entity(log_tile).try_insert(Selectable);
                    } else {
                        commands.entity(log_tile).try_remove::<Selectable>();
                    }
                }
            }
        }
    }
}
