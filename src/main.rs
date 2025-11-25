use bevy::prelude::*;

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
        ))
        .configure_sets(
            Update,
            (BackEndUpdateSystems, FrontEndUpdateSystems).chain(),
        )
        .run()
}
