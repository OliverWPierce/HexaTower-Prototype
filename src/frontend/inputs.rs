use bevy::prelude::*;

use crate::{
    backend::{
        game_actions::{IsEligible, SelectionRequest},
        pieces::OccupiesTile,
    },
    frontend::{visual_pieces::VisPieceOf, visual_tiles::VisTileOf},
};

pub struct InputsPlugin;

impl Plugin for InputsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(select_tiles);
    }
}

fn select_tiles(
    click: On<Pointer<Click>>,
    vis_tiles: Query<&VisTileOf>,
    visual_pieces: Query<&VisPieceOf>,
    logical_piece_occupies: Query<&OccupiesTile>,
    mut commands: Commands,
) {
    if let Ok(VisTileOf(log_tile)) = vis_tiles.get(click.entity) {
        commands.trigger(SelectionRequest(*log_tile));
    } else if let Ok(VisPieceOf(log_piece)) = visual_pieces.get(click.entity) {
        let log_tile = logical_piece_occupies
            .get(*log_piece)
            .expect("A logical piece occupied no tile.")
            .log_tile;

        commands.trigger(SelectionRequest(log_tile));
    }
}
