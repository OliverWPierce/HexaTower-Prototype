use bevy::prelude::*;

use crate::{
    backend::{
        game_actions::{SetActionTo, dangerous_selection_mechanics::SelectLogTile},
        pieces::{OccupiesTile, SetPieceToActive},
    },
    frontend::{visual_pieces::VisPieceOf, visual_tiles::VisTileOf},
};

pub struct InputsPlugin;

impl Plugin for InputsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(select_tiles);
        app.add_observer(register_click);
        app.init_resource::<ClickCounter>();

        app.add_systems(Update, cancel_input_sequence);
    }
}

fn select_tiles(
    click: On<Pointer<Click>>,
    vis_tiles: Query<&VisTileOf>,
    vis_pieces: Query<&VisPieceOf>,
    log_pieces: Query<&OccupiesTile>,
    mut meaningful_clicks: ResMut<ClickCounter>,
    mut commands: Commands,
) {
    if let Ok(VisTileOf(log_tile)) = vis_tiles.get(click.entity) {
        commands.trigger(SelectLogTile(*log_tile));
        meaningful_clicks.0 += 1;
    } else if let Ok(VisPieceOf(log_piece)) = vis_pieces.get(click.entity)
        && let Ok(OccupiesTile { log_tile }) = log_pieces.get(*log_piece)
    {
        commands.trigger(SelectLogTile(*log_tile));
        meaningful_clicks.0 += 1;
    }
}
#[derive(Debug, Resource, Default)]
pub struct ClickCounter(pub u8);

fn cancel_input_sequence(mut commands: Commands, mut clicks: ResMut<ClickCounter>) {
    if clicks.0 == 1 {
        commands.trigger(SetActionTo::None);
        commands.trigger(SetPieceToActive(None));
    }

    clicks.0 = 0;
}

fn register_click(_click: On<Pointer<Click>>, mut clicks: ResMut<ClickCounter>) {
    clicks.0 += 1;
}
