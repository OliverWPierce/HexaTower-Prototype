use bevy::prelude::*;

pub struct GameLogic;

impl Plugin for GameLogic {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PlayerSpawningPlugin,
            RefreshGroupPlugin,
            AppFlow,
            GameParameters,
        ));
    }
}

mod cards;

mod classes;
mod game_parameters;
mod gameflow;
mod players;
mod shop_refresh_groups;

pub use cards::*;
pub use classes::*;
pub use game_parameters::*;
pub use gameflow::*;
pub use players::*;
pub use shop_refresh_groups::*;

#[derive(Component)]
#[relationship(relationship_target = GroupCards)]
struct CardOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = CardOf)]
struct GroupCards(Vec<Entity>);
