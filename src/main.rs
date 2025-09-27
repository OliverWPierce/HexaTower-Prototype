use bevy::{prelude::*, window::WindowMode};

mod backend;
use backend::GameLogic;
mod frontend;
use frontend::InputAndGraphics;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resizable: false,
                    mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                    ..default()
                }),
                ..default()
            }),
            GameLogic,
            InputAndGraphics,
        ))
        .run()
}
