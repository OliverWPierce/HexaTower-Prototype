use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

use crate::backend::{
    BackEndSystems,
    game_actions::dangerous_selection_mechanics::{SelectedLogTiles, TileSelectionStatus},
    game_parameters::SetUpBoard,
    pieces::{OccupiedByPiece, SpawnLogPiece},
    tiles::{
        AdjacentTiles, DeleteLogTileRequest, EssentialTileCreationSystems, LogicalTileCreated,
    },
};

pub struct GameActionsPlugin;

impl Plugin for GameActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentAction>();
        app.init_resource::<SelectedLogTiles>();
        app.add_systems(
            Update,
            dangerous_selection_mechanics::handle_action_change
                .run_if(resource_changed::<CurrentAction>)
                .in_set(BackEndSystems),
        );
        app.add_observer(dangerous_selection_mechanics::select_tile);

        app.add_systems(ExecuteSelectedAction, execute_action.in_set(BackEndSystems));

        app.add_systems(
            ActionOrSelectionChanged,
            evaluate_tiles.in_set(BackEndSystems),
        );
        app.add_systems(
            SetUpBoard,
            insert_selection_data
                .after(EssentialTileCreationSystems)
                .in_set(BackEndSystems),
        );

        app.add_systems(Update, (insert_selection_data,).in_set(BackEndSystems));

        app.add_observer(validate_execution_request);
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect)]
pub enum ActionFunctionality {
    DeleteTile,
    SpawnTower,
    DoubleTakeTest,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect)]
pub enum EligibilityDeterminationMethod {
    AllTiles,
    AllPieces,
    UnoccupiedTiles,
    PieceChain,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect)]
pub struct ActionInfo {
    functionality: ActionFunctionality,
    eligibility_method: EligibilityDeterminationMethod,
    maximum_selected_tiles: usize,
}

impl ActionInfo {
    pub fn construct(
        function: ActionFunctionality,
        method: EligibilityDeterminationMethod,
        maximum_tiles: usize,
    ) -> Self {
        if maximum_tiles <= function.bounds().max_tiles
            && maximum_tiles >= function.bounds().min_tiles
            && maximum_tiles <= method.bounds().max_tiles
            && maximum_tiles >= method.bounds().min_tiles
        {
            ActionInfo {
                functionality: function,
                eligibility_method: method,
                maximum_selected_tiles: maximum_tiles,
            }
        } else {
            panic!(
                "Developer error. Attempted to create an action with a maximum number of tiles that contradicted the code's capabilities."
            )
        }
    }
    pub fn read_min_and_max_tiles(&self) -> (usize, usize) {
        let mins = (
            self.functionality.bounds().min_tiles,
            self.eligibility_method.bounds().min_tiles,
        );

        if mins.0 >= mins.1 {
            (mins.0, self.maximum_selected_tiles)
        } else {
            (mins.1, self.maximum_selected_tiles)
        }
    }
}

impl ActionFunctionality {
    fn bounds(&self) -> FunctionalTileCountBounds {
        match self {
            ActionFunctionality::DeleteTile => FunctionalTileCountBounds::new(1, usize::MAX),
            ActionFunctionality::SpawnTower => FunctionalTileCountBounds::new(1, usize::MAX),
            ActionFunctionality::DoubleTakeTest => FunctionalTileCountBounds {
                min_tiles: 2,
                max_tiles: 2,
            },
        }
    }
}

impl EligibilityDeterminationMethod {
    fn bounds(&self) -> FunctionalTileCountBounds {
        match &self {
            EligibilityDeterminationMethod::AllTiles => {
                FunctionalTileCountBounds::new(2, usize::MAX)
            }
            EligibilityDeterminationMethod::AllPieces => {
                FunctionalTileCountBounds::new(1, usize::MAX)
            }
            EligibilityDeterminationMethod::UnoccupiedTiles => {
                FunctionalTileCountBounds::new(1, usize::MAX)
            }
            EligibilityDeterminationMethod::PieceChain => {
                FunctionalTileCountBounds::new(1, usize::MAX)
            }
        }
    }
}

struct FunctionalTileCountBounds {
    min_tiles: usize,
    max_tiles: usize,
}

impl FunctionalTileCountBounds {
    fn new(min: usize, max: usize) -> Self {
        FunctionalTileCountBounds {
            max_tiles: max,
            min_tiles: min,
        }
    }
}

#[derive(Debug, Resource, PartialEq, Eq, Default)]
pub struct CurrentAction(pub Option<ActionInfo>);

#[derive(Debug, ScheduleLabel, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExecuteSelectedAction;

fn insert_selection_data(mut new_tiles: MessageReader<LogicalTileCreated>, mut commands: Commands) {
    for LogicalTileCreated(tile) in new_tiles.read() {
        commands.entity(*tile).insert(TileSelectionStatus::new());
    }
}

fn execute_action(
    mut action: ResMut<CurrentAction>,
    selected_tiles: Res<SelectedLogTiles>,
    mut deletions: MessageWriter<DeleteLogTileRequest>,
    mut piece_spawns: MessageWriter<SpawnLogPiece>,
) {
    let Some(ActionInfo { functionality, .. }) = action.0 else {
        return;
    };

    match functionality {
        ActionFunctionality::DeleteTile => {
            for log_tile in selected_tiles.as_read_only_list() {
                deletions.write(DeleteLogTileRequest(*log_tile));
            }
        }
        ActionFunctionality::SpawnTower => {
            for log_tile in selected_tiles.as_read_only_list() {
                piece_spawns.write(SpawnLogPiece {
                    piece_type: super::pieces::BasePieceType::Tower,
                    log_tile: *log_tile,
                });
            }
        }
        ActionFunctionality::DoubleTakeTest => {
            deletions.write(DeleteLogTileRequest(
                *(selected_tiles.as_read_only_list().get(1).unwrap()),
            ));
            piece_spawns.write(SpawnLogPiece {
                piece_type: super::pieces::BasePieceType::Tower,
                log_tile: *selected_tiles.as_read_only_list().first().unwrap(),
            });
        }
    }
    action.0 = None;
}

fn evaluate_tiles(
    action: Res<CurrentAction>,
    mut tiles: Query<(
        &mut TileSelectionStatus,
        Has<OccupiedByPiece>,
        &AdjacentTiles,
    )>,
    selected_tiles: Res<SelectedLogTiles>,
) {
    let Some(ActionInfo {
        eligibility_method,
        maximum_selected_tiles,
        ..
    }) = action.0
    else {
        return;
    };

    if selected_tiles.as_read_only_list().len() >= maximum_selected_tiles {
        for (mut selection_state, _, _) in tiles.iter_mut() {
            selection_state.try_make_ineligble();
        }
        return;
    }

    match eligibility_method {
        EligibilityDeterminationMethod::AllTiles => {
            for (mut selection_state, _, _) in tiles.iter_mut() {
                selection_state.try_make_eligible();
            }
        }
        EligibilityDeterminationMethod::AllPieces => {
            for (mut selection_state, is_occupied, _) in tiles.iter_mut() {
                if is_occupied {
                    selection_state.try_make_eligible();
                } else {
                    selection_state.try_make_ineligble();
                }
            }
        }
        EligibilityDeterminationMethod::UnoccupiedTiles => {
            for (mut selection_state, is_occupied, _) in tiles.iter_mut() {
                if !is_occupied {
                    selection_state.try_make_eligible();
                } else {
                    selection_state.try_make_ineligble();
                }
            }
        }
        EligibilityDeterminationMethod::PieceChain => {
            if selected_tiles.as_read_only_list().is_empty() {
                for (mut selection_state, is_occupied, _) in tiles.iter_mut() {
                    if is_occupied {
                        selection_state.try_make_eligible();
                    } else {
                        selection_state.try_make_ineligble();
                    }
                }
            } else {
                for (mut selection_state, _, _) in tiles.iter_mut() {
                    selection_state.try_make_ineligble();
                }

                for log_tile in selected_tiles.as_read_only_list() {
                    let Ok((_, _, adjacents)) = tiles.get(*log_tile) else {
                        error!("A tile had no adjacent tiles component");
                        continue;
                    };

                    for possible_tile in adjacents.0 {
                        let Some(adjacent_tile) = possible_tile else {
                            continue;
                        };

                        let Ok((mut selection_state, is_occupied, _)) =
                            tiles.get_mut(adjacent_tile)
                        else {
                            error!("An entity listed as adjacent to a tile was not a tile");
                            continue;
                        };
                        if is_occupied {
                            selection_state.try_make_eligible();
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, ScheduleLabel, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActionOrSelectionChanged;

pub mod dangerous_selection_mechanics {
    use crate::backend::game_actions::ActionOrSelectionChanged;
    use bevy::prelude::*;

    // The idea of this module is to privatize the ability to mutate selection data so that future me doesn't break stuff.
    // Essentially, it ensures that once a tile is selected, it cannot be deselected unless the selection process resets. Also,
    // the only way to select a tile is through a special event.

    #[derive(Debug, Default, Resource)]
    pub struct SelectedLogTiles(Vec<Entity>);

    impl SelectedLogTiles {
        pub fn as_read_only_list(&self) -> &Vec<Entity> {
            &self.0
        }
    }
    #[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
    pub enum SelectionState {
        Selected,
        Eligible,
        #[default]
        Neither,
    }

    #[derive(Debug, Component)]
    pub struct TileSelectionStatus(SelectionState);

    impl TileSelectionStatus {
        /// Unless the tile is already selected, this function will change a tile to "neither" eligible or selected.
        /// It is used to prevent accidental alterations to the data of selected tiles.
        pub fn try_make_ineligble(&mut self) {
            if self.0 != SelectionState::Selected {
                self.0 = SelectionState::Neither
            }
        }

        /// This function will make a tile eligible to be selected next unless it is already selected.
        /// It is used to prevent accidental alterations to the data of selected tiles.
        pub fn try_make_eligible(&mut self) {
            if self.0 != SelectionState::Selected {
                self.0 = SelectionState::Eligible
            }
        }

        pub fn read(&self) -> SelectionState {
            self.0
        }

        pub fn new() -> Self {
            TileSelectionStatus(SelectionState::default())
        }
    }

    pub fn handle_action_change(
        mut selection_list: ResMut<SelectedLogTiles>,
        mut log_tiles: Query<&mut TileSelectionStatus>,
        mut commands: Commands,
    ) {
        selection_list.0.clear();
        for mut tile_state in log_tiles.iter_mut() {
            tile_state.0 = SelectionState::Neither;
        }
        commands.run_schedule(ActionOrSelectionChanged);
    }

    #[derive(Debug, Event)]
    pub struct SelectLogTile(pub Entity);

    pub fn select_tile(
        tile: On<SelectLogTile>,
        mut selection_list: ResMut<SelectedLogTiles>,
        mut selection_data: Query<&mut TileSelectionStatus>,
        mut commands: Commands,
    ) {
        let Ok(mut status) = selection_data.get_mut(tile.0) else {
            warn!("A tile selection request was made for an entity with no tile selection status");
            return;
        };

        if status.0 == SelectionState::Eligible {
            status.0 = SelectionState::Selected;
            selection_list.0.push(tile.0);
            commands.run_schedule(ActionOrSelectionChanged);
        }
    }
}
#[derive(Debug, Event)]
pub struct ExecuteActionRequest;

fn validate_execution_request(
    request: On<ExecuteActionRequest>,
    mut commands: Commands,
    action: Res<CurrentAction>,
    selected_tiles: Res<SelectedLogTiles>,
) {
    let Some(action_info) = action.0 else { return };

    let amount_selected = selected_tiles.as_read_only_list().len();

    if amount_selected >= action_info.functionality.bounds().min_tiles
        && amount_selected <= action_info.functionality.bounds().max_tiles
    {
        commands.run_schedule(ExecuteSelectedAction);
    }
}
