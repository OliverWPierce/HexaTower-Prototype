use bevy::prelude::*;

mod backend;
use backend::GameLogic;

fn main() -> AppExit {
    App::new().add_plugins(GameLogic).run()
}
