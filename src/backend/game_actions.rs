use bevy::prelude::*;

use crate::backend::{pieces::SpawnLogPiece, tiles::DeleteLogTileRequest};

pub struct GameActionsPlugin;

impl Plugin for GameActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentGameActionPreviewed>();
        app.add_message::<DoActionOnTile>();

        app.add_systems(Update, (tmp_swap_previewed_action, execute_action));
    }
}

#[derive(Debug, Resource, Clone, Copy)]
struct CurrentGameActionPreviewed(GameAction);

impl Default for CurrentGameActionPreviewed {
    fn default() -> Self {
        Self(GameAction::SpawnTower)
    }
}

#[derive(Debug, Clone, Copy)]
enum GameAction {
    SpawnTower,
    DeleteTile,
}

#[derive(Debug, Clone, Copy, Message)]
pub struct DoActionOnTile(pub Entity);

fn tmp_swap_previewed_action(
    inputs: Res<ButtonInput<KeyCode>>,
    mut current_action: ResMut<CurrentGameActionPreviewed>,
) {
    if inputs.just_pressed(KeyCode::ArrowLeft) {
        current_action.0 = GameAction::DeleteTile;
    } else if inputs.just_pressed(KeyCode::ArrowRight) {
        current_action.0 = GameAction::SpawnTower;
    }
}

fn execute_action(
    mut tiles_affected: MessageReader<DoActionOnTile>,
    current_action: Res<CurrentGameActionPreviewed>,
    mut deletions: MessageWriter<DeleteLogTileRequest>,
    mut piece_spawns: MessageWriter<SpawnLogPiece>,
) {
    for DoActionOnTile(log_tile) in tiles_affected.read() {
        match current_action.0 {
            GameAction::SpawnTower => {
                piece_spawns.write(SpawnLogPiece {
                    piece_type: crate::backend::pieces::BasePieceType::Tower,
                    log_tile: *log_tile,
                });
            }
            GameAction::DeleteTile => {
                deletions.write(DeleteLogTileRequest(*log_tile));
            }
        }
    }
}
