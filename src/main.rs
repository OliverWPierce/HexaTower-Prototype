use bevy::prelude::*;

mod backend;
use backend::GameLogic;
mod frontend;
use frontend::InputAndGraphics;

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, GameLogic, InputAndGraphics))
        .run()
}
