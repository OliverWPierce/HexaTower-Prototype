use bevy::prelude::*;

use crate::{
    backend::{
        game_parameters::SetUpBoard,
        players::{CreatedLogPlayer, PlayersToCreate},
    },
    frontend::FrontEndSystems,
};

pub struct VisPlayerDataPlugin;

impl Plugin for VisPlayerDataPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, create_front_end_players.in_set(FrontEndSystems));
    }
}
#[derive(Debug, Component)]
pub struct DataForPlayer(pub Entity);

#[derive(Debug, Component)]
pub struct PieceBasePlateModel(pub Handle<Scene>);

#[derive(Debug, Component)]
pub struct DisplayName(pub String);

fn create_front_end_players(
    mut log_players_created: MessageReader<CreatedLogPlayer>,
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    all_instructions: Res<PlayersToCreate>,
) {
    for player_data in log_players_created.read() {
        let Some(instructions) = all_instructions.0.get(player_data.1) else {
            panic!()
        };

        commands.spawn((
            DataForPlayer(player_data.0),
            PieceBasePlateModel(
                asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset(instructions.base_pate_path.clone())),
            ),
            DisplayName(instructions.name.clone()),
        ));
    }
}
