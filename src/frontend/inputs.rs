use bevy::prelude::*;

use crate::{backend::game_actions::DoActionOnTile, frontend::visual_tiles::VisTileOf};

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
) {
    if let Ok(VisTileOf(log_tile)) = vis_tiles.get(click.entity) {
        action_writer.write(DoActionOnTile(*log_tile));
        println!("request sent!")
    }
}
