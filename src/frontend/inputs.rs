use bevy::prelude::*;

use crate::{
    backend::{game_actions::SelectLogTile, pieces::OccupiesTile},
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
