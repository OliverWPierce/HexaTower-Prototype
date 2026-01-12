use bevy::prelude::*;

mod backend;
mod frontend;

// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use backend::GameLogic;
use frontend::InputAndGraphics;

use crate::{backend::BackEndSystems, frontend::FrontEndSystems};

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            GameLogic,
            InputAndGraphics,
            MeshPickingPlugin,
            // FrameTimeDiagnosticsPlugin::default(),
            // LogDiagnosticsPlugin::default(),
        ))
        .configure_sets(Update, (BackEndSystems, FrontEndSystems).chain())
        .run()
}
