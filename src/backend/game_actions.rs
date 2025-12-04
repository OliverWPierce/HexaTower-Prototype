use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

use crate::backend::{
    BackEndUpdateSystems,
    pieces::{OccupiedByPiece, SpawnLogPiece},
    tiles::{AdjacentTiles, DeleteLogTileRequest},
};

pub struct GameActionsPlugin;

impl Plugin for GameActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentActionFunctionality>();
        app.init_resource::<CurrentEligibilityCritera>();

        app.add_observer(load_action_data_into_resources);

        app.add_systems(
            Update,
            (tmp_execute_action, tmp_load_actions).in_set(BackEndUpdateSystems),
        );

        app.add_systems(ExecuteSelectedAction, send_events);
        app.add_systems(
            ExecuteSelectedAction,
            (clear_action_instructions, clear_eligibility_markers).after(send_events),
        );

        app.add_systems(
            Update,
            (
                clear_eligibility_markers,
                mark_tiles_that_are_eligible_under_method_conditions,
            )
                .chain()
                .in_set(BackEndUpdateSystems)
                .run_if(resource_changed::<CurrentEligibilityCritera>),
        );
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

#[derive(Debug, Component)]
pub struct EligibileTile {
    pub selected: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum MethodForDeterminingEligibility {
    AllTiles,
    AllPieces,
    UnoccupiedTiles,
}

#[derive(Debug, Clone, Copy)]
struct TileEligibilityCritera {
    method: MethodForDeterminingEligibility,
    maximum_amount_of_selected_tiles_allowed: usize,
}

#[derive(Debug, Clone, Copy, Resource, Deref, Default)]
struct CurrentEligibilityCritera(Option<TileEligibilityCritera>);

fn tmp_execute_action(inputs: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if inputs.just_pressed(KeyCode::Space) {
        commands.run_schedule(ExecuteSelectedAction);
    }
}

fn tmp_load_actions(inputs: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if inputs.just_pressed(KeyCode::KeyA) {
        commands.trigger(LoadAction(ActionData {
            functionality: ActionFunctionality::DeleteTile,
            valid_selections: TileEligibilityCritera {
                method: MethodForDeterminingEligibility::UnoccupiedTiles,
                maximum_amount_of_selected_tiles_allowed: 3,
            },
        }));
    } else if inputs.just_pressed(KeyCode::KeyS) {
        commands.trigger(LoadAction(ActionData {
            functionality: ActionFunctionality::SpawnTower,
            valid_selections: TileEligibilityCritera {
                method: MethodForDeterminingEligibility::UnoccupiedTiles,
                maximum_amount_of_selected_tiles_allowed: 1,
            },
        }));
    } else if inputs.just_pressed(KeyCode::KeyD) {
        commands.trigger(LoadAction(ActionData {
            functionality: ActionFunctionality::DeleteTile,
            valid_selections: TileEligibilityCritera {
                method: MethodForDeterminingEligibility::AllPieces,
                maximum_amount_of_selected_tiles_allowed: 2,
            },
        }));
    } else if inputs.just_pressed(KeyCode::KeyF) {
        commands.trigger(LoadAction(ActionData {
            functionality: ActionFunctionality::DeleteTile,
            valid_selections: TileEligibilityCritera {
                method: MethodForDeterminingEligibility::AllTiles,
                maximum_amount_of_selected_tiles_allowed: 5,
            },
        }));
    }
}

fn send_events(
    tiles: Query<(Entity, &EligibileTile)>,
    functionality: Res<CurrentActionFunctionality>,
    mut deletions: MessageWriter<DeleteLogTileRequest>,
    mut spawns: MessageWriter<SpawnLogPiece>,
) {
    let selected = tiles.iter().filter(|(_, status)| status.selected);

    if let Some(functionality) = functionality.0 {
        match functionality {
            ActionFunctionality::DeleteTile => {
                for (log_tile, _) in selected {
                    deletions.write(DeleteLogTileRequest(log_tile));
                }
            }
            ActionFunctionality::SpawnTower => {
                for (log_tile, _) in selected {
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

fn clear_action_instructions(
    mut functionality: ResMut<CurrentActionFunctionality>,
    mut critera: ResMut<CurrentEligibilityCritera>,
) {
    functionality.0 = None;
    critera.0 = None
}

fn clear_eligibility_markers(
    mut commands: Commands,
    eligible_tiles: Query<Entity, With<EligibileTile>>,
) {
    for tile in eligible_tiles {
        commands.entity(tile).remove::<EligibileTile>();
    }
}

fn mark_tiles_that_are_eligible_under_method_conditions(
    log_tiles: Query<(Entity, Has<OccupiedByPiece>, Has<EligibileTile>), With<AdjacentTiles>>,
    critera: Res<CurrentEligibilityCritera>,
    mut commands: Commands,
) {
    if let Some(TileEligibilityCritera {
        method,
        maximum_amount_of_selected_tiles_allowed: _,
    }) = critera.0
    {
        match method {
            MethodForDeterminingEligibility::AllTiles => {
                for (tile, _, is_already_eligible) in log_tiles {
                    // this is an execption to the pattern laid out in the other match arms
                    if !is_already_eligible {
                        commands
                            .entity(tile)
                            .insert(EligibileTile { selected: false });
                    }
                }
            }
            MethodForDeterminingEligibility::AllPieces => {
                for (tile, is_occupied, is_already_eligible) in log_tiles {
                    let eligible = is_occupied; // put the condition logic here.

                    if !eligible {
                        commands.entity(tile).try_remove::<EligibileTile>();
                    } else if !is_already_eligible {
                        commands
                            .entity(tile)
                            .insert(EligibileTile { selected: false });
                    }
                }
            }
            MethodForDeterminingEligibility::UnoccupiedTiles => {
                for (tile, is_occupied, is_already_eligible) in log_tiles {
                    let eligible = !is_occupied;

                    if !eligible {
                        commands.entity(tile).try_remove::<EligibileTile>();
                    } else if !is_already_eligible {
                        commands
                            .entity(tile)
                            .insert(EligibileTile { selected: false });
                    }
                }
            }
        }
    }
}

#[derive(Debug, Component, Clone, Copy)]
struct ActionData {
    functionality: ActionFunctionality,
    valid_selections: TileEligibilityCritera,
}

#[derive(Debug, Event, Clone, Copy)]
struct LoadAction(ActionData);

fn load_action_data_into_resources(
    action_data: On<LoadAction>,
    mut functionality: ResMut<CurrentActionFunctionality>,
    mut selection_critera: ResMut<CurrentEligibilityCritera>,
) {
    functionality.0 = Some(action_data.0.functionality);
    selection_critera.0 = Some(action_data.0.valid_selections);
    println!("changing the loaded action");
}
