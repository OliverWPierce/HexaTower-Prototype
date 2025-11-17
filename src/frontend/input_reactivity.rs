use std::ops::Deref;

use bevy::{
    ecs::component, log::tracing_subscriber::filter::Targets, prelude::*, transform::commands,
};

use crate::backend::AppState;

#[derive(Debug, Component)]
pub struct Hoverable;

#[derive(Debug, Component)]
pub enum Hovered {
    Rising { level: u8 },
    Stable,
    Falling,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(clicks);

        // clicks:
        app.add_event::<ParameterClick>();
        app.add_event::<CreationClick>();
        app.add_event::<InGameClick>();
    }
}

pub fn clicks(
    trigger: Trigger<Pointer<Click>>,
    state: Res<State<AppState>>,
    mut commands: Commands,
) {
    match state.into_inner().deref() {
        AppState::GameParameters => {
            commands.trigger(ParameterClick(trigger.target()));
        }
        AppState::PlayerCreation => commands.trigger(CreationClick(trigger.target())),
        AppState::InGame => commands.trigger(InGameClick(trigger.target())),
        AppState::PostGame => todo!(),
    };
}

#[derive(Debug, Event, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParameterClick(pub Entity);

#[derive(Debug, Event, PartialEq, Eq, PartialOrd, Ord)]
pub struct CreationClick(pub Entity);

#[derive(Debug, Event)]
pub struct InGameClick(pub Entity);
