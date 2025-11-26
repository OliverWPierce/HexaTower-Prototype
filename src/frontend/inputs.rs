use bevy::prelude::*;

use crate::{
    backend::{game_actions::DoActionOnTile, pieces::OccupiesTile},
    frontend::{visual_pieces::VisPieceOf, visual_tiles::VisTileOf},
};

pub struct InputsPlugin;

impl Plugin for InputsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(tmp_execute_game_action);
    }
}

fn tmp_execute_game_action(
    click: On<Pointer<Click>>,
    mut action_writer: MessageWriter<DoActionOnTile>,
    vis_tiles: Query<&VisTileOf>,
    visual_pieces: Query<&VisPieceOf>,
    logical_piece_occupies: Query<&OccupiesTile>,
) {
    if let Ok(VisTileOf(log_tile)) = vis_tiles.get(click.entity) {
        action_writer.write(DoActionOnTile(*log_tile));
        println!("request sent!")
    }

    if let Ok(VisPieceOf(log_piece)) = visual_pieces.get(click.entity) {
        let tile_hit = logical_piece_occupies
            .get(*log_piece)
            .expect("A logical piece occupied no tile.")
            .log_tile;
        action_writer.write(DoActionOnTile(tile_hit));
    }
}
