use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

use crate::backend::{pieces::SpawnLogPiece, tiles::DeleteLogTileRequest};

pub struct GameActionsPlugin;

impl Plugin for GameActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedTiles>();
        app.init_resource::<ActionFunctionality>();

        app.add_systems(
            Update,
            (tmp_execute_action, tmp_change_action_functionality),
        );

        app.add_systems(ExecuteSelectedAction, (send_events, clear_selected).chain());
    }
}

#[derive(Debug, ScheduleLabel, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ExecuteSelectedAction;

#[derive(Debug, Resource, Default)]
enum ActionFunctionality {
    // this is both used as a resource and as a simple data type
    #[default]
    DeleteTile,
    SpawnTower,
}

#[derive(Debug, Resource, Default)]
pub struct SelectedTiles(pub Vec<Entity>);

fn tmp_execute_action(inputs: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if inputs.just_pressed(KeyCode::Space) {
        commands.run_schedule(ExecuteSelectedAction);
    }
}

fn tmp_change_action_functionality(
    inputs: Res<ButtonInput<KeyCode>>,
    mut functionality: ResMut<ActionFunctionality>,
) {
    if inputs.just_pressed(KeyCode::KeyQ) {
        *functionality = ActionFunctionality::DeleteTile;
    } else if inputs.just_pressed(KeyCode::KeyW) {
        *functionality = ActionFunctionality::SpawnTower;
    }
}

fn send_events(
    selected: Res<SelectedTiles>,
    functionality: Res<ActionFunctionality>,
    mut deletions: MessageWriter<DeleteLogTileRequest>,
    mut spawns: MessageWriter<SpawnLogPiece>,
) {
    match *functionality {
        ActionFunctionality::DeleteTile => {
            for log_tile in selected.0.iter() {
                deletions.write(DeleteLogTileRequest(*log_tile));
            }
        }
        ActionFunctionality::SpawnTower => {
            for log_tile in selected.0.iter() {
                spawns.write(SpawnLogPiece {
                    piece_type: super::pieces::BasePieceType::Tower,
                    log_tile: *log_tile,
                });
            }
        }
    }
}

fn clear_selected(mut selected: ResMut<SelectedTiles>) {
    selected.0.clear();
}
