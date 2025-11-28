use bevy::prelude::*;

use crate::{
    backend::{game_actions::SelectedTiles, pieces::OccupiesTile},
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
    mut selected: ResMut<SelectedTiles>,
) {
    let mut hit_tile: Option<Entity> = None;

    if let Ok(VisTileOf(log_tile)) = vis_tiles.get(click.entity) {
        hit_tile = Some(*log_tile);
    } else if let Ok(VisPieceOf(log_piece)) = visual_pieces.get(click.entity) {
        let log_tile = logical_piece_occupies
            .get(*log_piece)
            .expect("A logical piece occupied no tile.")
            .log_tile;
        hit_tile = Some(log_tile);
    }
    if hit_tile.is_none() {
    } else {
        // add a check for the number of tiles already selected.

        for tile in selected.0.iter() {
            if *tile == hit_tile.unwrap() {
                warn!(
                    "The player selected the same tile twice. Code prevented it from being added to the list twice"
                );
                return;
            }
        }

        selected.0.push(hit_tile.unwrap());
    }
}
