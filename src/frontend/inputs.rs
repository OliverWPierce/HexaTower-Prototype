use bevy::prelude::*;
use rand::seq::IndexedRandom;

use crate::{
    backend::{
        cards::{CardAsset, CardAssetLoader, CardHandles},
        game_actions::{CurrentAction, dangerous_selection_mechanics::SelectLogTile},
        pieces::OccupiesTile,
    },
    frontend::{FrontEndSystems, visual_pieces::VisPieceOf, visual_tiles::VisTileOf},
};

pub struct InputsPlugin;

impl Plugin for InputsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(select_tiles);
        app.add_systems(Update, tmp_load_card_action);
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

fn tmp_load_card_action(
    handles: Res<CardHandles>,
    assets: Res<Assets<CardAsset>>,
    mut current_action: ResMut<CurrentAction>,
    inputs: Res<ButtonInput<KeyCode>>,
) {
    if !inputs.just_pressed(KeyCode::Space) {
        return;
    }

    println!("attempting to load the card");

    let mut rng = rand::rng();

    let Some(chosen_card) = handles.0.choose(&mut rng) else {
        error!("there were no fully loaded cards, so no action was loaded.",);
        return;
    };

    let card = assets.get(chosen_card.id());

    if card.is_none() {
        println!("failed to get the card.");
        return;
    }
    println!("Loaded card {}", card.unwrap().name);

    current_action.0 = Some(card.unwrap().action);
}
