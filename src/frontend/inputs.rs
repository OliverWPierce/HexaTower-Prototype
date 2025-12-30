use bevy::prelude::*;

use crate::{
    backend::{
        game_actions::{
            ActionFunctionality, ActionInfo, CurrentAction, EligibilityDeterminationMethod,
            ExecuteSelectedAction, dangerous_selection_mechanics::SelectLogTile,
        },
        pieces::OccupiesTile,
    },
    frontend::{visual_pieces::VisPieceOf, visual_tiles::VisTileOf},
};

pub struct InputsPlugin;

impl Plugin for InputsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(select_tiles);

        app.add_systems(Update, (tmp_send_a_load_actions, tmp_execute_action));
    }
}

fn select_tiles(
    click: On<Pointer<Click>>,
    vis_tiles: Query<&VisTileOf>,
    vis_pieces: Query<&VisPieceOf>,
    log_pieces: Query<&OccupiesTile>,

    mut commands: Commands,
) {
    if let Ok(VisTileOf(log_tile)) = vis_tiles.get(click.entity) {
        commands.trigger(SelectLogTile(*log_tile));
    } else if let Ok(VisPieceOf(log_piece)) = vis_pieces.get(click.entity)
        && let Ok(OccupiesTile { log_tile }) = log_pieces.get(*log_piece)
    {
        commands.trigger(SelectLogTile(*log_tile));
    }
}

fn tmp_send_a_load_actions(mut action: ResMut<CurrentAction>, inputs: Res<ButtonInput<KeyCode>>) {
    if inputs.just_pressed(KeyCode::KeyA) {
        action.0 = Some(ActionInfo::construct(
            ActionFunctionality::DeleteTile,
            EligibilityDeterminationMethod::AllTiles,
            5,
        ));
    } else if inputs.just_pressed(KeyCode::KeyS) {
        action.0 = Some(ActionInfo::construct(
            ActionFunctionality::SpawnTower,
            EligibilityDeterminationMethod::UnoccupiedTiles,
            3,
        ));
    } else if inputs.just_pressed(KeyCode::KeyD) {
        action.0 = Some(ActionInfo::construct(
            ActionFunctionality::DeleteTile,
            EligibilityDeterminationMethod::AllPieces,
            2,
        ));
    } else if inputs.just_pressed(KeyCode::KeyF) {
        action.0 = Some(ActionInfo::construct(
            ActionFunctionality::SpawnTower,
            EligibilityDeterminationMethod::UnoccupiedTiles,
            20,
        ));
    }
}

fn tmp_execute_action(inputs: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if inputs.just_pressed(KeyCode::Space) {
        commands.run_schedule(ExecuteSelectedAction);
    }
}
