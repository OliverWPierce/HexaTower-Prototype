use bevy::prelude::*;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

mod backend;
use backend::GameLogic;
mod frontend;
use frontend::InputAndGraphics;

use crate::{backend::BackEndUpdateSystems, frontend::FrontEndUpdateSystems};

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            GameLogic,
            InputAndGraphics,
            MeshPickingPlugin,
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .configure_sets(
            Update,
            (BackEndUpdateSystems, FrontEndUpdateSystems).chain(),
        )
        .run()
}
