use std::f32::consts::PI;

use bevy::prelude::*;

pub struct TileSelectionIndicationPlugin;

impl Plugin for TileSelectionIndicationPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}

#[derive(Debug, Resource, Clone)]
struct IndicatorHandles {
    selected: Handle<Scene>,
    unselected: Handle<Scene>,
    ineligibe: Handle<Scene>,
}
