use bevy::prelude::*;

pub struct InputAndGraphics;

impl Plugin for InputAndGraphics {
    fn build(&self, app: &mut App) {
        todo!()
    }
}

mod themes;

pub use themes::*;
