use bevy::{ecs::component, prelude::*};

#[derive(Debug, Component)]
pub struct Hoverable;

#[derive(Debug, Component)]
pub enum Hovered {
    Rising { level: u8 },
    Stable,
    Falling,
}
