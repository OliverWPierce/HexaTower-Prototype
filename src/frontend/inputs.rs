use bevy::prelude::*;

use crate::{
    backend::{
        game_actions::{Selectable, Selected},
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
    selectable: Query<&Selectable>,
    mut commands: Commands,
) {
    if let Ok(VisTileOf(log_tile)) = vis_tiles.get(click.entity) {
        if selectable.contains(*log_tile) {
            commands
                .entity(*log_tile)
                .remove::<Selectable>()
                .insert(Selected);
        }
    } else if let Ok(VisPieceOf(log_piece)) = visual_pieces.get(click.entity) {
        let log_tile = logical_piece_occupies
            .get(*log_piece)
            .expect("A logical piece occupied no tile.")
            .log_tile;

        if selectable.contains(log_tile) {
            commands
                .entity(log_tile)
                .remove::<Selectable>()
                .insert(Selected);
        }
    }
}
