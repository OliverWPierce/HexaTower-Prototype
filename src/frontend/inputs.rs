use bevy::prelude::*;
use rand::seq::IteratorRandom;

use crate::{
    backend::{
        game_actions::{IsEligible, SelectionRequest},
        pieces::OccupiesTile,
    },
    frontend::{
        tile_selection_indicators::Indicator, visual_pieces::VisPieceOf, visual_tiles::VisTileOf,
    },
};

pub struct InputsPlugin;

impl Plugin for InputsPlugin {
    fn build(&self, app: &mut App) {
        // app.add_observer(select_tiles);

        app.add_systems(Update, click_random_tile);
    }
}

// fn select_tiles(
//     click: On<Pointer<Click>>,
//     vis_tiles: Query<&VisTileOf>,
//     visual_pieces: Query<&VisPieceOf>,
//     logical_piece_occupies: Query<&OccupiesTile>,
//     indicators: Query<&Indicator>,
//     mut commands: Commands,
// ) {
//     if let Ok(VisTileOf(log_tile)) = vis_tiles.get(click.entity) {
//         commands.trigger(SelectionRequest(*log_tile));
//     } else if let Ok(VisPieceOf(log_piece)) = visual_pieces.get(click.entity) {
//         let log_tile = logical_piece_occupies
//             .get(*log_piece)
//             .expect("A logical piece occupied no tile.")
//             .log_tile;
//         commands.trigger(SelectionRequest(log_tile));
//     } else if let Ok(indicator) = indicators.get(click.entity) {
//         commands.trigger(SelectionRequest(indicator.watches));
//     }
// }

fn click_random_tile(
    inputs: Res<ButtonInput<KeyCode>>,
    tiles: Query<Entity, With<IsEligible>>,
    mut commands: Commands,
) {
    if inputs.just_pressed(KeyCode::Enter) {
        if let Some(tile) = tiles.iter().choose(&mut rand::rng()) {
            commands.trigger(SelectionRequest(tile));
        } else {
            warn!("There were no eligible tiles.")
        }
    }
}
